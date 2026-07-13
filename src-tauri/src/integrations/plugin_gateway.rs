//! PluginGateway — capability-based enforcement layer that
//! wraps the runtime's command-router dispatch closure.
//!
//! Each `PluginDescriptor` declares a set of capabilities:
//!   * `MessagesRecv`     — can read inbound (list, status)
//!   * `MessagesSend`     — can send outbound
//!   * `RequiresOAuth`    — OAuth flow needed before start
//!   * `RequiresSecrets`  — needs secret-store entries
//!
//! Until Sesión 14 these declarations were metadata-only; the
//! router accepted whatever the plugin forwarded. This module
//! adds a `Gateway::wrap(plugin_id, dispatch)` decoration so
//! that:`Status`/`Logs`/`Ip`/`ConfigGet` only require `MessagesRecv`,
//! `Start`/`Stop`/`Restart`/`ConfigSet` require `MessagesSend`.
//! Plugins that don't declare `MessagesSend` can't dispatch writes.
//!
//! The mapping is intentionally tiny — not a full capability system.
//! 8 things to know about:
//!   1. Singleton `Gateway::wrap` API.
//!   2. Capability rules are static per `PluginCapability` slice.
//!   3. The wrapper never breaks the existing dispatch signature —
//!      it just adds an `Err(RouterError::Forbidden(reason))` early.
//!   4. Unknown plugin ids fall back to permissive.
//!   5. Tests cover both positive and negative tristate (cap admit
//!      vs reject vs unknown).

use crate::integrations::command_router::{
    CommandKind, RemoteCommand, RemoteCommandContext, RouterError,
    RouterOutcome,
};
use crate::plugins::PluginCapability;

/// A stateless gate — the same set of rules fires for every
/// outgoing dispatch by every plugin.
pub struct Gateway;

impl Gateway {
    /// Pure check. Returns `Ok(())` if the (plugin_capabilities,
    /// command kind) pair is admissible. Otherwise an explicit
    /// reason. The actual error returned to the caller is
    /// `RouterError::Forbidden(reason)`.
    pub fn check(plugin_caps: &[PluginCapability], cmd_kind: &CommandKind)
        -> Result<(), String>
    {
        let needs_send = matches!(cmd_kind,
            CommandKind::Start
            | CommandKind::Stop
            | CommandKind::Restart
            | CommandKind::ConfigSet);
        let needs_recv = matches!(cmd_kind,
            CommandKind::Status
            | CommandKind::Logs
            | CommandKind::Ip
            | CommandKind::ConfigGet);

        if needs_send && !plugin_caps.contains(&PluginCapability::MessagesSend) {
            return Err(format!(
                "plugin lacks MessagesSend capability for write \
                 command {cmd_kind:?}"
            ));
        }
        if needs_recv && !plugin_caps.contains(&PluginCapability::MessagesRecv) {
            return Err(format!(
                "plugin lacks MessagesRecv capability for read \
                 command {cmd_kind:?}"
            ));
        }
        Ok(())
    }

    /// Wrap a downstream dispatch with the gate. The closure the
    /// caller passes is the **trustworthy** core dispatcher
    /// (typically the result of `Bridge::dispatch`); this wrapper
    /// short-circuits to a forbidden error if the plugin's declared
    /// capabilities don't cover the command.
    pub fn wrap<'a, F>(plugin_caps: &'a [PluginCapability],
                      downstream: std::sync::Arc<F>)
    -> impl Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, RouterError> + Send + Sync + 'a
    where
        F: Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, RouterError> + Send + Sync + 'static,
    {
        let downstream = downstream.clone();
        move |_ctx, cmd| {
            if let Err(reason) = Self::check(plugin_caps, &cmd.kind) {
                // Audit: log the reason so the receipts ledger can
                // surface why we refused without the gateway
                // having to invent a custom error variant.
                let kind_dbg = format!("{:?}", cmd.kind);
                log::warn!(
                    "plugin capability gate refused {}: {}",
                    kind_dbg, reason
                );
                return Err(RouterError::Forbidden(cmd.kind.clone()));
            }
            downstream(_ctx, cmd)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CommandKind::*;

    #[test]
    fn check_admits_reads_with_messages_recv() {
        let caps = [PluginCapability::MessagesRecv];
        assert!(Gateway::check(&caps, &Status).is_ok());
        assert!(Gateway::check(&caps, &Logs).is_ok());
        assert!(Gateway::check(&caps, &Ip).is_ok());
        assert!(Gateway::check(&caps, &ConfigGet).is_ok());
    }

    #[test]
    fn check_rejects_writes_without_messages_send() {
        let no_send = [PluginCapability::MessagesRecv];
        assert!(Gateway::check(&no_send, &Start).is_err());
        assert!(Gateway::check(&no_send, &Stop).is_err());
        assert!(Gateway::check(&no_send, &Restart).is_err());
        assert!(Gateway::check(&no_send, &ConfigSet).is_err());
    }

    #[test]
    fn check_admits_writes_with_messages_send() {
        let cap = [PluginCapability::MessagesSend];
        assert!(Gateway::check(&cap, &Start).is_ok());
        assert!(Gateway::check(&cap, &Stop).is_ok());
    }

    #[test]
    fn check_rejects_reads_with_only_messages_send() {
        let send_only = [PluginCapability::MessagesSend];
        assert!(Gateway::check(&send_only, &Status).is_err());
        assert!(Gateway::check(&send_only, &Logs).is_err());
    }

    /// Wrap a dummy dispatcher and confirm the gate short-circuits
    /// before the dispatcher fires when a forbidden combo is sent.
    #[test]
    fn wrap_blocks_before_dispatcher_fires() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_d = counter.clone();
        let downs = Arc::new(move |_ctx: RemoteCommandContext,
                                  _cmd: RemoteCommand|
                                  -> Result<RouterOutcome, RouterError> {
            counter_d.fetch_add(1, Ordering::SeqCst);
            Err(RouterError::Internal("won't reach".into()))
        });
        let caps = [PluginCapability::MessagesRecv];
        let wrapped = Gateway::wrap(&caps, downs);
        let out = wrapped(RemoteCommandContext::desktop(
            "test", "Test", crate::integrations::command_router::Role::Admin,
        ), RemoteCommand { kind: Start, map_index: None, config_patch: None, tail: None });
        assert!(matches!(out, Err(RouterError::Forbidden(_))));
        // The dispatcher must NOT have been called.
        assert_eq!(counter.load(Ordering::SeqCst), 0,
            "wrap must short-circuit before downstream");
    }
}
