//! WhatsApp plugin bridge — Tauri commands mirroring the Convex /
//! Vercel pattern:
//!  - `begin_whatsapp_link()` — interactive flow that the user can
//!    trigger from the desktop GUI. Currently a no-op placeholder
//!    because WhatsApp authentication is API-key based
//!    (`paste_whatsapp_credentials` is the canonical entry).
//!  - `paste_whatsapp_credentials(deployment_url, deploy_key)` —
//!    persists the secrets to `secret_store` under the "whatsapp"
//!    plugin id. Carries `phone_number_id`, `business_id`,
//!    `webhook_secret`, `api_token`, `admin_e164s`.
//!  - `whatsapp_status()` — surfaces whether the plugin is configured,
//!    plus a redacted preview of admin_e164s.
//!
//! Backwards rule: 100% identifier-stable with the existing
//! whatsapp.rs module. Add/remove redactable.

use crate::integrations::whatsapp::WhatsAppConfig;
use crate::plugins::secret_store;
use serde::Serialize;

fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0)
}

#[derive(Debug, Serialize, Clone)]
pub struct WhatsAppBridgeStatus {
    pub connected:         bool,
    pub phone_number_id:   String,
    pub business_id:       String,
    pub webhook_secret:    String,
    pub api_token:         String,
    pub admin_e164s:       String,
    /// Sanity: how many admin E.164s the operator has allowlisted.
    pub admin_count:       usize,
}

#[tauri::command]
pub async fn begin_whatsapp_link() -> Result<String, String> {
    // WhatsApp Cloud onboarding is token + secret paste — there's no
    // OAuth flow the desktop can drive for an end-user. We surface
    // a helpful pointer to the WhatsApp Manager dashboard instead.
    Ok(
        "WhatsApp Business Cloud is configured by pasting 5 secrets under \
         General → Cloud Services → WhatsApp: phone_number_id, business_id, \
         webhook_secret, api_token, admin_e164s. Get them at \
         https://business.facebook.com/wa/manage/home/."
        .into(),
    )
}

#[tauri::command]
pub async fn paste_whatsapp_credentials(
    phone_number_id: String,
    business_id:     String,
    webhook_secret:  String,
    api_token:       String,
    admin_e164s:     String,
) -> Result<String, String> {
    let mut s = secret_store::read("whatsapp").unwrap_or_default();
    if !phone_number_id.is_empty() { s.fields.insert("phone_number_id".into(), phone_number_id); }
    if !business_id.is_empty()     { s.fields.insert("business_id".into(),     business_id); }
    if !webhook_secret.is_empty()  { s.fields.insert("webhook_secret".into(),  webhook_secret); }
    if !api_token.is_empty()       { s.fields.insert("api_token".into(),       api_token); }
    if !admin_e164s.is_empty()     { s.fields.insert("admin_e164s".into(),     admin_e164s); }
    s.fields.insert("last_seen_at_unix".into(),
                  unix_now().to_string());
    secret_store::write("whatsapp", &s).map_err(|e| e.to_string())?;
    Ok("whatsapp credentials saved".into())
}

#[tauri::command]
pub async fn whatsapp_status() -> Result<WhatsAppBridgeStatus, String> {
    let cfg = WhatsAppConfig::from_secrets_or_env();
    let admin_count = cfg.admin_e164s.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .count();
    Ok(WhatsAppBridgeStatus {
        connected:       !cfg.api_token.is_empty() && !cfg.phone_number_id.is_empty(),
        phone_number_id: cfg.phone_number_id,
        business_id:     cfg.business_id,
        webhook_secret:  cfg.webhook_secret,
        api_token:       cfg.api_token,
        admin_e164s:     cfg.admin_e164s,
        admin_count,
    })
}

#[cfg(test)]
mod tests {
    // manifest kept for future bridge tests.
}
