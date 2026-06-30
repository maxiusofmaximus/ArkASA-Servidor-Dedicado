//! Convex plugin — the v2.1 way to onboard the cloud BaaS without a
//! manual URL/key paste into TOML.
//!
//! Flow the operator sees in the Tauri app (Options → General → Convex):
//!   1. "Connect Convex" button.
//!   2. Desktop app opens browser to https://auth.convex.dev/...
//!   3. Operator approves + clicks "Authorize"; Convex redirects to
//!      `http://127.0.0.1:8768/oauth/callback?code=…`.
//!   4. We exchange the code for a deploy_key + deployment_url.
//!   5. Save to `~/.ark-asa/plugins/convex.toml`.
//!   6. Auto-push our `/convex/convex/*` to their deployment.
//!   7. Tauri loopback HTTP API now knows how to talk to Convex.
//!
//! NOTE: Convex's actual deployment-key issuance happens via
//! `convex deploy --prod` from a local CLI login. The OAuth-style flow
//! above is modelled on Tailscale's node-key exchange, which is a similar
//! pattern. For Convex we *also* ship a fallback CLI-bridge flow in
//! Hito 12 — operator can paste the `npx convex login`-generated
//! `CONVEX_DEPLOY_KEY` from `.convex/config.json` if the OAuth path
//! ever changes.
//!
//! Either way: zero TOML edits, zero deploy commands. The plugin
//! does it all.

use crate::plugins::{PluginContext, PluginDescriptor, Plugin, PluginCapability, ChannelKind};
use crate::plugins::secret_store::{self, StoredSecret};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    id: "convex",
    label: "Convex BaaS",
    channel: ChannelKind::Web,
    capabilities: &[
        PluginCapability::RequiresOAuth,
        PluginCapability::RequiresSecrets,
    ],
    required_secrets: &[],
    oauth_url: Some("https://auth.convex.dev/oauth/authorize"),
};

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

/// Tauri command — invoked by the React UI when the operator clicks
/// "Connect Convex". Returns the URL the desktop should open in the
/// default browser. The actual code-for-key exchange happens in
/// `convex::complete_oauth`.
#[tauri::command]
pub async fn begin_convex_oauth() -> Result<String, String> {
    // Real client_id for "ARK ASA Configuration Manager" lives in Convex
    // dashboard. We ship a default one; operators self-serve to
    // replace it with their own OAuth app for fully white-labelled deploys.
    let client_id = std::env::var("ARK_ASA_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "ark-asa-config-manager-default".into());

    // State carries a CSRF token + the local loopback port.
    let loopback_port = 8768;
    let state_token = uuid::Uuid::new_v4().to_string();

    // Persist state token so `convex::complete_oauth` can validate it.
    let mut s = secret_store::read("convex").unwrap_or_default();
    s.fields.insert("csrf_state".into(), state_token.clone());
    s.fields.insert("loopback_port".into(), loopback_port.to_string());
    secret_store::write("convex", &s).map_err(|e| e.to_string())?;

    let url = format!(
        "https://auth.convex.dev/oauth/authorize\
        ?client_id={client_id}\
        &scope=deploy%3Awrite%20project%3Aread\
        &state={state_token}\
        &redirect_uri=http%3A%2F%2F127.0.0.1%3A{loopback_port}%2Foauth%2Fcallback"
    );
    Ok(url)
}

/// Tauri command — receives the OAuth callback URL captured by the
/// local HTTP callback server (we embed that into the same
/// `http_api.rs` axum router as the loopback, but tied to port 8768).
#[tauri::command]
pub async fn complete_convex_oauth(code: String, state: String) -> Result<String, String> {
    let mut s = secret_store::read("convex").ok_or("convex plugin not initialised")?;
    if s.fields.get("csrf_state").map(String::as_str) != Some(state.as_str()) {
        return Err("CSRF state mismatch — possible MITM, refusing".to_string());
    }
    // Exchange `code` → { deployment_url, deploy_key, team_slug, project_slug }.
    let client_id = std::env::var("ARK_ASA_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "ark-asa-config-manager-default".to_string());
    let exchange: serde_json::Value = reqwest::Client::new()
        .post("https://auth.convex.dev/oauth/token")
        .form(&[("grant_type", "authorization_code"),
                ("code",      code.as_str()),
                ("client_id", client_id.as_str())])
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    if let Some(err) = exchange.get("error") {
        return Err(format!("oauth error: {err}"));
    }

    // Persist credentials.  Until Convex ships a real deployment-key
    // OAuth flow we also accept CLI-flow keys via `paste_deploy_key`.
    s.fields.remove("csrf_state");
    if let Some(url) = exchange.get("deployment_url").and_then(|v| v.as_str()) {
        s.fields.insert("deployment_url".into(), url.to_string());
    }
    if let Some(key) = exchange.get("deploy_key").and_then(|v| v.as_str()) {
        s.fields.insert("deploy_key".into(), key.to_string());
    }
    secret_store::write("convex", &s).map_err(|e| e.to_string())?;

    Ok("convex connected. Now auto-deploying schema…".into())
}

/// Tauri command — paste a deploy key the operator obtained by running
/// `npx convex login` on another machine. (Belt-and-braces.)
#[tauri::command]
pub async fn paste_convex_deploy_key(
    deployment_url: String,
    deploy_key: String,
) -> Result<String, String> {
    let mut s = StoredSecret { updated_at_unix: unix_now(), ..Default::default() };
    s.fields.insert("deployment_url".into(), deployment_url);
    s.fields.insert("deploy_key".into(), deploy_key);
    secret_store::write("convex", &s).map_err(|e| e.to_string())?;
    Ok("convex key saved".into())
}

#[tauri::command]
pub async fn convex_status() -> Result<ConvexWire, String> {
    let s = secret_store::read("convex");
    Ok(ConvexWire {
        connected: s.as_ref().map_or(false, |s| s.fields.contains_key("deployment_url")),
        deployment_url: s.as_ref().and_then(|s| s.fields.get("deployment_url").cloned()),
        deploy_key_present: s.as_ref().map_or(false, |s| s.fields.contains_key("deploy_key")),
        schema_pushed_at_unix: s.as_ref().and_then(|s| s.fields.get("schema_pushed_at_unix").cloned()).and_then(|s| s.parse().ok()),
        last_seen_at_unix: s.as_ref().and_then(|s| s.fields.get("last_seen_at_unix").cloned()).and_then(|s| s.parse().ok()),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConvexWire {
    pub connected: bool,
    pub deployment_url: Option<String>,
    pub deploy_key_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_pushed_at_unix: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at_unix: Option<i64>,
}

fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Pushes our `/convex/convex/*` schema + functions to the Convex
/// deployment using the saved deploy key. Called automatically by
/// `lib::run()` after a successful OAuth, and also exposed as a
/// Tauri command for the operator to manually re-run.
#[tauri::command]
pub async fn convex_push_schema() -> Result<String, String> {
    let s = secret_store::read("convex").ok_or("not connected")?;
    let url = s.fields.get("deployment_url").cloned().ok_or("missing deployment_url")?;
    let key = s.fields.get("deploy_key").cloned().ok_or("missing deploy_key")?;

    // The actual `convex deploy` is a child process.  In production we
    // call into Convex's REST API directly via the deploy_key, but for
    // v2.1.0-alpha we shell out: the operator has Node + pnpm on the
    // box anyway (the dev stack requires it).
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let convex_dir = cwd.join("convex");
    let child = std::process::Command::new("npx")
        .arg("convex")
        .arg("deploy")
        .arg("--prod")
        .arg("--url").arg(&url)
        .arg("--deploy-key").arg(&key)
        .current_dir(convex_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn npx convex deploy: {e}"))?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "convex deploy failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    // Stamp timestamp.
    let mut s = s;
    s.fields.insert("schema_pushed_at_unix".into(), unix_now().to_string());
    secret_store::write("convex", &s).map_err(|e| e.to_string())?;

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Plugin trait implementation.  Called by `lib::run()` once after
/// the loopback HTTP server is up.  At startup we don't actually have
/// to "start" anything — every interaction is operator-driven.  The
/// plugin is mostly here so its descriptor shows up in `Options →
/// Plugins`.
pub struct ConvexPlugin;

#[async_trait]
impl Plugin for ConvexPlugin {
    fn id() -> &'static str { "convex" }
    fn descriptor() -> PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
        // No-op: Convex push is event-driven by the React UI.
        Ok(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
        }))
    }
}

// Compile-time registration is centralised in `plugins::register_default_plugins`.
// See `lib::run` for the call site.
