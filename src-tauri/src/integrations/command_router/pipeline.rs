//! The receipt-emitting execution pipelines — sync and async.
//!
//! Split out of the original `command_router.rs` (P23) without behaviour or
//! public-API changes — both `run_with_receipts` and `run_with_receipts_async`
//! are re-exported by the parent `command_router` module
//! (`pub use pipeline::*`).

use super::policy::{authorize, default_runtime_class, session_key_from};
use super::types::{
    Channel, PipelineExecution, PipelineOutcome, RemoteCommand, RemoteCommandContext, RouterOutcome,
};
use crate::auth::Role;
use crate::integrations::identity::{ChannelBinding, IdentityResolution, Platform};
use crate::integrations::receipt_emit::DeliveryStatus;

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
