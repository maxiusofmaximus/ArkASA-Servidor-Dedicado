//! Internal command router used by ALL channels (desktop UI + remote adapters).
//!
//! The router accepts a normalized `RemoteCommand` and a `RemoteCommandContext`
//! (channel + actor info) and dispatches to the same Tauri-server-side
//! functions the desktop UI already calls. We are intentionally thin here so
//! that adding Telegram/Discord/etc is a parsing concern, not an execution one.
//!
//! ## Identity model (v2.2 — 7-axis)
//!
//! Every adapter MUST populate `RemoteCommandContext.identity` (an
//! `Identity` carrying the 7 axes of Agent Harness Core) BEFORE calling
//! `authorize()`. The resolver returns either `IdentityResolution::Bound(id)`
//! or `IdentityResolution::Rejected(reason)` (fail-closed).
//!
//! Backwards-compat: legacy desktop/UI callers may leave `identity = None`;
//! `authorize()` will admit them as the local admin scope.

use crate::integrations::identity::{
    ChannelBinding, Identity, IdentityResolution, Platform, RuntimeClass,
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

/// Maps our router `Channel` to the identity `Platform`. Centralised so
/// receipts and trace logs see a single source of truth.
pub fn platform_for_channel(c: Channel) -> Platform {
    match c {
        Channel::Desktop   => Platform::Desktop,
        Channel::Web       => Platform::Web,
        Channel::Rest      => Platform::Rest,
        Channel::Telegram  => Platform::Telegram,
        Channel::Discord   => Platform::Discord,
        Channel::Slack     => Platform::Slack,
        Channel::Whatsapp  => Platform::WhatsApp,
        Channel::Signal    => Platform::Signal,
        Channel::Wechat    => Platform::Wechat,
        Channel::Ssh       => Platform::Ssh,
    }
}

/// Build a `ChannelBinding` for a single chat thread, sandbox mode (anyone admitted).
pub fn default_chat_binding(channel: Channel, chat_id: &str) -> ChannelBinding {
    ChannelBinding {
        platform:     platform_for_channel(channel),
        account_id:   "@bot".to_string(),
        channel_id:    chat_id.to_string(),
        admin_actors:  None,
        default_agent: "main".to_string(),
    }
}

/// Build a `ChannelBinding` for a single chat thread with an explicit allowlist.
pub fn gated_chat_binding(channel: Channel, chat_id: &str, admins: Vec<String>) -> ChannelBinding {
    ChannelBinding {
        platform:     platform_for_channel(channel),
        account_id:   "@bot".to_string(),
        channel_id:    chat_id.to_string(),
        admin_actors:  Some(admins),
        default_agent: "main".to_string(),
    }
}

/// Default runtime class new adapters should use for human-issued commands.
pub fn default_runtime_class() -> RuntimeClass { RuntimeClass::Interactive }

/// Convenience: extract a session key from an adapter's update ID / message ID.
/// `None` ⇒ caller didn't supply one (we still mint a synthetic key).
pub fn session_key_from(parts: &[&str]) -> String {
    parts.iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(":")
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

/// Run the full receipt-emit pipeline for a single inbound message:
/// ChannelIngress → IdentityCheck → [QueueEnqueue] → RuntimePipeline → ChannelDelivery.
///
/// All three of `Telegram`, `Discord`, `Slack` invoke this — the chat-specific
/// parsing has already happened and surfaced a `CommandKind` (or skipped it
/// for AI-only paths).
///
/// Returns the user-facing reply string the bot should send back, plus a
/// `PipelineOutcome` tag describing what happened so the caller can decide
/// whether to actually post a message.
///
/// `emitter` may be `None` (e.g. unit tests, init-race), in which case all
/// receipts are silently dropped. Adapter still gets a reply either way.
pub fn run_with_receipts<F>(
    emitter:        Option<&crate::integrations::receipt_emit::Emitter>,
    platform:       Platform,
    ctx_channel:    Channel,
    binding:        &ChannelBinding,
    actor_id:       &str,
    actor_name:     &str,
    trace_id:       &str,
    raw_text:       &str,
    language:       Option<&str>,
    kind_label:     Option<&str>,   // None ⇒ no CommandKind resolved yet
    remote:         Option<RemoteCommand>,
    router:         &F,
) -> PipelineOutcome
where
    F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + ?Sized,
{
    let rctx = crate::integrations::receipt_emit::ReceiptContext {
        trace_id,
        actor_id,
        actor_name,
        runtime: default_runtime_class(),
    };

    // 1. Channel Ingress
    if let Some(e) = emitter { e.ingress(platform, rctx, raw_text, language); }

    // 2. Identity check
    let actor_id_str = actor_id.to_string();
    let resolution = binding.resolve(
        actor_id,
        default_runtime_class(),
        &session_key_from(&[trace_id, &actor_id_str]),
    );
    match resolution {
        IdentityResolution::Bound(id) => {
            if let Some(e) = emitter { e.identity_admitted(rctx, platform, &id); }
        }
        IdentityResolution::Rejected(reason) => {
            let size = match &binding.admin_actors {
                Some(v) => v.len() as u32,
                None    => 0,
            };
            if let Some(e) = emitter {
                e.identity_rejected(rctx, platform, actor_id, "channel_fail_closed", size);
            }
            let user_msg = format!(
                "⚠ Your identity `{}` is not authorised for this server ({})",
                actor_id, reason.human()
            );
            if let Some(e) = emitter { e.delivery(rctx, platform, actor_id, DeliveryStatus::Skipped, None); }
            return PipelineOutcome::Rejected(user_msg);
        }
    }

    // 3. Router invocation (if a CommandKind was resolved)
    let Some(remote) = remote else {
        // No command — don't emit RuntimePipeline; just deliver a no-op.
        return PipelineOutcome::NoCommand(String::new());
    };
    let kind_label = kind_label.unwrap_or("unknown");

    let ctx = match RemoteCommandContext::from_binding(
        ctx_channel,
        binding,
        actor_id,
        actor_name,
        Role::Admin,
        default_runtime_class(),
        &session_key_from(&[trace_id, &actor_id_str]),
    ) {
        Ok(c)  => c,
        Err(e) => {
            // Shouldn't happen if Bound above was admitted, but stay safe.
            let reply = format!("⚠ router-ctx-failed: {e}");
            if let Some(em) = emitter {
                em.runtime_completed(rctx, platform, kind_label, remote.map_index, &serde_json::json!({"error": &reply}));
                em.delivery(rctx, platform, actor_id, DeliveryStatus::Failed, Some(&reply));
            }
            return PipelineOutcome::Done(reply);
        }
    };
    if let Err(e) = authorize(&ctx, &remote) {
        let reply = format!("⚠ Forbidden: {e}");
        if let Some(em) = emitter {
            em.runtime_completed(rctx, platform, kind_label, remote.map_index, &serde_json::json!({"error": &reply}));
            em.delivery(rctx, platform, actor_id, DeliveryStatus::Failed, Some(&reply));
        }
        return PipelineOutcome::Done(reply);
    }

    match router(ctx, remote.clone()) {
        Ok(outcome) => {
            let user_msg = outcome.to_user_message();
            let v = serde_json::to_value(&outcome).unwrap_or(serde_json::Value::Null);
            if let Some(e) = emitter {
                e.runtime_completed(rctx, platform, kind_label, remote.map_index, &v);
                e.delivery(rctx, platform, actor_id, DeliveryStatus::Delivered, None);
            }
            PipelineOutcome::Done(user_msg)
        }
        Err(e) => {
            let reply = format!("⚠ Router error: {e}");
            if let Some(em) = emitter {
                em.runtime_completed(rctx, platform, kind_label, remote.map_index, &serde_json::json!({"error": &reply}));
                em.delivery(rctx, platform, actor_id, DeliveryStatus::Failed, Some(&reply));
            }
            PipelineOutcome::Done(reply)
        }
    }
}

use crate::integrations::receipt_emit::DeliveryStatus;

/// Async companion of [`run_with_receipts`] — emits the same five-stage
/// pipeline but `await`s the router and admits a `ChannelBinding`-driven
/// identity. P27 unifies the loopback HTTP API (`admin_only_call`,
/// `internal_dispatch`) on top of this helper so the trailing stages
/// (`RuntimePipeline` + `ChannelDelivery`) are no longer silently dropped.
///
/// The contract is identical to the sync version: an emitter backed by
/// the active `Emitter` may be `None` (e.g. unit tests, init-race), in
/// which case receipts are silently dropped and the adapter still gets
/// a `PipelineOutcome` back. `kind_label` distinguishes unknown-dispatch
/// shells (the resolver failed before producing a `RemoteCommand`).
///
/// `binding` is required: the HTTP API holds the operator's allow-list
/// for desktop + webadmin, so the identity gate has to resolve before
/// we invoke the router. Chat-bot callers (Telegram/Discord/Slack)
/// keep using the sync `run_with_receipts` + `spawn_blocking` adapter
/// because their Rust callbacks are synchronous and battle-tested.
pub async fn run_with_receipts_async<F, Fut>(
    emitter:        Option<&crate::integrations::receipt_emit::Emitter>,
    platform:       Platform,
    ctx_channel:    Channel,
    binding:        &ChannelBinding,
    actor_id:       &str,
    actor_name:     &str,
    trace_id:       &str,
    raw_text:       &str,
    language:       Option<&str>,
    kind_label:     Option<&str>,   // None ⇒ no CommandKind resolved yet
    remote:         Option<RemoteCommand>,
    router:         F,
) -> PipelineExecution
where
    F: FnOnce(RemoteCommandContext, RemoteCommand) -> Fut,
    Fut: std::future::Future<Output = Result<RouterOutcome, String>>,
{
    let rctx = crate::integrations::receipt_emit::ReceiptContext {
        trace_id,
        actor_id,
        actor_name,
        runtime: default_runtime_class(),
    };

    // 1. Channel Ingress
    if let Some(e) = emitter { e.ingress(platform, rctx, raw_text, language); }

    // 2. Identity check
    let actor_id_str = actor_id.to_string();
    let resolution = binding.resolve(
        actor_id,
        default_runtime_class(),
        &session_key_from(&[trace_id, &actor_id_str]),
    );
    match resolution {
        IdentityResolution::Bound(id) => {
            if let Some(e) = emitter { e.identity_admitted(rctx, platform, &id); }
        }
        IdentityResolution::Rejected(reason) => {
            let size = match &binding.admin_actors {
                Some(v) => v.len() as u32,
                None    => 0,
            };
            if let Some(e) = emitter {
                e.identity_rejected(rctx, platform, actor_id, "channel_fail_closed", size);
            }
            let user_msg = format!(
                "⚠ Your identity `{}` is not authorised for this server ({})",
                actor_id, reason.human()
            );
            if let Some(e) = emitter { e.delivery(rctx, platform, actor_id, DeliveryStatus::Skipped, None); }
            return PipelineExecution {
                pipeline:       PipelineOutcome::Rejected(user_msg),
                router_outcome: None,
            };
        }
    }

    // 3. Router invocation (if a CommandKind was resolved)
    let Some(remote) = remote else {
        // No command — don't emit RuntimePipeline; just deliver a no-op.
        return PipelineExecution {
            pipeline:       PipelineOutcome::NoCommand(String::new()),
            router_outcome: None,
        };
    };
    let kind_label = kind_label.unwrap_or("unknown");

    let ctx = match RemoteCommandContext::from_binding(
        ctx_channel,
        binding,
        actor_id,
        actor_name,
        Role::Admin,
        default_runtime_class(),
        &session_key_from(&[trace_id, &actor_id_str]),
    ) {
        Ok(c)  => c,
        Err(e) => {
            // Shouldn't happen if Bound above was admitted, but stay safe.
            let reply = format!("⚠ router-ctx-failed: {e}");
            if let Some(em) = emitter {
                em.runtime_completed(rctx, platform, kind_label, remote.map_index, &serde_json::json!({"error": &reply}));
                em.delivery(rctx, platform, actor_id, DeliveryStatus::Failed, Some(&reply));
            }
            return PipelineExecution {
                pipeline:       PipelineOutcome::Done(reply),
                router_outcome: None,
            };
        }
    };
    if let Err(e) = authorize(&ctx, &remote) {
        let reply = format!("⚠ Forbidden: {e}");
        if let Some(em) = emitter {
            em.runtime_completed(rctx, platform, kind_label, remote.map_index, &serde_json::json!({"error": &reply}));
            em.delivery(rctx, platform, actor_id, DeliveryStatus::Failed, Some(&reply));
        }
        return PipelineExecution {
            pipeline:       PipelineOutcome::Done(reply),
            router_outcome: None,
        };
    }

    match router(ctx, remote.clone()).await {
        Ok(outcome) => {
            let user_msg = outcome.to_user_message();
            let v = serde_json::to_value(&outcome).unwrap_or(serde_json::Value::Null);
            if let Some(e) = emitter {
                e.runtime_completed(rctx, platform, kind_label, remote.map_index, &v);
                e.delivery(rctx, platform, actor_id, DeliveryStatus::Delivered, None);
            }
            PipelineExecution {
                pipeline:       PipelineOutcome::Done(user_msg),
                router_outcome: Some(outcome),
            }
        }
        Err(e) => {
            let reply = format!("⚠ Router error: {e}");
            if let Some(em) = emitter {
                em.runtime_completed(rctx, platform, kind_label, remote.map_index, &serde_json::json!({"error": &reply}));
                em.delivery(rctx, platform, actor_id, DeliveryStatus::Failed, Some(&reply));
            }
            PipelineExecution {
                pipeline:       PipelineOutcome::Done(reply),
                router_outcome: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::identity::RejectionReason;

    #[test]
    fn platform_for_channel_maps_correctly() {
        assert_eq!(platform_for_channel(Channel::Telegram), Platform::Telegram);
        assert_eq!(platform_for_channel(Channel::Discord),  Platform::Discord);
        assert_eq!(platform_for_channel(Channel::Slack),    Platform::Slack);
        assert_eq!(platform_for_channel(Channel::Desktop),  Platform::Desktop);
        assert_eq!(platform_for_channel(Channel::Rest),     Platform::Rest);
    }

    #[test]
    fn default_chat_binding_is_sandbox() {
        let b = default_chat_binding(Channel::Telegram, "chat-42");
        assert_eq!(b.platform, Platform::Telegram);
        assert_eq!(b.channel_id, "chat-42");
        assert!(b.admin_actors.is_none(), "sandbox binding should be open");
        match b.resolve("anyone", default_runtime_class(), "s1") {
            IdentityResolution::Bound(_) => {}
            other => panic!("sandbox should admit, got {other:?}"),
        }
    }

    #[test]
    fn gated_chat_binding_rejects_non_admin() {
        let b = gated_chat_binding(Channel::Slack, "C123", vec!["U1".into(), "U2".into()]);
        match b.resolve("U3", default_runtime_class(), "s2") {
            IdentityResolution::Rejected(RejectionReason::AllowlistMiss {
                configured: 2, actor_id
            }) if actor_id == "U3" => {}
            other => panic!("expected AllowlistMiss, got {other:?}"),
        }
    }

    #[test]
    fn session_key_drops_empty_segments() {
        assert_eq!(session_key_from(&["tg", "", "4242"]), "tg:4242");
        assert_eq!(session_key_from(&["", "", ""]),       "");
        assert_eq!(session_key_from(&["only"]),           "only");
    }

    #[test]
    fn remote_command_context_desktop_constructor() {
        let ctx = RemoteCommandContext::desktop("local", "Max", Role::Admin);
        assert_eq!(ctx.channel, Channel::Desktop);
        assert_eq!(ctx.actor_id, "local");
        assert!(ctx.identity.is_none(), "desktop callers leave identity empty");
    }

    #[test]
    fn from_binding_admits_admin() {
        let binding = gated_chat_binding(Channel::Telegram, "chat-1", vec!["42".into()]);
        let ctx = RemoteCommandContext::from_binding(
            Channel::Telegram,
            &binding,
            "42",
            "Max",
            Role::Admin,
            default_runtime_class(),
            "tg:42",
        ).expect("admin admitted");
        let id = ctx.identity.expect("identity present");
        assert_eq!(id.platform, Platform::Telegram);
        assert_eq!(id.user_id, "42");
        assert_eq!(id.channel_id, "chat-1");
        assert_eq!(id.runtime_class, default_runtime_class());
        assert_eq!(id.session_key, "tg:42");
    }

    #[test]
    fn from_binding_rejects_non_admin_with_audit_message() {
        let binding = gated_chat_binding(Channel::Discord, "ch-7", vec!["alice".into()]);
        let err = RemoteCommandContext::from_binding(
            Channel::Discord, &binding,
            "eve", "Eve", Role::Admin,
            default_runtime_class(), "d:7",
        ).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("identity-rejected"), "message must be audit-friendly: {msg}");
        assert!(msg.contains("eve"), "actor id must surface in audit message");
    }

    #[test]
    fn authorize_backwards_compatible_when_identity_none() {
        // Legacy Desktop callers pass identity = None. They should still
        // be gated by the Role::Admin / Role::Viewer policy.
        let admin = RemoteCommandContext::desktop("local", "Max", Role::Admin);
        let cmd = RemoteCommand { kind: CommandKind::Start, map_index: None, config_patch: None, tail: None };
        assert!(authorize(&admin, &cmd).is_ok());

        let viewer = RemoteCommandContext::desktop("local", "Max", Role::Viewer);
        let read = RemoteCommand { kind: CommandKind::Status, map_index: None, config_patch: None, tail: None };
        assert!(authorize(&viewer, &read).is_ok());

        let viewer_write = RemoteCommand { kind: CommandKind::Start, map_index: None, config_patch: None, tail: None };
        assert!(authorize(&viewer, &viewer_write).is_err());
    }

    // ─────────────────────────────────────────────────────────────────────
    // run_with_receipts — pipeline regression tests
    // ─────────────────────────────────────────────────────────────────────

    use crate::receipts::ReceiptLedger;

    /// Build a fresh, isolated emitter backed by a temp-dir ledger. Each
    /// test starts with an empty file so receipts are deterministic.
    fn fresh_emitter() -> (std::sync::Arc<Emitter>,
                           std::path::PathBuf,
                           std::sync::Arc<ReceiptLedger>) {
        let tmp = std::env::temp_dir().join(
            format!("ark-asa-pipeline-{}-{}",
                std::process::id(),
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
        );
        let ledger = std::sync::Arc::new(ReceiptLedger::new(&tmp, "host-test".into()));
        let em = Emitter::new(ledger.clone());
        (std::sync::Arc::new(em), tmp, ledger)
    }

    fn fake_config_get_router() -> impl Fn(RemoteCommandContext, RemoteCommand)
        -> Result<crate::integrations::command_router::RouterOutcome, String> + 'static {
        |_ctx, _cmd| {
            Ok(crate::integrations::command_router::RouterOutcome::Status {
                running: true,
                maps:    vec![],
            })
        }
    }

    #[test]
    fn pipeline_emits_ingress_identitycheck_runtimedelivery_when_admitted() {
        let (emitter, tmp, ledger) = fresh_emitter();
        let admins = vec!["alice".to_string()];
        let binding = gated_chat_binding(Channel::Telegram, "chat-1", admins);

        let remote = RemoteCommand {
            kind: CommandKind::Status,
            map_index: None,
            config_patch: None,
            tail: None,
        };
        let outcome = run_with_receipts(
            Some(&emitter), Platform::Telegram, Channel::Telegram,
            &binding, "alice", "Alice", "trace-1", "/status", None,
            Some("status"), Some(remote), &fake_config_get_router(),
        );
        assert!(matches!(outcome, PipelineOutcome::Done(_)));

        let tail = ledger.tail(10).unwrap();
        // Expect 4 stages: ChannelIngress, IdentityCheck(admitted),
        // RuntimePipeline, ChannelDelivery(delivered)
        assert_eq!(tail.len(), 4, "got stages: {:?}", tail.iter().map(|r|(r.stage.clone(), serde_json::to_string(&r.payload).unwrap())).collect::<Vec<_>>());
        assert_eq!(tail[0].stage, ReceiptStage::ChannelIngress);
        assert_eq!(tail[1].stage, ReceiptStage::IdentityCheck);
        assert_eq!(tail[2].stage, ReceiptStage::RuntimePipeline);
        assert_eq!(tail[3].stage, ReceiptStage::ChannelDelivery);

        // IdentityCheck payload must surface decision=admitted and user_id.
        let idp = &tail[1].payload;
        assert_eq!(idp["decision"], "admitted");
        assert_eq!(idp["identity"]["userId"], "alice");
        assert_eq!(idp["identity"]["platform"], "telegram");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn pipeline_emits_identity_rejected_when_outsider() {
        let (emitter, tmp, ledger) = fresh_emitter();
        let admins = vec!["alice".to_string()];
        let binding = gated_chat_binding(Channel::Discord, "ch-1", admins);

        let remote = RemoteCommand {
            kind: CommandKind::Start,
            map_index: None,
            config_patch: None,
            tail: None,
        };
        let outcome = run_with_receipts(
            Some(&emitter), Platform::Discord, Channel::Discord,
            &binding, "eve", "Eve", "trace-2", "/start", None,
            Some("start"), Some(remote), &fake_config_get_router(),
        );
        assert!(matches!(outcome, PipelineOutcome::Rejected(_)));

        let tail = ledger.tail(10).unwrap();
        // Expect 3 receipts: ChannelIngress, IdentityCheck(rejected), ChannelDelivery(skipped)
        // Note: rejected paths DELIBERATELY do NOT emit RuntimePipeline.
        assert!(tail.len() >= 2, "got tail: {:?}", tail.len());
        assert_eq!(tail[1].stage, ReceiptStage::IdentityCheck);
        assert_eq!(tail[1].payload["decision"], "rejected");
        assert_eq!(tail[1].payload["actorId"], "eve");
        assert_eq!(tail[1].payload["allowlistSize"], 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn pipeline_skips_runtime_when_remote_none_but_logs_ingress() {
        let (emitter, tmp, ledger) = fresh_emitter();
        let binding = gated_chat_binding(Channel::Telegram, "chat-x",
            vec!["alice".into()]);
        let outcome = run_with_receipts(
            Some(&emitter), Platform::Telegram, Channel::Telegram,
            &binding, "alice", "Alice", "trace-3", "hi?", None,
            None, None, &fake_config_get_router(),
        );
        assert!(matches!(outcome, PipelineOutcome::NoCommand(_)));
        let tail = ledger.tail(10).unwrap();
        // Only ChannelIngress and IdentityCheck (no RuntimePipeline, no Delivery)
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[0].stage, ReceiptStage::ChannelIngress);
        assert_eq!(tail[1].stage, ReceiptStage::IdentityCheck);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ─────────────────────────────────────────────────────────────────────
    // run_with_receipts_async — pipeline regression tests
    // ─────────────────────────────────────────────────────────────────────

    /// Mirror of `fake_config_get_router()` but async — the closure body
    /// resolves to a future so the async pipeline can exercise the real
    /// await path (the tokio runtime mock is implicit: `#[tokio::test]`).
    fn fake_async_router() -> impl Fn(RemoteCommandContext, RemoteCommand)
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<RouterOutcome, String>> + Send>>
        + 'static
    {
        |_ctx, _cmd| {
            Box::pin(async {
                Ok(RouterOutcome::Status { running: true, maps: vec![] })
            })
        }
    }

    /// P27 happy path — the async pipeline must emit the same five-stage
    /// trace as the sync pipeline: ChannelIngress → IdentityCheck(admitted)
    /// → RuntimePipeline → ChannelDelivery(delivered).
    #[tokio::test]
    async fn async_pipeline_emits_full_trace_when_admitted() {
        let (emitter, tmp, ledger) = fresh_emitter();
        let admins = vec!["alice".to_string()];
        let binding = gated_chat_binding(Channel::Web, "http-api:convex", admins);

        let remote = RemoteCommand {
            kind: CommandKind::Status,
            map_index: None,
            config_patch: None,
            tail: None,
        };
        let outcome = run_with_receipts_async(
            Some(&emitter), Platform::Web, Channel::Web,
            &binding, "alice", "Alice", "trace-async-1", "/status", None,
            Some("status"), Some(remote), fake_async_router(),
        ).await;
        assert!(matches!(outcome.pipeline, PipelineOutcome::Done(_)));
        assert!(outcome.router_outcome.is_some(), "router outcome must be propagated for HTTP API");

        let tail = ledger.tail(10).unwrap();
        assert_eq!(tail.len(), 4, "got stages: {:?}", tail.iter().map(|r| r.stage.clone()).collect::<Vec<_>>());
        assert_eq!(tail[0].stage, ReceiptStage::ChannelIngress);
        assert_eq!(tail[1].stage, ReceiptStage::IdentityCheck);
        assert_eq!(tail[1].payload["decision"], "admitted");
        assert_eq!(tail[1].payload["identity"]["userId"], "alice");
        assert_eq!(tail[2].stage, ReceiptStage::RuntimePipeline);
        assert_eq!(tail[3].stage, ReceiptStage::ChannelDelivery);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// P27 reject path — non-admin actor must hit `IdentityResolution::Rejected`
    /// and trigger only ChannelIngress → IdentityCheck(rejected) →
    /// ChannelDelivery(skipped). The router must NEVER be invoked.
    #[tokio::test]
    async fn async_pipeline_emits_rejected_when_outsider_no_router() {
        let (emitter, tmp, ledger) = fresh_emitter();
        let admins = vec!["alice".to_string()];
        let binding = gated_chat_binding(Channel::Web, "http-api:convex", admins);

        let remote = RemoteCommand {
            kind: CommandKind::Start,
            map_index: None,
            config_patch: None,
            tail: None,
        };
        let outcome = run_with_receipts_async(
            Some(&emitter), Platform::Web, Channel::Web,
            &binding, "eve", "Eve", "trace-async-2", "/start", None,
            Some("start"), Some(remote),
            |_, _| Box::pin(async { panic!("router must NOT be invoked for rejected actor") }),
        ).await;
        assert!(matches!(outcome.pipeline, PipelineOutcome::Rejected(_)));
        assert!(outcome.router_outcome.is_none(), "rejected path leaks no RouterOutcome");

        let tail = ledger.tail(10).unwrap();
        assert_eq!(tail.len(), 3, "rejected path should emit exactly 3 receipts");
        assert_eq!(tail[0].stage, ReceiptStage::ChannelIngress);
        assert_eq!(tail[1].stage, ReceiptStage::IdentityCheck);
        assert_eq!(tail[1].payload["decision"], "rejected");
        assert_eq!(tail[1].payload["actorId"], "eve");
        assert_eq!(tail[2].stage, ReceiptStage::ChannelDelivery);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// P27 runtime-error path — admitted actor, router returns Err, must emit
    /// RuntimePipeline + ChannelDelivery(Failed).
    #[tokio::test]
    async fn async_pipeline_emits_failure_when_router_errors() {
        let (emitter, tmp, ledger) = fresh_emitter();
        let admins = vec!["alice".to_string()];
        let binding = gated_chat_binding(Channel::Web, "http-api:convex", admins);

        let remote = RemoteCommand {
            kind: CommandKind::Stop,
            map_index: Some(2),
            config_patch: None,
            tail: None,
        };
        let outcome = run_with_receipts_async(
            Some(&emitter), Platform::Web, Channel::Web,
            &binding, "alice", "Alice", "trace-async-3", "/stop 2", None,
            Some("stop"), Some(remote),
            |_ctx, _cmd| Box::pin(async { Err("simulated launcher panic".to_string()) }),
        ).await;
        assert!(matches!(outcome.pipeline, PipelineOutcome::Done(ref s) if s.contains("Router error")));
        assert!(outcome.router_outcome.is_none(), "router error leaks no RouterOutcome");

        let tail = ledger.tail(10).unwrap();
        assert_eq!(tail.len(), 4);
        assert_eq!(tail[2].stage, ReceiptStage::RuntimePipeline);
        assert!(tail[2].payload["outcome"]["error"].as_str().unwrap().contains("Router error"));
        assert_eq!(tail[3].stage, ReceiptStage::ChannelDelivery);
        assert_eq!(tail[3].payload["status"], "failed");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Local alias to give the test readable receipts::Stage without
    // conflicting with the ::command_router::Stage (the inner enums are
    // scoped, but in tests we may want a single short name).
    use crate::receipts::Stage as ReceiptStage;
    use crate::integrations::receipt_emit::Emitter;
}
