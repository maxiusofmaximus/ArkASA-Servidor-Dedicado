//! Telegram bot adapter with optional AI capabilities.
//!
//! Long-polls `api.telegram.org/bot<token>/getUpdates` and resolves the
//! configured commands (`/start`, `/stop`, `/status`, `/logs`, `/ip`,
//! `/restart`). Each command is mapped to a normalized `RemoteCommand`
//! and routed via the supplied function pointer.
//!
//! If AI is enabled and the user sends a natural language message (not starting with `/`),
//! it queries the OpenAI-compatible AI engine (Cerebras, Nvidia NIM, vLLM, llama.cpp, etc.)
//! to answer or generate a structured command.
//!
//! Authentication is per `chat_id`: only IDs in the TOML / environment allowlist may
//! invoke admin commands. Anyone outside the list gets a polite "not authorised" reply.

use crate::integrations::command_router::{
    CommandKind, RemoteCommand, RemoteCommandContext, RouterOutcome,
    Channel as RouterChannel,
    gated_chat_binding, run_with_receipts, PipelineOutcome,
};
use crate::integrations::Platform as IdentityPlatform;
use crate::integrations::receipt_emit::try_global;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub token:   String,
    pub admins:  Vec<i64>,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        let enabled = std::env::var("TELEGRAM_ENABLED").map(|v| v == "true").unwrap_or(false);
        let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        let admins = std::env::var("TELEGRAM_ADMINS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect();
        Self { enabled, token, admins }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Update {
    update_id: i64,
    #[serde(default)]
    message: Option<Message>,
}

#[derive(Debug, Clone, Deserialize)]
struct Message {
    chat: Chat,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    from: Option<User>,
}

#[derive(Debug, Clone, Deserialize)]
struct Chat { id: i64 }

#[derive(Debug, Clone, Deserialize)]
struct User {
    id: i64,
    #[serde(default)]
    first_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendMessageBody {
    chat_id: i64,
    text: String,
}

pub struct TelegramBot {
    cfg: TelegramConfig,
    http: reqwest::Client,
    last_long_poll: Option<Update>,
    offsets_set: bool,
    last_cmd_at: std::collections::HashMap<i64, Instant>,
    ai: Option<crate::integrations::ai::AiClient>,
}

impl TelegramBot {
    pub fn new(cfg: TelegramConfig) -> Self {
        let ai_client = crate::integrations::ai::AiClient::from_env();
        let ai = if ai_client.enabled() { Some(ai_client) } else { None };
        Self {
            cfg,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(35))
                .build().expect("client"),
            last_long_poll: None,
            offsets_set: false,
            last_cmd_at: Default::default(),
            ai,
        }
    }

    pub fn enabled(&self) -> bool { self.cfg.enabled && !self.cfg.token.is_empty() }

    pub fn last_offset(&self) -> i64 {
        self.last_long_poll.as_ref().map(|u| u.update_id).unwrap_or(0)
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<(), String> {
        if !self.enabled() { return Ok(()); }
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.cfg.token);
        let body = SendMessageBody { chat_id, text: text.to_string() };
        let resp = self.http.post(&url).json(&body).send().await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("sendMessage {}", resp.status()));
        }
        Ok(())
    }

    /// Single long-poll iteration. Generic over the router function.
    pub async fn tick<F>(&mut self, router: &F) -> Result<Option<(i64, String)>, String>
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + Send + Sync + ?Sized,
    {
        if !self.enabled() { return Ok(None); }

        let mut url = format!(
            "https://api.telegram.org/bot{}/getUpdates?timeout=25",
            self.cfg.token,
        );
        if self.offsets_set {
            url.push_str(&format!("&offset={}", self.last_offset() + 1));
        }
        let resp = self.http.get(&url).send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("telegram getUpdates {}", resp.status()));
        }
        let parsed: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if parsed["ok"] != serde_json::Value::Bool(true) {
            return Err(format!("telegram payload not ok: {parsed}"));
        }
        let updates: Vec<Update> = serde_json::from_value(parsed["result"].clone())
            .map_err(|e| e.to_string())?;
        for u in updates {
            self.last_long_poll = Some(u.clone());
            self.offsets_set = true;
            if let Some(msg) = u.message.clone() {
                if let Some(text) = msg.text.clone() {
                    if let Some(reply) = self.handle_message(&msg, &text, router).await {
                        return Ok(Some((msg.chat.id, reply)));
                    }
                }
            }
        }
        Ok(None)
    }

    async fn handle_message<F>(
        &mut self,
        msg: &Message,
        text: &str,
        router: &F,
    ) -> Option<String>
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + Send + Sync + ?Sized,
    {
        let chat_id = msg.chat.id;
        let chat_id_str = chat_id.to_string();

        // Rate limit (per chat_id). Pre-policy so a spammer can't flood
        // the receipts ledger either.
        if let Some(prev) = self.last_cmd_at.get(&chat_id) {
            if prev.elapsed() < Duration::from_secs(3) {
                return Some("⏱ Rate-limited: please wait a moment.".into());
            }
        }
        self.last_cmd_at.insert(chat_id, Instant::now());

        let name = msg.from.as_ref()
            .and_then(|u| u.first_name.clone())
            .unwrap_or_else(|| format!("tg-{chat_id}"));
        let emitter = try_global();

        // Build the 7-axis identity binding from the configured admins.
        let admins: Vec<String> = self.cfg.admins.iter().map(|i| i.to_string()).collect();
        let binding = if admins.is_empty() {
            crate::integrations::command_router::default_chat_binding(
                RouterChannel::Telegram, &chat_id_str,
            )
        } else {
            gated_chat_binding(RouterChannel::Telegram, &chat_id_str, admins)
        };

        let trace_id = format!("tg:{}", chat_id);

        // Slash command path
        if text.starts_with('/') {
            let mut parts = text.split_whitespace();
            let cmd_str = parts.next().unwrap_or("").to_ascii_lowercase();
            let arg: Option<String> = parts.collect::<Vec<_>>().join(" ").into();

            let kind_opt = match cmd_str.as_str() {
                "/start"      => Some(CommandKind::Start),
                "/stop"       => Some(CommandKind::Stop),
                "/restart"    => Some(CommandKind::Restart),
                "/status"     => Some(CommandKind::Status),
                "/logs"       => Some(CommandKind::Logs),
                "/ip"         => Some(CommandKind::Ip),
                _            => None,
            };

            let Some(kind) = kind_opt else {
                return Some("⚠ Unknown command. Try /start, /stop, /restart, /status, /logs, /ip".into());
            };

            let kind_label = match kind {
                CommandKind::Start      => "start",
                CommandKind::Stop       => "stop",
                CommandKind::Restart    => "restart",
                CommandKind::Status     => "status",
                CommandKind::Logs       => "logs",
                CommandKind::Ip         => "ip",
                CommandKind::ConfigGet  => "config_get",
                CommandKind::ConfigSet  => "config_set",
                CommandKind::StartInstance => "start_instance",
                CommandKind::StopInstance  => "stop_instance",
            };
            let remote = RemoteCommand {
                kind,
                map_index: None,
                config_patch: None,
                tail: arg.as_ref().and_then(|s| s.parse().ok()),
            };
            let outcome = run_with_receipts(
                emitter.as_deref(),
                IdentityPlatform::Telegram,
                RouterChannel::Telegram,
                &binding,
                &chat_id_str,
                &name,
                &trace_id,
                text,
                None,
                Some(kind_label),
                Some(remote),
                &|ctx, cmd| (router)(ctx, cmd),
            );
            return Some(pipeline_to_reply(&outcome));
        }

        // Natural-language path: query AI, possibly extract [COMMAND: {...}].
        let Some(ai) = &self.ai else {
            return Some(
                "🤖 El asistente de Inteligencia Artificial está inactivo. \
                 Usa comandos de barra diagonal como `/status` o `/start`.".to_string(),
            );
        };
        let ai_reply = match ai.query(text).await {
            Ok(r)  => r,
            Err(e) => return Some(format!("⚠ AI service error: {e}")),
        };

        if let Some(cmd_json_str) = extract_command_tag(&ai_reply).map(str::to_string) {
            #[derive(Debug, Deserialize)]
            struct ParsedAiCmd {
                kind:      String,
                #[serde(default)]
                map_index: Option<u32>,
                #[serde(default)]
                tail:      Option<u32>,
            }
            if let Ok(parsed_cmd) = serde_json::from_str::<ParsedAiCmd>(&cmd_json_str) {
                let cmd_kind = match parsed_cmd.kind.as_str() {
                    "start"    => Some(CommandKind::Start),
                    "stop"     => Some(CommandKind::Stop),
                    "restart"  => Some(CommandKind::Restart),
                    "status"   => Some(CommandKind::Status),
                    "logs"     => Some(CommandKind::Logs),
                    "ip"       => Some(CommandKind::Ip),
                    _ => None,
                };
                if let Some(kind) = cmd_kind {
                    let kind_label = parsed_cmd.kind.as_str();
                    let remote = RemoteCommand {
                        kind,
                        map_index: parsed_cmd.map_index,
                        config_patch: None,
                        tail: parsed_cmd.tail,
                    };
                    let outcome = run_with_receipts(
                        emitter.as_deref(),
                        IdentityPlatform::Telegram,
                        RouterChannel::Telegram,
                        &binding,
                        &chat_id_str,
                        &name,
                        &trace_id,
                        text,
                        None,
                        Some(kind_label),
                        Some(remote),
                        &|ctx, cmd| (router)(ctx, cmd),
                    );
                    let friendly = ai_reply
                        .replace(&format!("[COMMAND: {}]", &cmd_json_str), "")
                        .trim()
                        .to_string();
                    return match outcome {
                        PipelineOutcome::Done(msg) => Some(format!(
                            "{}\n\n⚡ **Ejecutando comando:**\n{}",
                            friendly, msg,
                        )),
                        PipelineOutcome::Rejected(msg) => Some(format!(
                            "{}\n\n⚠ {}",
                            friendly, msg,
                        )),
                        PipelineOutcome::NoCommand(_) => Some(friendly),
                    };
                }
            }
        }
        Some(ai_reply)
    }
}

fn extract_command_tag(text: &str) -> Option<&str> {
    let start = text.find("[COMMAND:")?;
    let after = start + "[COMMAND:".len();
    let rest = &text[after..];
    let end = rest.find(']')?;
    Some(rest[..end].trim())
}

fn pipeline_to_reply(outcome: &PipelineOutcome) -> String {
    match outcome {
        PipelineOutcome::Done(msg)       => msg.clone(),
        PipelineOutcome::Rejected(msg)   => msg.clone(),
        PipelineOutcome::NoCommand(s)    => s.clone(),
    }
}

/// Spawn the Telegram polling loop on the current thread (Tauri runtime is
/// `current_thread`, no `Send` required).
pub async fn spawn_looper<F>(mut bot: TelegramBot, router: std::sync::Arc<F>)
where
    F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + ?Sized + Send + Sync + 'static,
{
    log::info!("Telegram bot looper task successfully initialized.");
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        match bot.tick(&*router).await {
            Ok(Some((chat_id, reply))) => {
                if let Err(e) = bot.send_message(chat_id, &reply).await {
                    log::error!("Telegram send_message failed: {}", e);
                }
            }
            Ok(None) => {}
            Err(e) => {
                log::error!("Telegram polling error: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
