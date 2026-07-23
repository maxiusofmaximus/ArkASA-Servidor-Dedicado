//! Slack adapter using **Socket Mode** (no public HTTP URL required).
//!
//! Setup:
//!   1. https://api.slack.com/apps  → Create New App
//!   2. Socket Mode → Enable, generate App-Level Token (`xapp-...`). Tokens: `connections:write`,
//!      `authorizations:read`.
//!   3. Event Subscriptions → Enable, subscribe to `message.im` (and `message.channels` if you
//!      want the bot to listen in channels it has been invited to), `app_mention`.
//!   4. OAuth & Permissions → add scopes `chat:write`, `im:history`, `groups:history`, `channels:history`.
//!      Install the app to your workspace and copy the Bot User OAuth Token (`xoxb-...`).
//!   5. Env vars on the Tauri box:
//!        SLACK_ENABLED=true
//!        SLACK_BOT_TOKEN=xoxb-...
//!        SLACK_APP_TOKEN=xapp-...
//!        SLACK_ADMINS=U123,U456        (operator user ids, comma-separated; empty = any user)
//!        SLACK_AI_ENABLED=true|false

use crate::integrations::command_router::{
    Channel, CommandKind, RemoteCommand, RemoteCommandContext, RouterOutcome,
    gated_chat_binding, run_with_receipts, PipelineOutcome,
};
use crate::integrations::Platform as IdentityPlatform;
use crate::integrations::receipt_emit::{try_global, ReceiptContext};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub app_token: String,
    pub admins: Vec<String>,
    pub ai_enabled: bool,
}

impl Default for SlackConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("SLACK_ENABLED").map(|v| v == "true").unwrap_or(false),
            bot_token: std::env::var("SLACK_BOT_TOKEN").unwrap_or_default(),
            app_token: std::env::var("SLACK_APP_TOKEN").unwrap_or_default(),
            admins: std::env::var("SLACK_ADMINS")
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| {
                    let t = s.trim();
                    if t.is_empty() { None } else { Some(t.to_string()) }
                })
                .collect(),
            ai_enabled: std::env::var("SLACK_AI_ENABLED").map(|v| v == "true").unwrap_or(true),
        }
    }
}

impl SlackConfig {
    pub fn is_active(&self) -> bool {
        self.enabled && !self.bot_token.is_empty() && !self.app_token.is_empty()
    }

    fn is_admin(&self, user_id: &str) -> bool {
        if self.admins.is_empty() { return true; }
        self.admins.iter().any(|a| a == user_id)
    }
}

/// Slack Socket Mode payloads
#[derive(Debug, Deserialize)]
struct Envelope {
    envelope_id: String,
    #[serde(default)]
    payload: Option<serde_json::Value>,
    #[serde(default = "default_envelope_type")]
    r#type: String,
}
fn default_envelope_type() -> String { "unknown".to_string() }

#[derive(Debug, Deserialize)]
struct DisconnectPayload {
    reason: String,
}

pub struct SlackBot {
    cfg: SlackConfig,
    http: reqwest::Client,
    ai: Option<crate::integrations::ai::AiClient>,
}

impl SlackBot {
    pub fn new(cfg: SlackConfig) -> Self {
        let ai_client = crate::integrations::ai::AiClient::from_env();
        let ai = if cfg.ai_enabled && ai_client.enabled() { Some(ai_client) } else { None };
        Self {
            cfg,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build().expect("client"),
            ai,
        }
    }

    pub async fn run<F>(self, router: Arc<F>)
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String>
            + ?Sized + Send + Sync + 'static,
    {
        loop {
            log::info!("Slack bot reconnecting…");
            match self.run_once(&router).await {
                Ok(_) => log::warn!("Slack run_once exited cleanly, restarting…"),
                Err(e) => log::warn!("Slack errored: {e}. Restarting in 5s…"),
            }
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn run_once<F>(&self, router: &Arc<F>) -> Result<(), String>
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String>
            + ?Sized + Send + Sync + 'static,
    {
        // 1. Open a new WebSocket using apps.connections.open
        let resp = self.http.post("https://slack.com/api/apps.connections.open")
            .header("Authorization", format!("Bearer {}", self.cfg.app_token))
            .json(&json!({}))
            .send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("apps.connections.open HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if body["ok"] != json!(true) {
            return Err(format!("apps.connections.open: {}", body["error"]));
        }
        let wss_url = body["url"].as_str().ok_or("missing url")?.to_string();
        log::info!("Slack Socket Mode websocket URL obtained");

        // 2. Connect to the WebSocket.
        let req = tokio_tungstenite::tungstenite::handshake::client::Request::builder()
            .method("GET")
            .uri(&wss_url)
            .header("Host", "wss-primary.slack.com")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key())
            .header("Sec-WebSocket-Version", "13")
            .body(()).map_err(|e| e.to_string())?;
        let (mut ws, _) = tokio_tungstenite::connect_async(req)
            .await.map_err(|e| format!("websocket connect: {e}"))?;

        while let Some(msg) = ws.next().await {
            let msg = msg.map_err(|e| e.to_string())?;
            use tokio_tungstenite::tungstenite::protocol::Message;
            match msg {
                Message::Text(text) => {
                    let env: Envelope = match serde_json::from_str(&text) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    if env.r#type == "disconnect" {
                        if let Ok(disc) = serde_json::from_value::<DisconnectPayload>(
                            env.payload.clone().unwrap_or(json!({}))) {
                            log::warn!("Slack disconnected: {}", disc.reason);
                        }
                        return Ok(());
                    }
                    if env.r#type != "events_api" { continue; }
                    let Some(payload) = env.payload else { continue };
                    if let Ok(ack_text) = serde_json::to_string(&json!({"envelope_id": env.envelope_id})) {
                        let _ = ws.send(Message::Text(ack_text)).await;
                    }
                    let event = &payload["event"];
                    let event_type = event["type"].as_str().unwrap_or("");
                    if matches!(event_type, "message" | "app_mention") {
                        // Skip bot messages to avoid loops.
                        if event["bot_id"].is_string() { continue; }
                        let text = match event["text"].as_str() {
                            Some(t) => t.to_string(),
                            None => continue,
                        };
                        let user_id = event["user"].as_str().unwrap_or("").to_string();
                        let channel_id = event["channel"].as_str().unwrap_or("").to_string();
                        if !self.cfg.is_admin(&user_id) {
                            continue;
                        }
                        if let Some(reply) = self.handle_inbound(&text, &user_id, &env.envelope_id, router).await {
                            if let Err(e) = self.send_message(&channel_id, &reply).await {
                                log::warn!("Slack send failed: {e}");
                            }
                        }
                    }
                }
                Message::Close(_) => return Ok(()),
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_inbound<F>(
        &self,
        text: &str,
        user_id: &str,
        envelope_id: &str,
        router: &Arc<F>,
    ) -> Option<String>
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String>
            + ?Sized + Send + Sync + 'static,
    {
        let emitter = try_global();
        let channel_id = "slack"; // Best-effort: we only know user_id here. Slack multi-tenant support is room-binding based, not chat-binding based, but admins still gate by user_id.
        let envelope_id = if envelope_id.is_empty() {
            // Fall back to per-message synthetic id; never pass empty to
            // the receipts ledger so the queue-stage stays differentiable.
            format!("slack:envelope:unknown:{user_id}")
        } else { envelope_id.to_string() };
        let binding = if self.cfg.admins.is_empty() {
            crate::integrations::command_router::default_chat_binding(
                Channel::Slack, channel_id,
            )
        } else {
            gated_chat_binding(Channel::Slack, channel_id, self.cfg.admins.clone())
        };

        // Slack emits the Socket-Mode envelope ACK before reaching this
        // handler, so we can already tag the message as queued for
        // processing. Receipts are best-effort; we don't fail the
        // message if the ledger hiccups.
        let trace_id = format!("slack:{user_id}");
        if let Some(em) = emitter.as_ref() {
            em.queue_enqueued(
                ReceiptContext {
                    trace_id: &trace_id,
                    actor_id: user_id,
                    actor_name: user_id,
                    runtime: crate::integrations::RuntimeClass::Interactive,
                },
                IdentityPlatform::Slack,
                // Real envelope ID propagated from the WS frame, so the
                // audit log can correlate Slack events end-to-end.
                &envelope_id,
            );
        }

        let helper = |ctx, cmd| (router)(ctx, cmd);

        // Slash-command path
        if text.starts_with('/') {
            let parts: Vec<&str> = text.split_whitespace().collect();
            let cmd = parts[0].to_ascii_lowercase();
            let arg = if parts.len() > 1 { Some(parts[1..].join(" ")) } else { None };
            let Some(kind) = CommandKind::parse_slash(&cmd) else {
                return Some("⚠ unknown command.".into());
            };
            let kind_label = kind.as_str();
            let remote = RemoteCommand {
                kind,
                map_index: None,
                config_patch: None,
                tail: arg.as_ref().and_then(|s| s.parse().ok()),
            };
            let outcome = run_with_receipts(
                emitter.as_deref(), IdentityPlatform::Slack, Channel::Slack,
                &binding, user_id, user_id, &trace_id, text, None,
                Some(kind_label), Some(remote), &helper,
            );
            return Some(pipeline_to_reply(&outcome));
        }

        // Natural-language path: query AI, possibly extract [COMMAND: {...}].
        let Some(ai) = &self.ai else {
            return Some("🤖 AI disabled. Use slash commands.".into());
        };
        let ai_reply = match ai.query(text).await {
            Ok(r)  => r,
            Err(e) => return Some(format!("⚠ AI error: {e}")),
        };
        if let Some(cmd_json_str) = extract_command_tag(&ai_reply).map(str::to_string) {
            if let Ok(parsed) = serde_json::from_str::<ParsedAiCmd>(&cmd_json_str) {
                let cmd_kind = match parsed.kind.as_str() {
                    "start"    => Some(CommandKind::Start),
                    "stop"     => Some(CommandKind::Stop),
                    "restart"  => Some(CommandKind::Restart),
                    "status"   => Some(CommandKind::Status),
                    "logs"     => Some(CommandKind::Logs),
                    "ip"       => Some(CommandKind::Ip),
                    _ => None,
                };
                if let Some(kind) = cmd_kind {
                    let kind_label = parsed.kind.as_str();
                    let remote = RemoteCommand {
                        kind,
                        map_index: parsed.map_index,
                        config_patch: None,
                        tail: parsed.tail,
                    };
                    let outcome = run_with_receipts(
                        emitter.as_deref(), IdentityPlatform::Slack, Channel::Slack,
                        &binding, user_id, user_id, &trace_id, text, None,
                        Some(kind_label), Some(remote), &helper,
                    );
                    let friendly = ai_reply
                        .replace(&format!("[COMMAND: {}]", &cmd_json_str), "")
                        .trim()
                        .to_string();
                    return match outcome {
                        PipelineOutcome::Done(msg) => Some(format!(
                            "{}\n\n**Result:**\n{}", friendly, msg,
                        )),
                        PipelineOutcome::Rejected(msg) => Some(format!(
                            "{}\n\n⚠ {}", friendly, msg,
                        )),
                        PipelineOutcome::NoCommand(_) => Some(friendly),
                    };
                }
            }
        }
        Some(ai_reply)
    }

    async fn send_message(&self, channel: &str, text: &str) -> Result<(), String> {
        let url = "https://slack.com/api/chat.postMessage";
        let resp = self.http.post(url)
            .header("Authorization", format!("Bearer {}", self.cfg.bot_token))
            .json(&json!({"channel": channel, "text": text}))
            .send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if v["ok"] != json!(true) {
            return Err(format!("Slack API: {}", v["error"]));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct ParsedAiCmd {
    kind: String,
    map_index: Option<u32>,
    tail: Option<u32>,
}

fn pipeline_to_reply(outcome: &PipelineOutcome) -> String {
    match outcome {
        PipelineOutcome::Done(msg)     => msg.clone(),
        PipelineOutcome::Rejected(msg) => msg.clone(),
        PipelineOutcome::NoCommand(s)  => s.clone(),
    }
}

fn extract_command_tag(text: &str) -> Option<&str> {
    let start = text.find("[COMMAND:")?;
    let after = start + "[COMMAND:".len();
    let rest = &text[after..];
    let end = rest.find(']')?;
    Some(rest[..end].trim())
}

pub async fn spawn_looper<F>(bot: SlackBot, router: Arc<F>)
where
    F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String>
        + ?Sized + Send + Sync + 'static,
{
    tokio::spawn(async move {
        bot.run(router).await;
    }).await.ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slack_admin_allowlist() {
        let mut cfg = SlackConfig::default();
        cfg.admins = vec!["U123".into()];
        assert!(cfg.is_admin("U123"));
        assert!(!cfg.is_admin("U999"));
        cfg.admins.clear();
        assert!(cfg.is_admin("anyone"));
    }

    #[test]
    fn extract_command_tag_simple() {
        let s = "Sure [COMMAND: {\"kind\": \"status\"}]";
        assert_eq!(extract_command_tag(s).unwrap(), "{\"kind\": \"status\"}");
    }
}
