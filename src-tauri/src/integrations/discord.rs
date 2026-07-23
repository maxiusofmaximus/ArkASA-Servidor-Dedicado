//! Discord adapter — real WebSocket gateway client.
//!
//! Connects to `wss://gateway.discord.gg/?v=10&encoding=json` with bot token +
//! appropriate intents. Re-uses the existing bridge so all channels share the
//! exact same `RemoteCommand` pipeline and audit log.
//!
//! Setup (60 seconds):
//!   1. https://discord.com/developers/applications → New Application → Bot
//!   2. Reset Token, copy it.
//!   3. Privileged Intents → enable MESSAGE CONTENT (so the bot can read messages).
//!   4. Invite the bot to your server (OAuth2 → URL Generator → `bot` scope +
//!      `Send Messages`, plus your guild id in the install link).
//!   5. Copy CHANNEL_ID for the text channel the bot listens on (and the
//!      USER_IDs of the operators allowed to issue commands).
//!
//! Env vars:
//!   DISCORD_ENABLED=true
//!   DISCORD_BOT_TOKEN=...
//!   DISCORD_ADMINS=123456789012345678,234567890123456789   # comma-separated user ids
//!   DISCORD_CHANNEL=123456789012345678                       # channel this bot controls
//!   DISCORD_AI_ENABLED=true|false                            # if false, only slash commands accepted

use crate::integrations::command_router::{
    Channel, CommandKind, RemoteCommand, RemoteCommandContext, RouterOutcome,
    gated_chat_binding, run_with_receipts, PipelineOutcome,
};
use crate::integrations::Platform as IdentityPlatform;
use crate::integrations::receipt_emit::try_global;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub token: String,
    pub admins: Vec<String>,
    pub channel: String,
    pub ai_enabled: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("DISCORD_ENABLED").map(|v| v == "true").unwrap_or(false),
            token: std::env::var("DISCORD_BOT_TOKEN").unwrap_or_default(),
            admins: std::env::var("DISCORD_ADMINS")
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| {
                    let t = s.trim();
                    if t.is_empty() { None } else { Some(t.to_string()) }
                })
                .collect(),
            channel: std::env::var("DISCORD_CHANNEL").unwrap_or_default(),
            ai_enabled: std::env::var("DISCORD_AI_ENABLED").map(|v| v == "true").unwrap_or(true),
        }
    }
}

impl DiscordConfig {
    pub fn is_active(&self) -> bool {
        self.enabled && !self.token.is_empty() && !self.channel.is_empty()
    }

    fn is_admin(&self, user_id: &str) -> bool {
        // If admins is empty accept everyone in the configured channel (good for solo).
        if self.admins.is_empty() { return true; }
        self.admins.iter().any(|a| a == user_id)
    }
}

/// Op-codes we recognise — these are the integers Discord emits on the
/// Gateway. Matched numerically (1, 7, 9, etc.) so the docs/cargo don't
/// warn on every constant. See Discord spec:
/// https://discord.com/developers/docs/topics/opcodes-and-status-codes
///
/// The Deserialize structs below carry `#[allow(dead_code)]` on fields that
/// Discord includes in the gateway frame but we don't act on (we only need
/// `MESSAGE_CREATE` for the bridging logic and `READY` to confirm session
/// start). Keeping the fields parsed (rather than skipping them at the
/// serde layer) makes it trivial to wire a future "log raw frames for
/// debug" feature or to start consuming `guild_id` for filtering without
/// a binding refactor. P3.2 audit (IMPLEMENTATION_PLAN.md §7.2.2): kept
/// with rationale.

/// Minimal dispatch payload — we only care about MESSAGE_CREATE and READY.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Dispatch {
    #[serde(default)]
    t: Option<String>,
    #[serde(default)]
    s: Option<u64>,
    #[serde(default)]
    d: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Hello {
    d: HelloData,
}

#[derive(Debug, Deserialize)]
struct HelloData {
    heartbeat_interval: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Ready {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    resume_gateway_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageCreate {
    #[allow(dead_code)]
    id: String,
    #[serde(default)]
    content: Option<String>,
    author: Author,
    channel_id: String,
    #[allow(dead_code)]
    guild_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Author {
    id: String,
    #[serde(default)]
    username: Option<String>,
}

pub struct DiscordBot {
    cfg: DiscordConfig,
    http: reqwest::Client,
    ai: Option<crate::integrations::ai::AiClient>,
}

impl DiscordBot {
    pub fn new(cfg: DiscordConfig) -> Self {
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

    async fn send_message(&self, channel_id: &str, text: &str) -> Result<(), String> {
        let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
        let body = serde_json::json!({ "content": text });
        let resp = self.http.post(&url)
            .header("Authorization", format!("Bot {}", self.cfg.token))
            .json(&body)
            .send().await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            return Err(format!("Discord send failed {}: {}", status, txt));
        }
        Ok(())
    }

    /// Run the gateway client until the future is cancelled.
    /// `router` accepts a `RemoteCommand` and returns a user-facing `RouterOutcome`.
    pub async fn run<F>(self, router: Arc<F>)
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String>
            + ?Sized + Send + Sync + 'static,
    {
        log::info!("Discord gateway client starting…");
        // 1. Get the WSS gateway URL (cached for the lifetime of the bot).
        let wss_url = match self.fetch_gateway_url().await {
            Ok(u) => u,
            Err(e) => {
                log::error!("Discord: failed to fetch gateway URL: {e}");
                return;
            }
        };
        // 2. Auto-reconnect loop across transient socket failures.
        loop {
            if let Err(e) = self.run_once(&wss_url, &router).await {
                log::warn!("Discord disconnected: {e}. Reconnecting in 5s…");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    async fn fetch_gateway_url(&self) -> Result<String, String> {
        // Prefer /gateway/bot because it requires the token and lets us check auth.
        let resp = self.http.get("https://discord.com/api/v10/gateway/bot")
            .header("Authorization", format!("Bot {}", self.cfg.token))
            .send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("gateway/bot {}: {}", resp.status(),
                resp.text().await.unwrap_or_default()));
        }
        let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(v["url"].as_str().unwrap_or("wss://gateway.discord.gg/?v=10&encoding=json")
            .to_string())
    }

    async fn run_once<F>(&self, wss_url: &str, router: &Arc<F>) -> Result<(), String>
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + ?Sized + Send + Sync + 'static,
    {
        use tokio_tungstenite::tungstenite::handshake::client::Request;
        let url = if wss_url.contains('?') {
            format!("{}&v=10&encoding=json", wss_url)
        } else {
            format!("{}?v=10&encoding=json", wss_url)
        };
        let req = Request::builder()
            .method("GET")
            .uri(&url)
            .header("Host", "gateway.discord.gg")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
            .header("Sec-WebSocket-Version", "13")
            .body(())
            .map_err(|e| e.to_string())?;
        let (mut ws, _resp) = tokio_tungstenite::connect_async(req)
            .await
            .map_err(|e| format!("websocket connect: {e}"))?;
        log::info!("Discord gateway WebSocket connected.");

        // 2. Expect a Hello (op 10) with heartbeat_interval.
        let heartbeat_ms = match read_dispatch(&mut ws).await? {
            Dispatch { t, d, .. } if t.as_deref() == Some("HELLO") => {
                let h: HelloData = serde_json::from_value(d.ok_or("HELLO d is None")?)
                    .map_err(|e| e.to_string())?;
                h.heartbeat_interval
            }
            _ => return Err("expected HELLO first".into()),
        };
        log::info!("Discord heartbeat interval = {} ms", heartbeat_ms);

        // 3. Identify (op 2) with required intents.
        // GUILD_MESSAGES (1<<9) + DIRECT_MESSAGES (1<<12) + MESSAGE_CONTENT (1<<15) = 513 + 32768 = 33281
        let intents: u64 = (1 << 9) | (1 << 12) | (1 << 15);
        let identify = serde_json::json!({
            "op": 2, // IDENTIFY
            "d": {
                "token": self.cfg.token,
                "intents": intents,
                "properties": { "os": "windows", "browser": "ark-asa", "device": "ark-asa" }
            }
        });
        ws.send(Message::Text(identify.to_string())).await.map_err(|e| e.to_string())?;

        let mut last_seq: Option<u64> = None;
        let mut session_id: Option<String> = None;
        // `resume_gateway_url` is captured by the `"READY"` arm above (logged) —
        // we clear the binding here so we don't trigger dead-code warnings
        // while keeping the code path obvious for the next session's resume.

        // Heartbeating inline — keeps the WS stream owned in one place.
        let mut hb_interval = tokio::time::interval(Duration::from_millis(heartbeat_ms));
        // Drop the first immediate tick so we don't heartbeat before any
        // dispatch has been observed.
        hb_interval.tick().await;

        while let Some(msg) = ws.next().await {
            let msg = msg.map_err(|e| e.to_string())?;
            match msg {
                Message::Text(text) => {
                    let parsed: serde_json::Value = serde_json::from_str(&text)
                        .map_err(|e| e.to_string())?;
                    let op = parsed["op"].as_u64().unwrap_or(0);
                    match op {
                        11 => {
                            // HEARTBEAT_ACK
                        }
                        1 => {
                            // HEARTBEAT — server requested immediate heartbeat.
                            let hb = serde_json::json!({
                                "op": 1,
                                "d": last_seq,
                            });
                            ws.send(Message::Text(hb.to_string())).await.map_err(|e| e.to_string())?;
                        }
                        0 => {
                            // DISPATCH
                            let t = parsed["t"].as_str().unwrap_or("").to_string();
                            let s_val = parsed["s"].as_u64();
                            if let Some(s) = s_val { last_seq = Some(s); }
                            match t.as_str() {
                                "READY" => {
                                    let d = &parsed["d"];
                                    if let Ok(id) = serde_json::from_value::<String>(d["session_id"].clone()) {
                                        session_id = Some(id);
                                    }
                                    // Discord exposes a per-session resume
                                    // gateway; we don't yet implement op 6
                                    // RESUME, so we just log the URL for
                                    // future debugging / supervisor tuning.
                                    if let Ok(r) = serde_json::from_value::<String>(d["resume_gateway_url"].clone()) {
                                        log::info!("Discord READY: resume_gateway_url = {r}");
                                    }
                                    log::info!("Discord READY: session_id = {:?}", session_id);
                                }
                                "MESSAGE_CREATE" => {
                                    let raw = parsed["d"].clone();
                                    if let Ok(mc) = serde_json::from_value::<MessageCreate>(raw) {
                                        if mc.channel_id == self.cfg.channel
                                            && self.cfg.is_admin(&mc.author.id)
                                        {
                                            let content = mc.content.clone().unwrap_or_default();
                                            let display = mc.author.username.clone()
                                                .unwrap_or_else(|| mc.author.id.clone());
                                            if let Some(reply) = self
                                                .handle_inbound(&content, &display, &mc.author.id, router)
                                                .await
                                            {
                                                if let Err(e) = self.send_message(&mc.channel_id, &reply).await {
                                                    log::warn!("Discord send_message failed: {e}");
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        7 => {
                            // RECONNECT
                            log::warn!("Discord asked for reconnect.");
                            return Ok(());
                        }
                        9 => {
                            // INVALID_SESSION
                            let resumable = parsed["d"].as_bool().unwrap_or(false);
                            log::warn!("Discord INVALID_SESSION, resumable={resumable}");
                            if !resumable {
                                return Ok(());
                            }
                        }
                        _ => {}
                    }
                }
                Message::Close(frame) => {
                    return Err(format!("Discord closed: {:?}", frame));
                }
                _ => {}
            }
            // Heartbeat tick is sent inline to avoid shared-state complexity.
            hb_interval.tick().await;
            let hb = serde_json::json!({"op": 1, "d": last_seq });
            let _ = ws.send(Message::Text(hb.to_string())).await;
        }
        Ok(())
    }

    async fn handle_inbound<F>(
        &self,
        text: &str,
        display_name: &str,
        user_id: &str,
        router: &Arc<F>,
    ) -> Option<String>
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + ?Sized + Send + Sync + 'static,
    {
        let emitter = try_global();
        let binding = if self.cfg.admins.is_empty() {
            crate::integrations::command_router::default_chat_binding(
                Channel::Discord, &self.cfg.channel,
            )
        } else {
            gated_chat_binding(Channel::Discord, &self.cfg.channel, self.cfg.admins.clone())
        };
        let trace_id = format!("discord:{user_id}");
        let helper = |ctx, cmd| (router)(ctx, cmd);

        // Slash command path
        if text.starts_with('/') {
            let parts: Vec<&str> = text.split_whitespace().collect();
            let cmd = parts[0].to_ascii_lowercase();
            let arg = if parts.len() > 1 { Some(parts[1..].join(" ")) } else { None };
            let Some(kind) = CommandKind::parse_slash(&cmd) else {
                return Some("⚠ unknown command. Try /start /stop /restart /status /logs /ip".into());
            };
            let kind_label = kind.as_str();
            let remote = RemoteCommand {
                kind,
                map_index: None,
                config_patch: None,
                tail: arg.as_ref().and_then(|s| s.parse().ok()),
            };
            let outcome = run_with_receipts(
                emitter.as_deref(), IdentityPlatform::Discord, Channel::Discord,
                &binding, user_id, display_name, &trace_id, text, None,
                Some(kind_label), Some(remote), &helper,
            );
            return Some(pipeline_to_reply(&outcome));
        }

        // Natural language routed to AI.
        let Some(ai) = &self.ai else {
            return Some("🤖 AI is disabled for this server. Use slash commands.".into());
        };
        let ai_reply = match ai.query(text).await {
            Ok(r)  => r,
            Err(e) => return Some(format!("⚠ AI service error: {e}")),
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
                        emitter.as_deref(), IdentityPlatform::Discord, Channel::Discord,
                        &binding, user_id, display_name, &trace_id, text, None,
                        Some(kind_label), Some(remote), &helper,
                    );
                    let friendly = ai_reply
                        .replace(&format!("[COMMAND: {}]", &cmd_json_str), "")
                        .trim()
                        .to_string();
                    return match outcome {
                        PipelineOutcome::Done(msg) => Some(format!(
                            "{}\n\n**Ejecutado:**\n{}", friendly, msg,
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
}

/// Local copy of the pipeline-output → user reply bridge so we don't pull
/// telegram's helper into this module.
fn pipeline_to_reply(outcome: &PipelineOutcome) -> String {
    match outcome {
        PipelineOutcome::Done(msg)     => msg.clone(),
        PipelineOutcome::Rejected(msg) => msg.clone(),
        PipelineOutcome::NoCommand(s)  => s.clone(),
    }
}

#[derive(Debug, Deserialize)]
struct ParsedAiCmd {
    kind: String,
    map_index: Option<u32>,
    tail: Option<u32>,
}

fn extract_command_tag(text: &str) -> Option<&str> {
    let start = text.find("[COMMAND:")?;
    let after = start + "[COMMAND:".len();
    let rest = &text[after..];
    let end = rest.find(']')?;
    Some(rest[..end].trim())
}

async fn read_dispatch<S>(ws: &mut S) -> Result<Dispatch, String>
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    let msg = ws.next().await.ok_or("stream closed before HELLO")?;
    let msg = msg.map_err(|e| e.to_string())?;
    match msg {
        Message::Text(text) => serde_json::from_str::<Dispatch>(&text).map_err(|e| e.to_string()),
        other => Err(format!("unexpected frame: {:?}", other)),
    }
}

use tokio_tungstenite::tungstenite::protocol::Message;

/// Spawn Discord bot on the current thread. Returns the JoinHandle so the
/// caller can cancel it during shutdown.
pub async fn spawn_looper<F>(bot: DiscordBot, router: Arc<F>)
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
    fn parses_command_tag_basic() {
        let sample = "Sure [COMMAND: {\"kind\": \"start\", \"map_index\": 0}]";
        assert_eq!(extract_command_tag(sample).unwrap(),
            "{\"kind\": \"start\", \"map_index\": 0}");
    }

    #[test]
    fn parses_command_tag_missing() {
        let sample = "Hello world";
        assert_eq!(extract_command_tag(sample), None);
    }

    #[test]
    fn admin_allowlist_logic() {
        let mut cfg = DiscordConfig::default();
        cfg.admins = vec!["alice".into(), "bob".into()];
        assert!(cfg.is_admin("alice"));
        assert!(!cfg.is_admin("eve"));

        cfg.admins.clear();
        // empty allowlist => everyone admitted (sandbox mode)
        assert!(cfg.is_admin("anyone"));
    }
}
