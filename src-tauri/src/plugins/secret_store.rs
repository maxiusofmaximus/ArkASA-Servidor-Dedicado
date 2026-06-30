//! Provider secret lifecycle — used by every plugin that has operator
//! credentials they don't want to type into a TOML.
//!
//! Saved on disk under `~/.ark-asa/plugins/<plugin_id>/secret.toml` (root-only
//! file perms when possible). For OAuth-style providers the secret is a
//! single opaque token (PAT or refresh_token). For bot-text providers it's
//! a TOML map mirroring what we'd read into the plugin's `PluginSecrets`.
//!
//! Storage is deliberately "good enough for a single-operator desktop app":
//! it does NOT defend against a stolen disk image. In production you'd use
//! OS keychain (Windows DPAPI / macOS Keychain / Linux libsecret) via the
//! `keyring` crate. This is Hito 12 territory.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredSecret {
    /// Last write unix epoch seconds.
    pub updated_at_unix: i64,
    /// Free-form key/value pairs. Interpretation depends on the plugin:
    /// - `convex`           : `{"deployment_url": "https://...convex.cloud", "deploy_key": "dev:..."}`
    /// - `vercel`           : `{"team_id": "...", "token": "...", "project_id": "..."}`
    /// - `telegram`         : `{"bot_token": "123456:ABC", "admin_chat_ids": "987654321"}` (id list joined by `,`)
    /// - `discord`          : `{"bot_token": "...", "guild_id": "...", "admin_user_ids": "..."}`
    /// - etc.
    pub fields: BTreeMap<String, String>,
}

pub fn secret_path(plugin_id: &str) -> std::path::PathBuf {
    let mut p = super::plugin_storage_dir();
    p.push(format!("{}.toml", plugin_id));
    p
}

pub fn read(plugin_id: &str) -> Option<StoredSecret> {
    let p = secret_path(plugin_id);
    let raw = std::fs::read_to_string(&p).ok()?;
    // Strip the trailing newline so serde doesn't choke on trailing CRLF.
    match toml::from_str::<StoredSecret>(&raw) {
        Ok(s)  => Some(s),
        Err(e) => {
            log::warn!("secret file {p:?} is corrupt: {e}");
            None
        }
    }
}

/// Persists secret to disk atomically (write to temp, rename).
pub fn write(plugin_id: &str, value: &StoredSecret) -> Result<(), String> {
    let p = secret_path(plugin_id);
    let dir = p.parent().ok_or("secret path missing parent")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("create {dir:?}: {e}"))?;
    let tmp = dir.join(format!(".{}.tmp", plugin_id));
    let serialised = toml::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, serialised).map_err(|e| format!("write tmp: {e}"))?;
    std::fs::rename(&tmp, &p).map_err(|e| format!("rename: {e}"))?;
    apply_0600_perms(&p);
    Ok(())
}

#[cfg(unix)]
fn apply_0600_perms(p: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600)) {
        log::warn!("set 0600 on {p:?}: {e}");
    }
}

#[cfg(not(unix))]
fn apply_0600_perms(_p: &std::path::Path) {
    // Windows: ACLs by default restrict to current user; best we can.
}
