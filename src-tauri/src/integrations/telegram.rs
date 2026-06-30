//! Telegram bot adapter.
//!
//! Long-polls `api.telegram.org/bot<token>/getUpdates` and resolves the
//! configured commands (`/start`, `/stop`, `/status`, `/logs`, `/ip`,
//! `/restart`). Each command is mapped to a normalized `RemoteCommand`
//! and routed via the supplied function pointer.
//!
//! Authentication is per `chat_id`: only IDs in the TOML allowlist may
//! invoke admin commands. Anyone outside the list gets a polite
//! "not authorised" reply. Rate-limited at 1 cmd / 5 s per chat.
//!
//! Setup:
//!   1. Create a bot via `@BotFather`.
//!   2. Configure `[integrations.telegram]` in TOML:
//!        enabled = true
//!        token   = "12345:ABC-DEF…"
//!        admins  = [ 123456789 ]   # your chat_id
//!   3. Restart the desktop app — the polling loop starts in `lib::run()`.

use crate::integrations::command_router::{
    authorize, CommandKind, RemoteCommand, RemoteCommandContext, RouterOutcome,
};
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
        Self { enabled: false, token: String::new(), admins: vec![] }
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
}

impl TelegramBot {
    pub fn new(cfg: TelegramConfig) -> Self {
        Self {
            cfg,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(35))
                .build().expect("client"),
            last_long_poll: None,
            offsets_set: false,
            last_cmd_at: Default::default(),
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

    /// Single long-poll iteration. Generic over the router function so
    /// concrete async tasks stay `Send`-compatible.
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
        if !text.starts_with('/') { return None; }
        let chat_id = msg.chat.id;
        if !self.cfg.admins.contains(&chat_id) {
            return Some(format!(
                "⚠ your chat id ({chat_id}) is not authorised for this server. \
                 Ask the operator to add it to `[integrations.telegram] admins` in TOML.",
            ));
        }

        // Rate limit
        if let Some(prev) = self.last_cmd_at.get(&chat_id) {
            if prev.elapsed() < Duration::from_secs(5) {
                return Some("⏱ rate-limited: 1 cmd / 5s".into());
            }
        }

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
            return Some("⚠ unknown command. Try /start /stop /restart /status /logs /ip".into());
        };
        self.last_cmd_at.insert(chat_id, Instant::now());

        let name = msg.from.as_ref()
            .and_then(|u| u.first_name.clone())
            .unwrap_or_else(|| format!("tg-{chat_id}"));

        let ctx = RemoteCommandContext {
            channel:    crate::integrations::command_router::Channel::Telegram,
            actor_id:   chat_id.to_string(),
            actor_name: name,
            role: crate::integrations::command_router::Role::Admin,
        };
        let remote = RemoteCommand {
            kind,
            map_index: None,
            config_patch: None,
            tail: arg.as_ref().and_then(|s| s.parse().ok()),
        };
        if let Err(e) = authorize(&ctx, &remote) {
            return Some(format!("⚠ forbidden: {e}"));
        }
        match router(ctx, remote) {
            Ok(o)  => Some(o.to_user_message()),
            Err(e) => Some(format!("⚠ router: {e}")),
        }
    }
}

/// Spawn the Telegram polling loop with a generic router type.
///
/// Tauri 2 spawns tasks on a `current_thread` runtime so we can use
/// `tauri::async_runtime::spawn` directly — the future does not need
/// to be `Send` because there is no cross-thread boundary.
/// Spawn the Telegram polling loop. Returns a 'static task handle.
///
/// Tauri 2's runtime is `current_thread`, so we use `tokio::task::spawn_local`
/// (which does NOT require `Send`). The caller decides when to abort.
///
/// The signature uses `web_local` (an owned closure) and `tokio::task::LocalSet`
/// isn't strictly necessary because each tick + send is independent.
/// Spawn the Telegram polling loop.
///
/// Hito 6 final: keeps the `Bot` owned inside the future (no shared
/// Mutex, no Send-hostile guards). If you need to inspect te bot state
/// externally, push it through a `watch::channel` in Hito 12.
pub fn spawn_looper<F>(bot: TelegramBot, router: std::sync::Arc<F>) -> impl std::future::Future<Output = ()>
where
    F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + Send + Sync + 'static,
{
    let _ = bot;
    let _ = router;
    async move {
        // Implementation lives inline in `lib.rs::run()` once Hito 12 wires
        // the actual loop. This stub keeps the integration endpoint exported
        // so the rest of the pipeline compiles and the wiring in lib.rs can
        // simply call `telegram::bot.tick_loop(router).await` if convenient.
    }
}
