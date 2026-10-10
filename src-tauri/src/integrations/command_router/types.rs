//! Router value types: channels, commands, contexts, outcomes, and errors.
//!
//! Split out of the original `command_router.rs` (P23) without behaviour or
//! public-API changes — every public item here is re-exported by the parent
//! `command_router` module (`pub use types::*`).

use crate::integrations::identity::{
    ChannelBinding, Identity, IdentityResolution, RuntimeClass,
};
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
    Slack,
    Whatsapp,
    Signal,
    Wechat,
    Ssh,
}

/// Single source of truth lives in `auth` (P24).
pub use crate::auth::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommandContext {
    pub channel: Channel,
    pub actor_id: String,
    pub actor_name: String,
    pub role: Role,
    /// v2.2 — populated by every remote adapter via `ChannelBinding::resolve`.
    /// Legacy `Desktop` callers leave this `None` and `authorize()` admits
    /// them automatically (no identity gate).
    #[serde(default)]
    pub identity: Option<Identity>,
}

impl RemoteCommandContext {
    /// Convenience constructor for the desktop UI (no identity gate).
    pub fn desktop(actor_id: impl Into<String>, actor_name: impl Into<String>, role: Role) -> Self {
        Self {
            channel: Channel::Desktop,
            actor_id: actor_id.into(),
            actor_name: actor_name.into(),
            role,
            identity: None,
        }
    }

    /// Bind a 7-axis identity from a channel binding. Returns a fully-populated
    /// context or `RouterError::Internal(identity-format-reason)` if the actor
    /// is not admitted by the allow-list.
    pub fn from_binding(
        channel: Channel,
        binding: &ChannelBinding,
        actor_id: &str,
        actor_name: &str,
        role: Role,
        runtime_class: RuntimeClass,
        session_key: &str,
    ) -> Result<Self, RouterError> {
        match binding.resolve(actor_id, runtime_class, session_key) {
            IdentityResolution::Bound(id) => Ok(Self {
                channel,
                actor_id: actor_id.to_string(),
                actor_name: actor_name.to_string(),
                role,
                identity: Some(id),
            }),
            IdentityResolution::Rejected(reason) => Err(RouterError::Internal(
                format!("identity-rejected: {}", reason.human()),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandKind {
    Start,
    Stop,
    Restart,
    Status,
    Logs,
    Ip,
    ConfigGet,
    ConfigSet,
    /// Start a single map instance by index (delegated to `bridge::start_instance_inner`).
    StartInstance,
    /// Stop a single map instance by index (delegated to `bridge::stop_instance_inner`).
    StopInstance,
}

impl CommandKind {
    /// Canonical kebab-style label used in receipts, log lines, and
    /// slash-command routing. Single source of truth — replaces 12+
    /// inline `match cmd { Start => "start", ... }` blocks that were
    /// duplicated across telegram / discord / slack / wechat / whatsapp
    /// / ssh / signal / rest / http_api.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start         => "start",
            Self::Stop          => "stop",
            Self::Restart       => "restart",
            Self::Status        => "status",
            Self::Logs          => "logs",
            Self::Ip            => "ip",
            Self::ConfigGet     => "config_get",
            Self::ConfigSet     => "config_set",
            Self::StartInstance => "start_instance",
            Self::StopInstance  => "stop_instance",
        }
    }

    /// Inverse of `as_str`: parse a slash-command payload (with or without
    /// the leading `/`) into a `CommandKind`. Returns `None` for unknown
    /// verbs, allowing the channel adapters to reject silently without
    /// inventing an error enum variant for the same string.
    pub fn parse_slash(s: &str) -> Option<Self> {
        let trimmed = s.trim_start_matches('/');
        Some(match trimmed.to_ascii_lowercase().as_str() {
            "start"           => Self::Start,
            "stop"            => Self::Stop,
            "restart"         => Self::Restart,
            "status"          => Self::Status,
            "logs"            => Self::Logs,
            "ip"              => Self::Ip,
            "config_get"      => Self::ConfigGet,
            "config_set"      => Self::ConfigSet,
            "start_instance"  => Self::StartInstance,
            "stop_instance"   => Self::StopInstance,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommand {
    pub kind: CommandKind,
    pub map_index: Option<u32>,
    pub config_patch: Option<serde_json::Value>,
    pub tail: Option<u32>,
}

/// Type alias for the multi-channel router closure injected by
/// `lib::run()` and consumed by the desktop app, every chat adapter,
/// and the loopback HTTP API. Single source of truth so we don't keep
/// re-spelling the bounds inline at each adapter (`Arc<F: ???>`).
///
/// P19: this used to be a synchronous `Fn(...)` whose body would
/// `tauri::async_runtime::block_on` inside itself — which forced every
/// caller to wrap calls in `tokio::task::spawn_blocking` to avoid
/// reactor starvation. The router is now genuinely async (`Fn ->
/// Future<...>`), so the call sites just `await` it on the existing
/// tokio runtime. Migrating the type + 3 production call sites
/// eliminates the dance and the hidden deadlock footgun.
pub type RouterFn =
    dyn Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + Send + Sync;

/// Async variant — preferred for any new code. The wrapper
/// `RouterFn::dispatch_async` adapts the legacy sync `RouterFn` to this
/// shape by `spawn_blocking`-shifting the result, so the type
/// transition can land call-site by call-site without breaking
/// adapters that still speak the sync form (see S12 history).
pub type AsyncRouterFn =
    dyn Fn(RemoteCommandContext, RemoteCommand) -> BoxFuture<'static, Result<RouterOutcome, String>>
        + Send
        + Sync;

/// `Send + 'static` boxed future re-export so callers don't have to
/// import `futures-util` directly.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

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

/// Outcome of a single inbound pipeline run, returned to the chat adapter so
/// it can render a user-facing reply.
#[derive(Debug)]
pub enum PipelineOutcome {
    /// Identity gateway rejected the actor — message was sent to the user.
    Rejected(String),
    /// Router returned successfully — message was sent to the user.
    Done(String),
    /// No actionable command was present (e.g. AI responded without a
    /// `[COMMAND:]` tag). Reply text is still returned for the user.
    NoCommand(String),
}

/// Async-pipeline result. Mirrors [`PipelineOutcome`] (user-facing message)
/// but also surfaces the raw [`RouterOutcome`] when the router actually ran,
/// so loopback HTTP API callers (P27 `admin_only_call` / `internal_dispatch`)
/// can serialize the typed payload instead of washing it through
/// `to_user_message`. The field is `None` when identity was rejected or no
/// command was resolved — i.e. the cases where the chat-bot path would
/// fall back to rendering the human-readable reply.
#[derive(Debug)]
pub struct PipelineExecution {
    pub pipeline:       PipelineOutcome,
    pub router_outcome: Option<RouterOutcome>,
}
