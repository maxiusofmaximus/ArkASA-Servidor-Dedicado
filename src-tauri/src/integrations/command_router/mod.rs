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
//!
//! ## Layout (P23)
//!
//! This module was split into a directory so each concern is addressable on
//! its own, without changing the public API — every item below is re-exported
//! here so `crate::integrations::command_router::X` keeps resolving exactly as
//! before:
//!  - [`types`]    — channels, commands, contexts, outcomes, errors.
//!  - [`policy`]   — authorization gate + channel/binding helpers.
//!  - [`pipeline`] — `run_with_receipts` (sync) and `run_with_receipts_async`.

mod pipeline;
mod policy;
mod types;

pub use pipeline::*;
pub use policy::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::identity::{IdentityResolution, Platform, RejectionReason};

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
