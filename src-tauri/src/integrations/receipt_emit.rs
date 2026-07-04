//! ReceiptEmitter — minimal wrapper around `ReceiptLedger` that bots and
//! bridges call at every pipeline stage.
//!
//! Decoupled from `LEDGER` so it can be cloned cheaply across async tasks.
//! All `emit_*` helpers are infallible from the caller's point of view;
//! ledger errors are logged and dropped (`warn!` only) so a disk failure
//! never blocks a chat-bot reply.
//!
//! Pipeline:
//! ```text
//!   ChannelIngress  →  IdentityCheck  →  [QueueEnqueue]*  →  RuntimePipeline  →  ChannelDelivery
//!                                     * Slack Socket Mode only
//! ```

use crate::integrations::identity::{Identity, Platform, RuntimeClass};
use crate::receipts::{ReceiptLedger, Stage};
use parking_lot::RwLock;
use std::sync::{Arc, OnceLock};

/// Shared emitter for **all** bot/bridge code paths. Initialised in
/// `lib::run()` after the receipts ledger is installed.
static EMITTER: OnceLock<RwLock<Option<Arc<Emitter>>>> = OnceLock::new();

pub(crate) fn install_emitter(ledger: Arc<ReceiptLedger>) {
    let cell = EMITTER.get_or_init(|| RwLock::new(None));
    *cell.write() = Some(Arc::new(Emitter { ledger }));
}

fn shared() -> Option<Arc<Emitter>> {
    EMITTER.get().and_then(|c| c.read().clone())
}

/// Public façade. Cloneable, `Send + Sync`.
#[derive(Clone)]
pub struct Emitter {
    ledger: Arc<ReceiptLedger>,
}

impl Emitter {
    pub fn new(ledger: Arc<ReceiptLedger>) -> Self { Self { ledger } }

    pub fn is_ready(&self) -> bool { true }

    fn emit(&self, stage: Stage, payload: serde_json::Value) {
        if let Err(e) = self.ledger.append(payload, stage) {
            log::warn!("[receipts] ledger write failed at stage={stage:?}: {e}");
        }
    }

    /// 1. Inbound message landed at the bot's transport layer. Always emit.
    pub fn ingress(&self, platform: Platform, ctx: ReceiptContext<'_>, raw_text: &str, language: Option<&str>) {
        let mut p = json_payload(ctx, platform);
        p["rawText"]    = serde_json::Value::String(raw_text.to_string());
        p["textLength"] = serde_json::Value::from(raw_text.chars().count());
        if let Some(lang) = language {
            p["language"] = serde_json::Value::String(lang.to_string());
        }
        self.emit(Stage::ChannelIngress, p);
    }

    /// 2a. Pass: actor admitted by the binding's allow-list.
    pub fn identity_admitted(&self, ctx: ReceiptContext<'_>, platform: Platform, identity: &Identity) {
        let mut p = json_payload(ctx, platform);
        p["decision"] = serde_json::Value::String("admitted".into());
        p["identity"] = identity.snapshot();
        self.emit(Stage::IdentityCheck, p);
    }

    /// 2b. Reject: actor not in the binding's allow-list. Don't proceed
    /// to runtime — the calling adapter must surface the rejection up
    /// the `ChannelDelivery` stage if it sends anything to the user.
    pub fn identity_rejected(
        &self,
        ctx: ReceiptContext<'_>,
        platform: Platform,
        actor_id: &str,
        policy: &str,
        allowlist_size: u32,
    ) {
        let mut p = json_payload(ctx, platform);
        p["decision"]      = serde_json::Value::String("rejected".into());
        p["actorId"]       = serde_json::Value::String(actor_id.to_string());
        p["policy"]        = serde_json::Value::String(policy.to_string());
        p["allowlistSize"] = serde_json::Value::from(allowlist_size as i64);
        self.emit(Stage::IdentityCheck, p);
    }

    /// 3. Slack-only — work has been queued behind an ACK barrier so the
    /// 3-second Socket Mode deadline won't kill us mid-runtime.
    pub fn queue_enqueued(&self, ctx: ReceiptContext<'_>, platform: Platform, envelope_id: &str) {
        let mut p = json_payload(ctx, platform);
        p["envelopeId"] = serde_json::Value::String(envelope_id.to_string());
        self.emit(Stage::QueueEnqueue, p);
    }

    /// 4. Router returned. Capture the outcome (serialised RouterOutcome)
    /// so the replay path can reconstruct operator actions even when the
    /// daemon has restarted.
    pub fn runtime_completed(
        &self,
        ctx: ReceiptContext<'_>,
        platform: Platform,
        kind_label: &str,
        map_index: Option<u32>,
        outcome: &serde_json::Value,
    ) {
        let mut p = json_payload(ctx, platform);
        p["kind"]      = serde_json::Value::String(kind_label.to_string());
        if let Some(i) = map_index {
            p["mapIndex"] = serde_json::Value::from(i);
        }
        p["outcome"]   = outcome.clone();
        self.emit(Stage::RuntimePipeline, p);
    }

    /// 5. Reply was sent (or send failed). Pair with `runtime_completed`
    /// to close the trace.
    pub fn delivery(&self, ctx: ReceiptContext<'_>, platform: Platform, recipient: &str, status: DeliveryStatus, error: Option<&str>) {
        let mut p = json_payload(ctx, platform);
        p["recipient"] = serde_json::Value::String(recipient.to_string());
        p["status"]    = serde_json::Value::String(status.as_str().into());
        if let Some(e) = error {
            p["error"]  = serde_json::Value::String(e.to_string());
        }
        self.emit(Stage::ChannelDelivery, p);
    }

    /// Helper for hosting-stage receipts (separate from chat pipeline).
    pub fn hosting(&self, host_provider: &str, region: &str, ssh_host: &str, disk_gb: u32) {
        self.emit(
            Stage::Hosting,
            serde_json::json!({
                "provider": host_provider,
                "region":   region,
                "sshHost":  ssh_host,
                "diskGb":   disk_gb,
            }),
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeliveryStatus {
    Delivered,
    Failed,
    Skipped,
}

impl DeliveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            DeliveryStatus::Delivered => "delivered",
            DeliveryStatus::Failed    => "failed",
            DeliveryStatus::Skipped   => "skipped",
        }
    }
}

/// Lightweight per-message context shared across the 5 stages of one
/// trace. Everything inside is borrowed; nothing outlives the call.
#[derive(Copy, Clone)]
pub struct ReceiptContext<'a> {
    /// Stable request id correlating all receipts for this single inbound.
    pub trace_id:    &'a str,
    pub actor_id:    &'a str,
    pub actor_name:  &'a str,
    pub runtime:     RuntimeClass,
}

fn json_payload(ctx: ReceiptContext<'_>, platform: Platform) -> serde_json::Value {
    serde_json::json!({
        "traceId":    ctx.trace_id,
        "actorId":    ctx.actor_id,
        "actorName":  ctx.actor_name,
        "runtime":    ctx.runtime.as_str(),
        "platform":   platform.as_str(),
    })
}

/// Convenience — fetch the shared emitter (returns `None` if not yet
/// installed by `lib::run()`). Adapters fall back to silently dropping
/// receipts in that case so a misconfigured host doesn't break chat.
pub fn try_global() -> Option<Arc<Emitter>> { shared() }
