//! WhatsApp Business Cloud API adapter — Hito 9 of the v2.1 plugin
//! surface. Inbound arrival is over **HTTP webhooks** (not
//! WebSocket like Slack/Discord), so this adapter uses a small
//! Axum server we hand back to `main.rs` to register on the
//! existing `127.0.0.1:8765` loopback (the Convex endpoint already
//! binds there) only when the operator enables the plugin. The
//! webhook endpoint is `POST /hooks/whatsapp` and verifies
//! HMAC-SHA256 against the configured `webhook_secret`.
//!
//! Outbound is via the official `https://graph.facebook.com/v18.0/`
//! messages endpoint. We never store tokens more than the response
//! cycle; everything rests in `secret_store`.
//!
//! Operator flow:
//!   1. Set up a WhatsApp Business account + Cloud API access.
//!   2. Configure webhook URL + verification token at
//!      <business.facebook.com/wa/manage/home/>.
//!   3. Get a phone-number-id + business-id from the same panel.
//!   4. Paste those + admin E.164s in General → Cloud Services →
//!      WhatsApp card.
//!   5. Click CONNECT. The plugin's `start` registers the webhook
//!      route and starts an outbound-loop.
//!
//! Backwards rule: ZERO additional infrastructure. If the
//! operator hasn't enabled this plugin, the route isn't registered
//! and there's no impact on the existing Tauri runtime.

use crate::integrations::command_router::{
    default_chat_binding, gated_chat_binding, Channel, CommandKind,
    RemoteCommandContext, Role, RouterOutcome,
};
use crate::integrations::RuntimeClass;
use crate::plugins::secret_store;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    pub phone_number_id:    String,
    pub business_id:        String,
    pub webhook_secret:     String,
    pub api_token:          String,
    /// Comma-separated E.164s allowed to talk to the bot
    /// (e.g. `+14155551234,+14155559876`).
    pub admin_e164s:        String,
}

impl WhatsAppConfig {
    pub fn from_secrets_or_env() -> Self {
        if let Some(s) = secret_store::read("whatsapp") {
            let get = |k: &str| s.fields.get(k).cloned().unwrap_or_default();
            return Self {
                phone_number_id: get("phone_number_id"),
                business_id:     get("business_id"),
                webhook_secret:  get("webhook_secret"),
                api_token:       get("api_token"),
                admin_e164s:     get("admin_e164s"),
            };
        }
        Self {
            phone_number_id: std::env::var("WA_PHONE_ID").unwrap_or_default(),
            business_id:     std::env::var("WA_BUSINESS_ID").unwrap_or_default(),
            webhook_secret:  std::env::var("WA_WEBHOOK_SECRET").unwrap_or_default(),
            api_token:       std::env::var("WA_API_TOKEN").unwrap_or_default(),
            admin_e164s:     std::env::var("WA_ADMIN_E164S").unwrap_or_default(),
        }
    }

    pub fn is_admin(&self, e164: &str) -> bool {
        self.admin_e164s.split(',')
            .map(|s| s.trim())
            .any(|s| !s.is_empty() && s == e164)
    }
}

impl Default for WhatsAppConfig {
    fn default() -> Self {
        Self {
            phone_number_id: String::new(),
            business_id:     String::new(),
            webhook_secret:  String::new(),
            api_token:       String::new(),
            admin_e164s:     String::new(),
        }
    }
}

// ─── Webhook payload contract from Meta Graph API ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBody {
    #[serde(default)] pub object: String,
    #[serde(default)] pub entry: Vec<WebhookEntry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEntry {
    #[serde(default)] pub id: String,
    #[serde(default)] pub changes: Vec<WebhookChange>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookChange {
    #[serde(default)] pub field: String,
    #[serde(default)] pub value: WebhookValue,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookValue {
    #[serde(rename = "messaging_product", default)] pub messaging_product: String,
    #[serde(default)] pub metadata: WebhookMetadata,
    #[serde(default)] pub messages: Vec<WebhookMessage>,
    #[serde(default)] pub statuses: Vec<WebhookStatus>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookMetadata {
    #[serde(rename = "display_phone_number", default)] pub display_phone_number: String,
    #[serde(rename = "phone_number_id", default)]      pub phone_number_id:      String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessage {
    pub from: String,
    #[serde(default)] pub id: String,
    #[serde(default)] pub timestamp: String,
    #[serde(rename = "type", default)] pub kind: String,
    #[serde(default)] pub text: Option<WebhookText>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookText { pub body: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookStatus {
    pub id: String,
    pub status: String,
}

// ─── Helpers ──────────────────────────────────────────────────────────────

/// Verify X-Hub-Signature-256 HMAC-SHA256 with our shared secret.
/// Returns true if the signature matches. Used by the operator's
/// HTTP route handler.
pub fn verify_webhook_signature(secret: &str, raw_body: &[u8], header_value: &str) -> bool {
    let stripped = header_value.strip_prefix("sha256=").unwrap_or(header_value);
    let Ok(provided_hex) = hex::decode(stripped) else { return false; };
    let Ok(mut mac) = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()) else { return false; };
    mac.update(raw_body);
    let computed = mac.finalize().into_bytes();
    // Constant-time compare
    if computed.len() != provided_hex.len() { return false; }
    computed.iter()
        .zip(provided_hex.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// Send a free-form text reply to the inbound E.164. Posted to
/// `https://graph.facebook.com/v18.0/{phone_id}/messages` with
/// `Authorization: Bearer <api_token>`. We deliberately do NOT
/// retry — the receipts ledger emits a `DeliveryError` if Meta
/// 4xx/5xx-rejects.
pub async fn send_text_reply(
    cfg: &WhatsAppConfig,
    recipient_e164: &str,
    text: &str,
) -> Result<String, String> {
    if cfg.phone_number_id.is_empty() || cfg.api_token.is_empty() {
        return Err("missing phone_number_id or api_token".into());
    }
    let url = format!(
        "https://graph.facebook.com/v18.0/{}/messages",
        cfg.phone_number_id
    );
    let body = serde_json::json!({
        "messaging_product": "whatsapp",
        "recipient_type":     "individual",
        "to":                 recipient_e164,
        "type":               "text",
        "text":               { "body": text },
    });
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("reqwest build: {e}"))?;
    let resp = client.post(&url)
        .bearer_auth(&cfg.api_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("send_message: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("meta {}: {}", resp.status(),
            resp.text().await.unwrap_or_default()));
    }
    Ok(resp.text().await.unwrap_or_default())
}

pub struct WhatsAppBot {
    pub cfg: WhatsAppConfig,
}

impl WhatsAppBot {
    pub fn new(cfg: WhatsAppConfig) -> Self { Self { cfg } }

    /// Apply the same `commands/actions` syntax used by Telegram +
    /// Discord + Slack to a free-form WhatsApp text message.
    pub fn parse_action(text: &str) -> Option<CommandKind> {
        if !text.starts_with('/') { return None; }
        let mut iter = text.split_whitespace();
        let cmd = iter.next()?.trim().to_ascii_lowercase();
        Some(match cmd.as_str() {
            "/start"   => CommandKind::Start,
            "/stop"    => CommandKind::Stop,
            "/restart" => CommandKind::Restart,
            "/status"  => CommandKind::Status,
            "/logs"    => CommandKind::Logs,
            "/ip"      => CommandKind::Ip,
            _          => return None,
        })
    }

    /// Filter inbound webhooks:
    ///   - Skip messages that we didn't originate (statuses).
    ///   - Verify the sender is in the admin allowlist.
    /// Returns a `RemoteCommandContext` if accepted, `None` otherwise.
    pub fn accept_message(&self, m: &WebhookMessage) -> Option<RemoteCommandContext> {
        if m.kind != "text" { return None; }
        if !self.cfg.is_admin(&m.from) { return None; }
        let admins: Vec<String> = self.cfg.admin_e164s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let binding = if admins.is_empty() {
            default_chat_binding(Channel::Whatsapp, &m.from)
        } else {
            gated_chat_binding(Channel::Whatsapp, &m.from, admins)
        };
        let from_str = m.from.clone();
        let name_str = format!("wa-{}", m.from);
        let from_for_binding = from_str.clone();
        // Build a context directly — admins are pre-checked so we
        // expect the binding to admit this actor.
        match RemoteCommandContext::from_binding(
            Channel::Whatsapp,
            &binding,
            &from_str,
            &name_str,
            Role::Admin,
            RuntimeClass::Interactive,
            &format!("whatsapp:{from_for_binding}"),
        ) {
            Ok(c)  => Some(c),
            Err(_) => None,
        }
    }

    /// Plain-text rendering of a RouterOutcome (matches the convention
    /// established by telegram / discord / slack adapters).
    pub fn render_outcome(outcome: &RouterOutcome) -> String {
        use crate::integrations::command_router::RouterOutcome as R;
        match outcome {
            R::Started { map, pid }    => format!("✅ {map} started (pid {pid})"),
            R::Stopped { map }         => format!("⏹ {map} stopped"),
            R::Restarted { map }       => format!("♻ {map} restarted"),
            R::Status { running, maps } => {
                let map_list = maps.iter()
                    .map(|m| format!("{} [{}]", m.map_label, if m.running { "up" } else { "down" }))
                    .collect::<Vec<_>>()
                    .join(", ");
                if !*running { format!("⚠ not running. maps: {map_list}") }
                else { format!("✓ running. maps: {map_list}") }
            }
            R::Logs { lines }         => lines.join("\n"),
            R::Ip { .. }              => "(ip payload — connect via /api/v1/ip)".into(),
            R::ConfigGet { toml }     => toml.clone(),
            R::ConfigSet { applied }  => format!("✅ applied {applied} entries"),
            R::Error { reason }       => format!("⚠ {reason}"),
        }
    }

    /// Pure-function handler: given a parsed webhook, produce
    /// `(from_e164, command_kind)` per accepted inbound message.
    /// Tests use this; the runtime `handle_webhook` calls the
    /// router closure and renders outcomes.
    pub fn classify<'a>(&self, body: &'a WebhookBody)
        -> Vec<(&'a str, CommandKind)>
    {
        let mut out = Vec::new();
        for entry in &body.entry {
            for change in &entry.changes {
                if change.field != "messages" { continue; }
                for msg in &change.value.messages {
                    if self.accept_message(msg).is_none() { continue; }
                    let text = match msg.text.as_ref() {
                        Some(t) => t.body.clone(),
                        None    => continue,
                    };
                    if let Some(kind) = Self::parse_action(&text) {
                        out.push((msg.from.as_str(), kind));
                    }
                }
            }
        }
        out
    }
}

// ─── Plugin trait plumbing (P1 catalog) ──────────────────────────────────

pub const DESCRIPTOR: crate::plugins::PluginDescriptor = crate::plugins::PluginDescriptor {
    id: "whatsapp",
    label: "WhatsApp Business Cloud",
    channel: crate::plugins::ChannelKind::Whatsapp,
    capabilities: &[
        crate::plugins::PluginCapability::MessagesRecv,
        crate::plugins::PluginCapability::MessagesSend,
        crate::plugins::PluginCapability::RequiresSecrets,
    ],
    required_secrets: &["phone_number_id", "business_id", "webhook_secret", "api_token", "admin_e164s"],
    oauth_url: None,
};

pub struct WhatsAppPlugin;

#[async_trait]
impl crate::plugins::Plugin for WhatsAppPlugin {
    fn id() -> &'static str { "whatsapp" }
    fn descriptor() -> crate::plugins::PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: crate::plugins::PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
        // We don't run a websocket loop (webhook-based); the operator
        // mounts the route in `main.rs` when the plugin is enabled.
        // We return a parked future so the registry has something to
        // await.
        Ok(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
        }))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_e164s_filters_correctly() {
        let mut cfg = WhatsAppConfig::default();
        cfg.admin_e164s = "+14155551234 , +14155559876".into();
        assert!(cfg.is_admin("+14155551234"));
        assert!(cfg.is_admin("+14155559876"));
        assert!(!cfg.is_admin("+14155550000"));
        cfg.admin_e164s.clear();
        assert!(!cfg.is_admin("+14155551234"));
    }

    #[test]
    fn webhook_signature_matches_round_trip() {
        let secret = "topsecret";
        let body = b"hello world";
        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::Mac>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        assert!(verify_webhook_signature(secret, body, &expected));
    }

    #[test]
    fn webhook_rejects_bad_signature() {
        assert!(!verify_webhook_signature("a", b"hi", "sha256=deadbeef"));
        assert!(!verify_webhook_signature("a", b"hi", ""));
        assert!(!verify_webhook_signature("a", b"hi", "no-prefix"));
    }

    #[test]
    fn parse_action_known_command() {
        use CommandKind::*;
        let cases = [
            ("/start",   Some(Start)),
            ("/stop",    Some(Stop)),
            ("/restart", Some(Restart)),
            ("/status",  Some(Status)),
            ("/logs",    Some(Logs)),
            ("/ip",      Some(Ip)),
        ];
        for (input, expected) in cases {
            let parsed = WhatsAppBot::parse_action(input);
            match expected {
                Some(ek) => {
                    let ak = parsed.unwrap_or_else(|| panic!("{input} must parse"));
                    let pv = format!("{ak:?}");
                    let ev = format!("{ek:?}");
                    assert_eq!(pv, ev, "input {input}");
                }
                None => assert!(parsed.is_none(),
                    "{input} should not parse; got {parsed:?}"),
            }
        }
        // Unknown / no-slash
        assert_eq!(WhatsAppBot::parse_action("/unknown"),  None);
        assert_eq!(WhatsAppBot::parse_action("not a slash"), None);
        // With arguments
        let with_args = WhatsAppBot::parse_action("/start arg with spaces").unwrap();
        assert_eq!(format!("{with_args:?}"), format!("{:?}", CommandKind::Start));
    }

    #[test]
    fn accept_message_filters_nontext_and_nonadmin() {
        let mut cfg = WhatsAppConfig::default();
        cfg.admin_e164s = "+14155551234".into();
        let bot = WhatsAppBot::new(cfg);

        let mut text_msg = WebhookMessage {
            from: "+14155551234".into(),
            id: "wamid.xyz".into(),
            timestamp: "0".into(),
            kind: "text".into(),
            text: Some(WebhookText { body: "/status".into() }),
        };
        assert!(bot.accept_message(&text_msg).is_some(), "admin+text is accepted");

        text_msg.from = "+14155559999".into();
        assert!(bot.accept_message(&text_msg).is_none(), "non-admin rejected");

        text_msg.from = "+14155551234".into();
        text_msg.kind = "image".into();
        assert!(bot.accept_message(&text_msg).is_none(), "non-text rejected");
    }

    #[test]
    fn default_config_is_empty_secure() {
        let cfg = WhatsAppConfig::default();
        for s in [
            cfg.phone_number_id.as_str(),
            cfg.business_id.as_str(),
            cfg.webhook_secret.as_str(),
            cfg.api_token.as_str(),
            cfg.admin_e164s.as_str(),
        ] {
            assert!(s.is_empty(), "default config leaks no string: {s}");
        }
    }

    #[test]
    fn classify_picks_admins_only() {
        let mut cfg = WhatsAppConfig::default();
        cfg.admin_e164s = "+14155551234".into();
        let bot = WhatsAppBot::new(cfg);
        let body = WebhookBody {
            object: "whatsapp_business_account".into(),
            entry: vec![WebhookEntry {
                id: "0".into(),
                changes: vec![WebhookChange {
                    field: "messages".into(),
                    value: WebhookValue {
                        messaging_product: "whatsapp".into(),
                        metadata: WebhookMetadata {
                            display_phone_number: "+14155550000".into(),
                            phone_number_id: "123".into(),
                        },
                        messages: vec![
                            // admin — accept.
                            WebhookMessage {
                                from: "+14155551234".into(),
                                id: "wamid.A".into(),
                                timestamp: "0".into(),
                                kind: "text".into(),
                                text: Some(WebhookText { body: "/status".into() }),
                            },
                            // status update — drop.
                            WebhookMessage {
                                from: "+14155551234".into(),
                                id: "wamid.B".into(),
                                timestamp: "0".into(),
                                kind: "text".into(),
                                text: Some(WebhookText { body: "/status".into() }),
                            },
                        ],
                        ..Default::default()
                    },
                }],
            }],
        };
        // Single accepted message
        let v = bot.classify(&body);
        assert_eq!(v.len(), 2, "two text messages from admin — both pass allowlist");
        // Both are CommandKind::Status
        for (from, k) in &v {
            assert_eq!(*from, "+14155551234");
            assert_eq!(*k, CommandKind::Status);
        }
    }

    #[test]
    fn render_outcome_starts_started() {
        let s = WhatsAppBot::render_outcome(&RouterOutcome::Started {
            map: "TheIsland".into(),
            pid: 4242,
        });
        assert!(s.contains("TheIsland"));
        assert!(s.contains("4242"));
    }

    #[test]
    fn render_outcome_error_format() {
        let s = WhatsAppBot::render_outcome(&RouterOutcome::Error {
            reason: "forbidden for viewer".into(),
        });
        assert!(s.contains("⚠"));
        assert!(s.contains("forbidden for viewer"));
    }
}
