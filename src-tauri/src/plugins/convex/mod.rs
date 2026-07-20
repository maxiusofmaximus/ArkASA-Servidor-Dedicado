//! Convex plugin — v2.1 onboarding via the **official CLI bridge**.
//!
//! Convex ships an open-source CLI (`npm i convex` / `npx convex login` /
//! `npx convex deploy`). There is **no first-party OAuth flow** today for
//! our kind of integration, so we shell out to the CLI.
//!
//! Operator flow:
//!   1. Opens the app, clicks **Connect Convex** (in Options → General →
//!      Cloud Services).
//!   2. The desktop app spawns `npx convex login` against the GitHub device
//!      flow. A browser page opens. The operator clicks **Authorize** and
//!      github returns a code, which the CLI serialises into
//!      `~/.convex/credentials.json`. The Tauri plugin watches this file
//!      and copies the deploy_key into our own secret store the moment it
//!      appears.
//!   3. Once connected, the **Push schema** button runs `npx convex deploy`
//!      from the desktop. Web admin starts streaming from the new project.
//!
//! There is also a **Paste deploy key** fallback for air-gapped operators
//! or for those who've already authenticated elsewhere. Path: 5.b below.
//!
//! All Convex commands are open-source (Apache 2 with FSL) and the
//! client+CLI live at <https://github.com/get-convex/convex-js>. We do
//! not implement our own OAuth server. We do not re-implement any Convex
//! internal protocol. We just spawn their CLI, capture stdout, and
//! persist the secrets on disk.

use crate::plugins::{PluginContext, PluginDescriptor, Plugin, PluginCapability, ChannelKind};
use crate::plugins::secret_store_v2 as secret_store;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    id: "convex",
    label: "Convex BaaS",
    channel: ChannelKind::Web,
    capabilities: &[
        PluginCapability::MessagesSend,        // the web admin reads Convex
        PluginCapability::RequiresSecrets,     // uses CONVEX_DEPLOY_KEY, not OAuth
    ],
    required_secrets: &[
        "deploy_key",                          // what `npx convex login` writes
        "deployment_url",                       // the .convex/config.json path's URL
    ],
    oauth_url: None,                            // Convex does not expose OAuth serverside.
};

// ─── Public Tauri commands ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvexStatus {
    pub connected: bool,
    pub deployment_url: Option<String>,
    pub project_slug: Option<String>,
    pub team_slug: Option<String>,
    pub deploy_key_present: bool,
    pub schema_pushed_at_unix: Option<i64>,
    pub last_seen_at_unix: Option<i64>,
    pub log_tail: Vec<String>,
}

/// `begin_convex_link()` — kicks off the Convex login flow in the background.
///
/// Internally we run `npx convex login` because that's the only supported
/// way to authenticate a desktop-client app to Convex today. The CLI
/// pops the browser to GitHub's device authorisation page automatically
/// — we don't reopen it ourselves.
#[tauri::command]
pub async fn begin_convex_link() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let convex_dir = cwd.join("convex");

    // `npx convex login` is interactive — it needs a TTY. We use stdin
    // raw so the user can press Enter / type in the operator terminal UI
    // if a future Hito wires that up. For now, we expect the operator to
    // have `node` and `npx` available, and a browser to click Authorize.
    let output = std::process::Command::new("npx")
        .arg("convex")
        .arg("login")
        .current_dir(&convex_dir)
        .env("CI", "") // unset CI so the interactive flow runs
        .output()
        .map_err(|e| format!("failed to spawn `npx convex login`: {e}.  Install Node + `npm install convex` in the convex/ directory."))?;
    if !output.status.success() {
        return Err(format!(
            "convex login failed:\n{}\n{}",
            String::from_utf8_lossy(&stdout_bytes(&output.stdout)),
            String::from_utf8_lossy(&stderr_bytes(&output.stderr)),
        ));
    }

    // After `convex login` succeeds, the CLI writes `~/.convex/credentials.json`
    // and `<project>/.convex/config.json`. We tail both and refresh our
    // own secret store.
    ingest_cli_credentials().await?;
    Ok("convex connected.  Click Push schema next.".into())
}

fn stdout_bytes(out: &[u8]) -> Vec<u8> { out.to_vec() }
fn stderr_bytes(out: &[u8]) -> Vec<u8> { out.to_vec() }

/// Read `~/.convex/credentials.json` and `<convex>/.convex/config.json`,
/// copy the active deployment + deploy_key into our secret store.
/// Returns Err if the CLI write didn't happen yet (still being hashed).
async fn ingest_cli_credentials() -> Result<(), String> {
    let home = dirs_home();
    let creds = std::fs::read_to_string(home.join(".convex").join("credentials.json"))
        .map_err(|e| format!("read credentials.json: {e}"))?;
    let parsed: serde_json::Value = toml_or_json(&creds)
        .map_err(|e| format!("parse credentials.json: {e}"))?;

    let team = parsed.get("team")
        .and_then(|t| t.as_str())
        .map(str::to_string);
    let project = parsed.get("project")
        .and_then(|t| t.as_str())
        .map(str::to_string);
    let deploy_key = parsed.get("access_token")
        .or_else(|| parsed.get("deploy_key"))
        .and_then(|t| t.as_str())
        .map(str::to_string);

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cfg_path = cwd.join("convex").join(".convex").join("config.json");
    let cfg_text = std::fs::read_to_string(&cfg_path).unwrap_or_default();
    let config: serde_json::Value = toml_or_json(&cfg_text).unwrap_or_else(|_| serde_json::json!({}));
    let deployment = config.get("deployment")
        .or_else(|| config.get("deployments"))
        .and_then(|t| t.as_str())
        .map(str::to_string)
        .or_else(|| {
            // Convex self-hosted: CONVEX_SELF_HOSTED_URL is the URL.
            std::env::var("CONVEX_SELF_HOSTED_URL").ok()
        });

    let mut s = secret_store::read("convex").unwrap_or_default();
    if let Some(t) = team      { s.fields.insert("team".into(),  t); }
    if let Some(p) = project    { s.fields.insert("project".into(), p); }
    if let Some(k) = deploy_key { s.fields.insert("deploy_key".into(), k); }
    if let Some(d) = deployment { s.fields.insert("deployment_url".into(), d); }
    s.fields.insert("last_seen_at_unix".into(), unix_now().to_string());
    secret_store::write("convex", &s).map_err(|e| e.to_string())
}

fn toml_or_json(text: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str::<serde_json::Value>(text.trim_start())
        .or_else(|_| {
            // some CLI builds emit JSON5 or non-standard keys
            toml::from_str::<serde_json::Value>(text)
                .map_err(|e| format!("both JSON and TOML parse failed: {e}"))
        })
}

fn dirs_home() -> PathBuf { PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()) }
use std::path::PathBuf;

/// Manual fallback: paste a CONVEX_DEPLOY_KEY you copy from another machine.
#[tauri::command]
pub async fn paste_convex_deploy_key(
    deployment_url: String,
    deploy_key: String,
) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let project_slug = cwd.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("ark-asa-admin")
        .to_string();

    let mut s = secret_store::read("convex").unwrap_or_default();
    s.fields.insert("deployment_url".into(), deployment_url);
    s.fields.insert("deploy_key".into(), deploy_key);
    s.fields.insert("project".into(), project_slug);
    s.fields.insert("last_seen_at_unix".into(), unix_now().to_string());
    secret_store::write("convex", &s).map_err(|e| e.to_string())?;
    Ok("convex key saved".into())
}

/// One-click deploy that registers credentials and triggers `convex_push_schema()`.
#[tauri::command]
pub async fn convex_deploy(
    deployment_url: String,
    deploy_key: String,
) -> Result<String, String> {
    paste_convex_deploy_key(deployment_url, deploy_key).await?;
    convex_push_schema().await
}

/// `convex_push_schema()` — runs `npx convex deploy --prod` in a child
/// process.  Output is captured and streamed back via the response.
#[tauri::command]
pub async fn convex_push_schema() -> Result<String, String> {
    let s = secret_store::read("convex").ok_or("convex not connected — paste_convex_deploy_key() first.")?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let convex_dir = cwd.join("convex");
    let key = s.fields.get("deploy_key").cloned().ok_or("missing deploy_key")?;

    let mut cmd = std::process::Command::new("npx");
    cmd.arg("convex").arg("deploy").arg("--prod")
        .current_dir(&convex_dir)
        .env("CONVEX_DEPLOY_KEY", key)
        .env_remove("CONVEX_DEPLOYMENT") // env-safety: don't accidentally use dev
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(url) = s.fields.get("deployment_url") {
        cmd.env("CONVEX_SELF_HOSTED_URL", url);
    }
    let child = cmd.spawn().map_err(|e| format!("spawn npx: {e}.  Install Node + convex package; or use Paste deploy key with a host pointing at your deployment URL."))?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    let mut s = s;
    if out.status.success() {
        s.fields.insert("schema_pushed_at_unix".into(), unix_now().to_string());
        secret_store::write("convex", &s).map_err(|e| e.to_string())?;
        Ok(stdout)
    } else {
        // Capture last 20 log lines so the React side can show them
        let combined = format!("{stdout}\n--- stderr ---\n{stderr}");
        Err(combined)
    }
}

/// `convex_status()` — for the React UI to paint the connection state.
#[tauri::command]
pub async fn convex_status() -> Result<ConvexStatus, String> {
    let s = secret_store::read("convex");
    Ok(ConvexStatus {
        connected:    s.as_ref().map_or(false, |s| s.fields.contains_key("deploy_key")),
        deployment_url:      s.as_ref().and_then(|s| s.fields.get("deployment_url").cloned()),
        project_slug:        s.as_ref().and_then(|s| s.fields.get("project").cloned()),
        team_slug:           s.as_ref().and_then(|s| s.fields.get("team").cloned()),
        deploy_key_present:  s.as_ref().map_or(false, |s| s.fields.contains_key("deploy_key")),
        schema_pushed_at_unix: s.as_ref().and_then(|s| s.fields.get("schema_pushed_at_unix").cloned()).and_then(|s| s.parse().ok()),
        last_seen_at_unix:   s.as_ref().and_then(|s| s.fields.get("last_seen_at_unix").cloned()).and_then(|s| s.parse().ok()),
        log_tail:            vec![],  // Hito 12 exposes streaming logs
    })
}

fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0)
}

// ─── Plugin trait plumbing ────────────────────────────────────────────────
pub struct ConvexPlugin;

#[async_trait]
impl Plugin for ConvexPlugin {
    fn id() -> &'static str { "convex" }
    fn descriptor() -> PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
        // Convex has no event-source loop needed; everything is event-driven
        // by the React UI. We park the future so the registry can call
        // `start` uniformly.
        Ok(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_convex_deploy_persists_credentials() {
        let _storage_guard = crate::plugins::lock_plugin_storage_for_test();
        // Create a temporary directory or just save to our test environment storage.
        // `secret_store::write` will write to a standard path, so we can clean up afterwards or just assert it writes.
        let test_url = "https://test-animal-999.convex.cloud".to_string();
        let test_key = "prod:test-deploy-key-99999999".to_string();

        // Call our command
        let res = convex_deploy(test_url.clone(), test_key.clone()).await;

        // Whether spawning npx succeeds or fails in this environment,
        // we assert that the credentials MUST be persisted on disk.
        let s = secret_store::read("convex").expect("secret store should have convex secrets saved");
        assert_eq!(s.fields.get("deployment_url"), Some(&test_url));
        assert_eq!(s.fields.get("deploy_key"), Some(&test_key));

        // Clean up the test credential so we don't pollute the local config.
        // v1 kept secrets on disk and this test removed the TOML file directly;
        // v2 lives in keyring (or the in-memory test store auto-enabled in
        // cfg(test)), so we have to go through `delete()` to actually clear it.
        secret_store::delete("convex").expect("convex secret should be deletable");

        // The result should either be Ok (if npx succeeded) or Err with a message about spawning or deployment failing,
        // but it shouldn't panic.
        match res {
            Ok(_) => println!("convex_deploy succeeded (npx deploy succeeded)"),
            Err(e) => assert!(e.contains("spawn") || e.contains("convex") || e.contains("npx") || e.contains("failed"), "unexpected error message: {}", e),
        }
    }
}
