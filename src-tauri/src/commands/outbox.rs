//! Durable action queue, surfaced to the UI and drained by the loopback API or
//! a scheduled task.
//!
//! The queue itself lives in `ark-core::offline` (shared with the mobile app),
//! and the drain policy lives in `ark-core::offline::sender`. This module only
//! supplies the **transport**: a dispatcher that speaks RCON to the ARK server.
//!
//! Consequence worth stating: the desktop and the phone now share the same
//! state machine and the same idempotency guarantees, and only the socket
//! differs.

use std::sync::Arc;

use ark_core::offline::outbox::{NewOutboxEntry, Outbox, OutboxEntry};
use ark_core::offline::sender::{Dispatcher, OutboxSender};
use serde::Serialize;

use crate::ark::rcon::RconClient;

/// Speaks RCON to the server for one outbox entry.
///
/// Entries carry `{ "command": "..." }` as their payload; the RCON endpoint is
/// resolved from the active configuration, never from the entry, so a stale
/// queued action cannot target a different server than the one now selected.
pub struct RconDispatcher {
    rcon_port: u16,
    password: String,
}

impl RconDispatcher {
    /// Resolves the RCON endpoint from the active profile's configuration.
    ///
    /// Async because the profile registry lives on disk; the dispatcher is
    /// built once per drain, not per entry.
    pub async fn from_active_config(app: &tauri::AppHandle) -> Result<Self, String> {
        let config = crate::commands::server::active_config(app).await?;
        Ok(Self {
            rcon_port: config.network.rcon_port,
            password: config.identification.admin_password.clone(),
        })
    }
}

#[async_trait::async_trait]
impl Dispatcher for RconDispatcher {
    async fn dispatch(&self, entry: &OutboxEntry) -> Result<String, String> {
        if entry.action != "rcon.command" {
            // An unknown action is a bug, not a transient failure: fail hard so
            // it exhausts its budget instead of retrying forever.
            return Err(format!("unsupported action: {}", entry.action));
        }

        let payload: serde_json::Value = serde_json::from_str(&entry.payload)
            .map_err(|e| format!("malformed payload: {e}"))?;
        let command = payload
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "payload has no `command` field".to_string())?;

        let client = RconClient::new(self.rcon_port, self.password.clone());
        client.exec(command).await
    }
}

/// Queue status for the UI.
#[derive(Debug, Clone, Serialize)]
pub struct OutboxStatusView {
    pub pending: u64,
    pub in_flight: u64,
    pub done: u64,
    pub failed: u64,
    pub path: String,
}

/// One queued action, as shown to the operator.
#[derive(Debug, Clone, Serialize)]
pub struct OutboxEntryView {
    pub idempotency_key: String,
    pub action: String,
    pub payload: String,
    pub state: String,
    pub attempts: u32,
    pub max_attempts: u32,
    pub last_error: Option<String>,
}

fn open_outbox(app: &tauri::AppHandle) -> Result<Outbox, String> {
    let dir = crate::commands::get_config_dir(app)?;
    Outbox::open(dir).map_err(|e| e.to_string())
}

/// Current outbox counts and location.
#[tauri::command]
pub fn outbox_status(app: tauri::AppHandle) -> Result<OutboxStatusView, String> {
    let outbox = open_outbox(&app)?;
    let (pending, in_flight, done, failed) = outbox.counts().map_err(|e| e.to_string())?;
    Ok(OutboxStatusView {
        pending,
        in_flight,
        done,
        failed,
        path: outbox.path().display().to_string(),
    })
}

/// Every queued action, oldest first.
#[tauri::command]
pub fn outbox_list(app: tauri::AppHandle) -> Result<Vec<OutboxEntryView>, String> {
    let outbox = open_outbox(&app)?;
    Ok(outbox
        .all()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|e| OutboxEntryView {
            idempotency_key: e.idempotency_key,
            action: e.action,
            payload: e.payload,
            state: e.state.as_str().to_string(),
            attempts: e.attempts,
            max_attempts: e.max_attempts,
            last_error: e.last_error,
        })
        .collect())
}

/// Queues an RCON command.
///
/// Returns `false` when the idempotency key was already present, which the UI
/// should read as "already on its way" rather than an error.
#[tauri::command]
pub fn outbox_enqueue_command(
    app: tauri::AppHandle,
    idempotency_key: String,
    command: String,
) -> Result<bool, String> {
    let outbox = open_outbox(&app)?;
    let payload = serde_json::json!({ "command": command }).to_string();
    outbox
        .enqueue(NewOutboxEntry::new(
            idempotency_key,
            "rcon.command",
            payload,
        ))
        .map_err(|e| e.to_string())
}

/// Drains the queue once, sending due actions over RCON.
#[tauri::command]
pub async fn outbox_drain(app: tauri::AppHandle) -> Result<String, String> {
    let outbox = Arc::new(open_outbox(&app)?);

    // Recover anything a previous crash left claimed, before attempting new work.
    let reclaimed = {
        let sender = OutboxSender::new(outbox.clone(), Arc::new(NoopDispatcher));
        sender.reclaim_stale_in_flight().map_err(|e| e.to_string())?
    };

    let dispatcher = RconDispatcher::from_active_config(&app).await?;
    let sender = OutboxSender::new(outbox, Arc::new(dispatcher));
    let report = sender.drain_once().await.map_err(|e| e.to_string())?;

    Ok(format!(
        "delivered={} retrying={} gave_up={} reclaimed={}",
        report.delivered(),
        report.retrying(),
        report.gave_up(),
        reclaimed
    ))
}

/// Deletes delivered entries older than `age_secs`.
#[tauri::command]
pub fn outbox_prune(app: tauri::AppHandle, age_secs: u64) -> Result<u64, String> {
    let outbox = open_outbox(&app)?;
    outbox.prune_done(age_secs).map_err(|e| e.to_string())
}

/// Only used for `reclaim_stale_in_flight`, which never calls `dispatch`.
struct NoopDispatcher;

#[async_trait::async_trait]
impl Dispatcher for NoopDispatcher {
    async fn dispatch(&self, _entry: &OutboxEntry) -> Result<String, String> {
        unreachable!("reclaim must not dispatch")
    }
}
