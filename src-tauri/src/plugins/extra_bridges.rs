//! Tauri bridge commands for plugins added in Session 8 — Signal,
//! WeChat, SSH, REST. WhatsApp's bridge lives in
//! `whatsapp_bridge.rs` (Session 7); the four here have the same
//! shape so the GUI can iterate uniformly: persistent secret-store,
//! plus a status probe that surfaces admin counts and readiness.

use crate::plugins::secret_store;
use serde::Serialize;

// ─── Signal ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct SignalStatus {
    pub configured: bool,
    pub phone_e164:     String,
    pub admin_count:    usize,
}

#[tauri::command]
pub async fn paste_signal_credentials(
    phone_e164:     String,
    admin_e164s:    String,
    signal_cli_bin: String,
) -> Result<String, String> {
    let mut s = secret_store::read("signal").unwrap_or_default();
    if !phone_e164.is_empty()     { s.fields.insert("phone_e164".into(),     phone_e164); }
    if !admin_e164s.is_empty()    { s.fields.insert("admin_e164s".into(),    admin_e164s); }
    if !signal_cli_bin.is_empty() { s.fields.insert("signal_cli_bin".into(), signal_cli_bin); }
    secret_store::write("signal", &s).map_err(|e| e.to_string())?;
    Ok("signal credentials saved".into())
}

#[tauri::command]
pub async fn signal_status() -> Result<SignalStatus, String> {
    use crate::integrations::signal::SignalConfig;
    let cfg = SignalConfig::from_secrets_or_env();
    Ok(SignalStatus {
        configured: !cfg.phone_e164.is_empty() && !cfg.signal_cli_bin.is_empty(),
        phone_e164: cfg.phone_e164,
        admin_count: cfg.admin_e164s.split(',')
            .map(|s| s.trim()).filter(|s| !s.is_empty()).count(),
    })
}

// ─── WeChat ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct WeChatStatus {
    pub configured: bool,
    pub corp_id:    String,
    pub admin_count: usize,
}

#[tauri::command]
pub async fn paste_wechat_credentials(
    corp_id:        String,
    corp_secret:    String,
    agent_id:       String,
    admin_user_ids: String,
) -> Result<String, String> {
    let mut s = secret_store::read("wechat").unwrap_or_default();
    if !corp_id.is_empty()        { s.fields.insert("corp_id".into(),        corp_id); }
    if !corp_secret.is_empty()    { s.fields.insert("corp_secret".into(),    corp_secret); }
    if !agent_id.is_empty()       { s.fields.insert("agent_id".into(),       agent_id); }
    if !admin_user_ids.is_empty() { s.fields.insert("admin_user_ids".into(), admin_user_ids); }
    secret_store::write("wechat", &s).map_err(|e| e.to_string())?;
    Ok("wechat credentials saved".into())
}

#[tauri::command]
pub async fn wechat_status() -> Result<WeChatStatus, String> {
    use crate::integrations::wechat::WeChatConfig;
    let cfg = WeChatConfig::from_secrets_or_env();
    Ok(WeChatStatus {
        configured: !cfg.corp_id.is_empty() && !cfg.corp_secret.is_empty() && !cfg.agent_id.is_empty(),
        corp_id:    cfg.corp_id,
        admin_count: cfg.admin_user_ids.split(',')
            .map(|s| s.trim()).filter(|s| !s.is_empty()).count(),
    })
}

// ─── SSH dispatcher ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct SshStatus {
    pub listen_port:         u16,
    pub fingerprint_count:  usize,
}

#[tauri::command]
pub async fn paste_ssh_credentials(
    listen_port:        u16,
    allowed_fingerprints: String,
) -> Result<String, String> {
    let mut s = secret_store::read("ssh").unwrap_or_default();
    s.fields.insert("listen_port".into(),         listen_port.to_string());
    if !allowed_fingerprints.is_empty() {
        s.fields.insert("allowed_fingerprints".into(), allowed_fingerprints);
    }
    secret_store::write("ssh", &s).map_err(|e| e.to_string())?;
    Ok("ssh credentials saved".into())
}

#[tauri::command]
pub async fn ssh_status() -> Result<SshStatus, String> {
    use crate::integrations::ssh::SshConfig;
    let cfg = SshConfig::from_secrets_or_env();
    Ok(SshStatus {
        listen_port:        cfg.listen_port,
        fingerprint_count: cfg.allowed_fingerprints.split(',')
            .map(|s| s.trim()).filter(|s| !s.is_empty()).count(),
    })
}

// ─── REST/HTTP dispatcher ────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct RestStatus {
    pub auth_required: bool,
    pub allowed_count:  usize,
}

#[tauri::command]
pub async fn paste_rest_credentials(
    auth_required: bool,
    allowed_e164s_or_tokens: String,
) -> Result<String, String> {
    let mut s = secret_store::read("rest").unwrap_or_default();
    s.fields.insert("auth_required".into(), auth_required.to_string());
    if !allowed_e164s_or_tokens.is_empty() {
        s.fields.insert("allowed_e164s_or_tokens".into(), allowed_e164s_or_tokens);
    }
    secret_store::write("rest", &s).map_err(|e| e.to_string())?;
    Ok("rest credentials saved".into())
}

#[tauri::command]
pub async fn rest_status() -> Result<RestStatus, String> {
    use crate::integrations::rest::RestConfig;
    let cfg = RestConfig::from_secrets_or_env();
    Ok(RestStatus {
        auth_required: cfg.auth_required,
        allowed_count: cfg.allowed_e164s_or_tokens.split(',')
            .map(|s| s.trim()).filter(|s| !s.is_empty()).count(),
    })
}
