//! Internal command router used by ALL channels (desktop UI + remote adapters).
//!
//! The router accepts a normalized `RemoteCommand` and a `RemoteCommandContext`
//! (channel + actor info) and dispatches to the same Tauri-server-side
//! functions the desktop UI already calls. We are intentionally thin here so
//! that adding Telegram/Discord/etc is a parsing concern, not an execution one.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Desktop,
    Web,
    Rest,
    Telegram,
    Discord,
    Whatsapp,
    Signal,
    Wechat,
    Ssh,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommandContext {
    pub channel: Channel,
    pub actor_id: String,
    pub actor_name: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandKind {
    Start,
    Stop,
    Restart,
    Status,
    Logs,
    Ip,
    ConfigGet,
    ConfigSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommand {
    pub kind: CommandKind,
    pub map_index: Option<u32>,
    pub config_patch: Option<serde_json::Value>,
    pub tail: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum RouterOutcome {
    Started { pid: u32, map: String },
    Stopped { map: String },
    Restarted { map: String },
    Status { running: bool, maps: Vec<MapDigest> },
    Logs { lines: Vec<String> },
    Ip { primary: Option<String>, entries: Vec<IpDigest> },
    ConfigGet { toml: String },
    ConfigSet { applied: usize },
    Error { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDigest {
    pub map_index: u32,
    pub map_id: String,
    pub map_label: String,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpDigest {
    pub id: String,
    pub address: String,
    pub primary: bool,
}

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("viewer cannot run admin command `{0:?}`")]
    Forbidden(CommandKind),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("server error: {0}")]
    Internal(String),
}

impl RouterOutcome {
    pub fn to_user_message(&self) -> String {
        match self {
            RouterOutcome::Started { pid, map }     => format!("✅ {map} started (pid {pid})"),
            RouterOutcome::Stopped { map }          => format!("⏹ {map} stopped"),
            RouterOutcome::Restarted { map }         => format!("♻ {map} restarted"),
            RouterOutcome::Status { running, maps } => {
                let running_count = maps.iter().filter(|m| m.running).count();
                format!(
                    "Server {} · {} / {} maps running",
                    if *running { "RUNNING" } else { "STOPPED" },
                    running_count, maps.len(),
                )
            }
            RouterOutcome::Logs { lines }          => lines.join("\n"),
            RouterOutcome::Ip { primary, entries } => {
                let mut s = String::new();
                s.push_str(&format!("primary={}\n", primary.as_deref().unwrap_or("none")));
                for e in entries {
                    s.push_str(&format!("  - {}\n", e.address));
                }
                s
            }
            RouterOutcome::ConfigGet { toml }      => toml.clone(),
            RouterOutcome::ConfigSet { applied }   => format!("✅ applied {applied} patch entries"),
            RouterOutcome::Error { reason }        => format!("⚠ {reason}"),
        }
    }
}

/// The router trait — adopted by the integration dispatcher in `lib.rs`.
///
/// In production this is fulfilled by a real closure injected from
/// `lib::run()` (see signature comment below). Keeping it as a trait enables
/// unit tests to swap in a recording fakes without booting an INI subprocess.
pub trait CommandRouter: Send + Sync + 'static {
    fn dispatch(&self, ctx: RemoteCommandContext, cmd: RemoteCommand) -> Result<RouterOutcome, RouterError>;
}

/// Builds a default authorization policy. Admin can do anything. Viewer can
/// only read (`status`, `logs`, `ip`, `config_get`). All write/config_set
/// are restricted to admins. Centralised here so all 8 channel adapters
/// apply the same gate.
pub fn authorize(ctx: &RemoteCommandContext, cmd: &RemoteCommand) -> Result<(), RouterError> {
    use CommandKind::*;
    if ctx.role == Role::Admin { return Ok(()); }
    match cmd.kind {
        Status | Logs | Ip | ConfigGet => Ok(()),
        _ => Err(RouterError::Forbidden(cmd.kind.clone())),
    }
}
