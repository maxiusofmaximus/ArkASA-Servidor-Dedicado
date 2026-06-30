//! Convex outbound publisher.
//!
//! Periodically serializes the current ARK cluster state and pushes it to
//! the configured Convex deployment via the `convex/internal/servers:upsert`
//! action (defined in convex/convex/servers.ts in Hito 3).
//!
//! Authentication: HMAC-SHA256 of the body using a shared secret configured
//! in TOML under `[integrations] convex_secret = "..."`. The Convex action
//! verifies the HMAC before performing any writes.
//!
//! Failure handling: this publisher **never panics**. It logs and continues.
//! State staleness in Convex is recovered by the next successful push.

use crate::config::schema::ServerConfig;
use crate::integrations::http_api::AdminApiState;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

const PUSH_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatePushPayload {
    host_id: String,
    session_name: String,
    motd: Option<String>,
    cluster_maps: Vec<String>,
    map_statuses: Vec<ConvexMapStatus>,
    last_seen_ms: i64,
    signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConvexMapStatus {
    map_index: u32,
    map_id: String,
    map_label: String,
    running: bool,
}

/// Spawn the periodic Convex push task. Returns the JoinHandle so the caller
/// can cancel it during shutdown.
pub async fn spawn_publisher(
    api: Arc<AdminApiState>,
    convex_url: String,
    convex_secret: String,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .expect("reqwest client");
        let mut tick = interval(Duration::from_secs(PUSH_INTERVAL_SECS));
        tick.tick().await; // immediate first tick skipped
        loop {
            tick.tick().await;
            if let Err(e) = push_once(&api, &convex_url, &convex_secret, &client).await {
                log::warn!("convex push failed: {e}");
            }
        }
    })
}

async fn push_once(
    api:       &Arc<AdminApiState>,
    convex_url:&str,
    secret:    &str,
    client:    &Client,
) -> Result<(), String> {
    // Pull the latest state from the HTTP API server's in-memory snapshot.
    let snap = api.snapshot().await;
    let cfg  = match &*api.config_snapshot.read().await {
        Some(c) => c.clone(),
        None    => return Ok(()), // nothing to push yet
    };

    let payload = StatePushPayload {
        host_id: api.host_id.clone(),
        session_name: cfg.identification.session_name.clone(),
        motd: if cfg.identification.server_message_of_the_day.is_empty() {
            None
        } else {
            Some(cfg.identification.server_message_of_the_day.clone())
        },
        cluster_maps: cfg.cluster_maps.clone(),
        map_statuses: snap.map_statuses.into_iter().map(|m| ConvexMapStatus {
            map_index: m.map_index as u32,
            map_id:    m.map_id,
            map_label: m.map_label,
            running:   m.running,
        }).collect(),
        last_seen_ms: chrono::Utc::now().timestamp_millis(),
        signature_hex: String::new(), // to be computed
    };

    // Sign the body (everything except the signature field). The Convex
    // action verifies by re-computing the HMAC.
    let mut body = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    let sig = compute_signature(&payload, secret);
    body["signature_hex"] = serde_json::Value::String(sig.clone());

    let url = format!("{}/api/internal/servers/upsert", convex_url.trim_end_matches('/'));
    let resp = client.post(&url).json(&body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("convex responded {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }
    Ok(())
}

fn compute_signature(payload: &StatePushPayload, secret: &str) -> String {
    let mut body = serde_json::to_value(payload).expect("serialize");
    body.as_object_mut().unwrap().remove("signature_hex");
    let canonical = canonicalize_json(&body);
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(canonical.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Deterministic JSON canonicalization (sorted object keys, no whitespace).
/// Needed so that HMAC inputs match between sender and Verifier.
fn canonicalize_json(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "null".into(),
        serde_json::Value::Bool(b) => if *b { "true".into() } else { "false".into() },
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        serde_json::Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(canonicalize_json).collect();
            format!("[{}]", parts.join(","))
        }
        serde_json::Value::Object(obj) => {
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys.into_iter().map(|k| {
                format!("\"{}\":{}", k.replace('\\', "\\\\").replace('"', "\\\""), canonicalize_json(&obj[k]))
            }).collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

/// Helper used by `lib.rs` to detect whether the operator enabled Convex
/// push in TOML. Centralising here keeps `lib.rs` free of TOML shape.
pub fn is_enabled(cfg: &ServerConfig) -> bool {
    cfg.paths.server_dir.contains("convex")
        || std::env::var("CONVEX_URL").is_ok()
}
