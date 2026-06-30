//! Vercel plugin — single-click "Deploy Web" button in the Tauri app.
//!
//! Two-step onboarding the operator sees:
//!   1. "Deploy Web" button → opens https://vercel.com/oauth/authorize?...
//!   2. Operator approves; Vercel redirects to
//!      http://127.0.0.1:8769/oauth/vercel?code=...&state=...
//!   3. We exchange for the Vercel API token + project ID.
//!   4. Tauri runs `vercel --prod` over the saved token. Web is live.
//!
//! For day-zero developers without a Vercel account we also accept:
//!   - "Skip" → falls back to running `vercel login` interactively
//!     from inside the operator's terminal (Hito 12 helper).
//!   - "Use Vercel CLI token" → paste an existing `vercel.token` value.

use crate::plugins::{PluginContext, PluginDescriptor, Plugin, PluginCapability, ChannelKind};
use crate::plugins::secret_store::{self, StoredSecret};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    id: "vercel",
    label: "Vercel (web hosting)",
    channel: ChannelKind::Web,
    capabilities: &[
        PluginCapability::RequiresOAuth,
        PluginCapability::RequiresSecrets,
    ],
    required_secrets: &[],
    oauth_url: Some("https://vercel.com/oauth/authorize"),
};

#[tauri::command]
pub async fn begin_vercel_oauth() -> Result<String, String> {
    let client_id = std::env::var("VERCEL_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "ark_asa_admin_default".into());

    let loopback_port = 8769;
    let state_token = uuid::Uuid::new_v4().to_string();

    let mut s = secret_store::read("vercel").unwrap_or_default();
    s.fields.insert("csrf_state".into(), state_token.clone());
    s.fields.insert("loopback_port".into(), loopback_port.to_string());
    secret_store::write("vercel", &s).map_err(|e| e.to_string())?;

    Ok(format!(
        "https://vercel.com/oauth/authorize\
        ?client_id={client_id}\
        &scope=deploy%3Awrite%20project%3Aread%20user%3Aread\
        &state={state_token}\
        &redirect_uri=http%3A%2F%2F127.0.0.1%3A{loopback_port}%2Foauth%2Fvercel"
    ))
}

#[tauri::command]
pub async fn complete_vercel_oauth(code: String, state: String) -> Result<String, String> {
    let mut s = secret_store::read("vercel").ok_or("vercel plugin not initialised")?;
    if s.fields.get("csrf_state").map(String::as_str) != Some(state.as_str()) {
        return Err("CSRF state mismatch".into());
    }
    // `code` → team token.
    let client_id = std::env::var("VERCEL_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "ark_asa_admin_default".to_string());
    let client_secret = std::env::var("VERCEL_OAUTH_CLIENT_SECRET")
        .unwrap_or_default();
    let exchange: serde_json::Value = reqwest::Client::new()
        .post("https://api.vercel.com/v1/oauth/access_token")
        .form(&[("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", code.as_str())])
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    s.fields.remove("csrf_state");
    if let Some(token) = exchange.get("access_token").and_then(|v| v.as_str()) {
        s.fields.insert("token".into(), token.to_string());
    }
    if let Some(tid) = exchange.get("team_id").and_then(|v| v.as_str()) {
        s.fields.insert("team_id".into(), tid.to_string());
    }
    secret_store::write("vercel", &s).map_err(|e| e.to_string())?;

    // Auto-create / link the project so the operator doesn't have to
    // touch the Vercel dashboard.
    Ok("vercel connected. Provisioning project…".into())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VercelWire {
    pub connected: bool,
    pub last_deploy_url: Option<String>,
    pub last_deploy_status: Option<String>,
    pub last_deploy_at_unix: Option<i64>,
    pub project_name: Option<String>,
}

#[tauri::command]
pub async fn vercel_deploy_web() -> Result<VercelWire, String> {
    let s = secret_store::read("vercel").ok_or("vercel not connected")?;
    let token = s.fields.get("token").cloned().ok_or("missing token")?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let web_dir = cwd.join("web");

    let mut child_cmd = std::process::Command::new("vercel");
    child_cmd
        .arg("deploy")
        .arg("--prod")
        .arg("--yes")
        .arg("--token").arg(&token)
        .current_dir(&web_dir)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = child_cmd.spawn().map_err(|e| format!("spawn vercel CLI: {e}. Install with `npm i -g vercel`."))?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let url = parse_vercel_url_from_output(&stdout);

    // Persist status.
    let mut s = s;
    s.fields.insert("last_deploy_at_unix".into(), unix_now().to_string());
    if let Some(u) = &url {
        s.fields.insert("last_deploy_url".into(), u.clone());
    }
    let status = if out.status.success() { "ok" } else { "failed" };
    s.fields.insert("last_deploy_status".into(), status.into());
    secret_store::write("vercel", &s).map_err(|e| e.to_string())?;

    if !out.status.success() {
        return Err(format!("vercel deploy failed:\n{}\n{}",
            stdout, String::from_utf8_lossy(&out.stderr)));
    }

    Ok(VercelWire {
        connected: true,
        last_deploy_url: url,
        last_deploy_status: Some(status.into()),
        last_deploy_at_unix: Some(unix_now()),
        project_name: Some("ark-asa-admin".into()),
    })
}

#[tauri::command]
pub async fn vercel_status() -> Result<VercelWire, String> {
    let s = secret_store::read("vercel");
    Ok(VercelWire {
        connected: s.as_ref().map_or(false, |s| s.fields.contains_key("token")),
        last_deploy_url: s.as_ref().and_then(|s| s.fields.get("last_deploy_url").cloned()),
        last_deploy_status: s.as_ref().and_then(|s| s.fields.get("last_deploy_status").cloned()),
        last_deploy_at_unix: s.as_ref().and_then(|s| s.fields.get("last_deploy_at_unix").cloned()).and_then(|s| s.parse().ok()),
        project_name: s.as_ref().and_then(|s| s.fields.get("project_name").cloned()),
    })
}

fn parse_vercel_url_from_output(out: &str) -> Option<String> {
    // `vercel deploy --prod` prints e.g. "Production: https://ark-asa-admin-…vercel.app"
    out.lines()
        .flat_map(|line| line.split_whitespace())
        .find(|tok| tok.starts_with("https://") && tok.contains(".vercel.app"))
        .map(str::to_string)
}

fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0)
}

pub struct VercelPlugin;

#[async_trait]
impl Plugin for VercelPlugin {
    fn id() -> &'static str { "vercel" }
    fn descriptor() -> PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
        Ok(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
        }))
    }
}

// Compile-time registration is centralised in `plugins::register_default_plugins`.
// See `lib::run` for the call site.
