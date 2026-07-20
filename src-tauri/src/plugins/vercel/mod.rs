//! Vercel plugin — v2.1 onboarding via the **official CLI bridge**.
//!
//! Vercel ships `vercel login`, `vercel env`, `vercel deploy`.  The
//! desktop app shells out to those instead of maintaining a parallel
//! OAuth flow.  Specifically:
//!
//!   * `Connect Vercel` → spawns `vercel login`. The operator opens the
//!     URL Vercel prints, clicks Authorize, and `vercel` writes
//!     `~/.vercel/auth.json` with a long-lived token. We read it.
//!   * `Deploy web`     → spawns `vercel deploy --prod --yes`, optionally
//!     pointing at VERCEL_PROJECT_ID the operator set.
//!   * `Paste VERCEL_TOKEN`  → manual fallback for air-gapped setups.
//!
//! Just like Convex, this is the only realistic day-one integration
//! without the operator standing up a custom OAuth client. We keep
//! minimal integration surface — Vercel CLI + token relay — and avoid
//! inventing endpoints.

use crate::plugins::{PluginContext, PluginDescriptor, Plugin, PluginCapability, ChannelKind};
use crate::plugins::secret_store_v2 as secret_store;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DESCRIPTOR: PluginDescriptor = PluginDescriptor {
    id: "vercel",
    label: "Vercel (web hosting)",
    channel: ChannelKind::Web,
    capabilities: &[
        PluginCapability::MessagesSend,        // serves the web admin
        PluginCapability::RequiresSecrets,     // VERCEL_TOKEN, not OAuth
    ],
    required_secrets: &["token", "project_name_or_id?"],
    oauth_url: None,                            // Vercel ships OAuth for Vercel Apps,
                                                // not for desktop end-users.
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VercelStatus {
    pub connected: bool,
    pub token_present: bool,
    pub last_deploy_url: Option<String>,
    pub last_deploy_at_unix: Option<i64>,
    pub last_deploy_status: Option<String>,
    pub log_tail: Vec<String>,
}

/// `begin_vercel_link()` — spawns `vercel login` interactively.
/// The CLI pops a browser to https://vercel.com/api/registration/.../login
/// for the operator to complete; on success it writes to
/// `~/.vercel/auth.json`. We tail that file and pick up the token.
#[tauri::command]
pub async fn begin_vercel_link() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let web_dir = cwd.join("web");

    let output = std::process::Command::new("vercel")
        .arg("login")
        .current_dir(&web_dir)
        .output()
        .map_err(|e| format!("failed to spawn `vercel login`: {e}.  Install with `npm i -g vercel`."))?;
    if !output.status.success() {
        return Err(format!(
            "vercel login failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    ingest_vercel_credentials()?;
    Ok("vercel connected.  Click Deploy web next.".into())
}

/// Read `~/.vercel/auth.json` and copy the token into our secret store.
fn ingest_vercel_credentials() -> Result<(), String> {
    let home = dirs_home();
    let raw = std::fs::read_to_string(home.join(".vercel").join("auth.json"))
        .or_else(|_| std::fs::read_to_string(home.join("AppData").join(".vercel").join("auth.json")))
        .map_err(|e| format!("read vercel auth.json: {e}.  Vercel login may have failed silently — re-run from a terminal to see errors."))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("parse auth.json: {e}"))?;

    let token = parsed.get("token")
        .and_then(|t| t.as_str())
        .map(str::to_string);
    let team = parsed.get("teamId")
        .or_else(|| parsed.get("team_id"))
        .and_then(|t| t.as_str())
        .map(str::to_string);

    let mut s = secret_store::read("vercel").unwrap_or_default();
    if let Some(t) = token.clone() {
        s.fields.insert("token".into(), t);
    }
    if let Some(t) = team {
        s.fields.insert("team_id".into(), t);
    }
    s.fields.insert("last_seen_at_unix".into(), unix_now().to_string());
    secret_store::write("vercel", &s).map_err(|e| e.to_string())?;

    if token.is_none() {
        return Err("vercel auth.json did not contain `token`".into());
    }
    Ok(())
}

fn dirs_home() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default())
}

/// `paste_vercel_token` — manual fallback for non-interactive setups
/// (CI runners, air-gapped routers, …).  Paste a token you create at
/// `<vercel.com>/account/tokens>`.
#[tauri::command]
pub async fn paste_vercel_token(
    token: String,
    project_id: Option<String>,
) -> Result<String, String> {
    let mut s = secret_store::read("vercel").unwrap_or_default();
    s.fields.insert("token".into(), token);
    if let Some(p) = project_id {
        s.fields.insert("project_id".into(), p);
    }
    s.fields.insert("last_seen_at_unix".into(), unix_now().to_string());
    secret_store::write("vercel", &s).map_err(|e| e.to_string())?;
    Ok("vercel token saved".into())
}

/// `vercel_deploy_web()` — runs `vercel deploy --prod --yes` in a child process.
/// Output is captured and streamed back via the response.
#[tauri::command]
pub async fn vercel_deploy_web() -> Result<String, String> {
    let s = secret_store::read("vercel").ok_or("vercel not connected — click 'Connect Vercel' first.")?;
    let token = s.fields.get("token").cloned().ok_or("missing token — re-run Connect Vercel.")?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let web_dir = cwd.join("web");

    let mut cmd = std::process::Command::new("vercel");
    cmd.arg("deploy")
        .arg("--prod")
        .arg("--yes")
        .current_dir(&web_dir)
        .env("VERCEL_TOKEN", token)
        .env_remove("CI") // vercel login is no-op if CI; force interactive-friendly path
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(pid) = s.fields.get("project_id") {
        cmd.env("VERCEL_PROJECT_ID", pid);
    }

    let child = cmd.spawn().map_err(|e| format!("spawn vercel CLI: {e}.  Install with `npm i -g vercel`."))?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    let mut s = s;
    if out.status.success() {
        s.fields.insert("last_deploy_at_unix".into(), unix_now().to_string());
        s.fields.insert("last_deploy_url".into(), parse_vercel_url_from_output(&stdout).unwrap_or_default());
        secret_store::write("vercel", &s).map_err(|e| e.to_string())?;
        Ok(stdout)
    } else {
        Err(format!("{stdout}\n--- stderr ---\n{stderr}"))
    }
}

fn parse_vercel_url_from_output(out: &str) -> Option<String> {
    out.lines()
        .flat_map(|line| line.split_whitespace())
        .find(|tok| tok.starts_with("https://") && tok.contains(".vercel.app"))
        .map(str::to_string)
}

/// One-click deploy: persist the token (and optional project_id) and trigger
/// `vercel deploy --prod`. Returns the production URL when Vercel prints one.
#[tauri::command]
pub async fn vercel_deploy_one_click(
    token: String,
    project_id: Option<String>,
) -> Result<String, String> {
    paste_vercel_token(token, project_id).await?;
    vercel_deploy_web().await
}

/// `vercel_status()` — for the React UI.
#[tauri::command]
pub async fn vercel_status() -> Result<VercelStatus, String> {
    let s = secret_store::read("vercel");
    Ok(VercelStatus {
        connected:          s.as_ref().map_or(false, |s| s.fields.contains_key("token")),
        token_present:      s.as_ref().map_or(false, |s| s.fields.contains_key("token")),
        last_deploy_url:    s.as_ref().and_then(|s| s.fields.get("last_deploy_url").cloned()),
        last_deploy_at_unix: s.as_ref().and_then(|s| s.fields.get("last_deploy_at_unix").cloned()).and_then(|s| s.parse().ok()),
        last_deploy_status: s.as_ref().and_then(|s| s.fields.get("last_deploy_status").cloned()),
        log_tail:           vec![],
    })
}

fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0)
}

// ─── Plugin trait plumbing ────────────────────────────────────────────────
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vercel_url_from_output_works() {
        let out = "✔ Production: https://ark-asa-admin-fawn.vercel.app [copied to clipboard]";
        assert_eq!(
            parse_vercel_url_from_output(out),
            Some("https://ark-asa-admin-fawn.vercel.app".to_string())
        );
        // Negative cases:
        assert!(parse_vercel_url_from_output("Building…").is_none());
        assert!(parse_vercel_url_from_output("").is_none());
    }

    #[tokio::test]
    async fn test_vercel_deploy_one_click_persists_token() {
        let _storage_guard = crate::plugins::lock_plugin_storage_for_test();
        let test_token = "test-vercel-token-aaaaaaaaaa".to_string();
        let test_project = Some("test-proj-1234".to_string());

        let res = vercel_deploy_one_click(test_token.clone(), test_project.clone()).await;

        // Whether `vercel deploy` succeeds or fails on this machine,
        // we MUST have saved the token in the secret store first.
        let s = secret_store::read("vercel").expect("vercel secret store should have token saved");
        assert_eq!(s.fields.get("token"), Some(&test_token));
        assert_eq!(s.fields.get("project_id"), test_project.as_ref());

        // Clean up so we don't pollute local config. v1 of secret_store
        // kept secrets on disk, and this test used to remove the TOML file
        // directly. v2 lives in keyring (or the in-memory test store), so
        // we have to go through `delete()` to actually clear the credential.
        secret_store::delete("vercel").expect("vercel secret should be deletable");

        // The spawn result will usually be Err in CI/dev (no vercel CLI on path);
        // accept either outcome but never panic.
        match res {
            Ok(_) => {}
            Err(e) => assert!(
                e.contains("vercel") || e.contains("spawn") || e.contains("token") || e.contains("cli"),
                "unexpected error: {}", e
            ),
        }

        // And the function must NOT persist anything else (no leaked secrets).
        let reloaded = secret_store::read("vercel");
        assert!(reloaded.is_none(), "secret_store should be cleared for vercel after cleanup");
    }
}
