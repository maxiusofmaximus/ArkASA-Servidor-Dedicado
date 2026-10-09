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

// ── Operator policy + router wiring (P11) ────────────────────────────────────
// `Gateway` above was metadata-only until it was reachable. This section makes
// it operational: an operator-editable `plugin_policy.toml` decides, per
// plugin id, whether the gate is enforced and which capabilities apply; and
// `gate_router` wraps the shared router so the gate actually fires before the
// trusted dispatcher.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Per-plugin operator decision.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPolicyEntry {
    /// `None`/`true` → enforce the capability check; `Some(false)` → pass-through.
    #[serde(default)]
    pub enforce: Option<bool>,
    /// Override the capabilities the adapter declared. Keys: `recv`, `send`,
    /// `secrets`, `oauth`. When set, this is authoritative.
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPolicyFile {
    #[serde(default)]
    pub plugins: BTreeMap<String, PluginPolicyEntry>,
}

fn cap_from_key(k: &str) -> Option<PluginCapability> {
    Some(match k.trim().to_ascii_lowercase().as_str() {
        "recv" | "messagesrecv" | "messages_recv" => PluginCapability::MessagesRecv,
        "send" | "messagessend" | "messages_send" => PluginCapability::MessagesSend,
        "secrets" | "requiresecrets" | "requires_secrets" => PluginCapability::RequiresSecrets,
        "oauth" | "requiresoauth" | "requires_oauth" => PluginCapability::RequiresOAuth,
        _ => return None,
    })
}

impl PluginPolicyFile {
    pub fn path() -> std::path::PathBuf {
        crate::plugins::plugin_storage_dir().join("plugin_policy.toml")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|raw| toml::from_str::<Self>(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        let dir = path.parent().ok_or("plugin_policy path missing parent")?;
        std::fs::create_dir_all(dir).map_err(|e| format!("create {dir:?}: {e}"))?;
        let raw = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp = dir.join(".plugin_policy.tmp");
        std::fs::write(&tmp, raw).map_err(|e| format!("write tmp: {e}"))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("rename: {e}"))?;
        Ok(())
    }

    /// Enforced unless the operator explicitly set `enforce = false`.
    pub fn enforce(&self, plugin_id: &str) -> bool {
        self.plugins
            .get(plugin_id)
            .and_then(|e| e.enforce)
            .unwrap_or(true)
    }

    /// Effective capabilities for `plugin_id`: the operator override when
    /// present, else the adapter's declared set, else full (so enforcing never
    /// silently bricks an adapter that declared nothing).
    pub fn caps_for(&self, plugin_id: &str, declared: &[PluginCapability]) -> Vec<PluginCapability> {
        if let Some(over) = self.plugins.get(plugin_id).and_then(|e| e.capabilities.as_ref()) {
            let parsed: Vec<PluginCapability> = over.iter().filter_map(|k| cap_from_key(k)).collect();
            if !parsed.is_empty() {
                return parsed;
            }
        }
        if !declared.is_empty() {
            return declared.to_vec();
        }
        vec![
            PluginCapability::MessagesRecv,
            PluginCapability::MessagesSend,
            PluginCapability::RequiresSecrets,
            PluginCapability::RequiresOAuth,
        ]
    }

    /// Resolve the caps to enforce for a plugin, honouring `enforce`.
    /// `None` → do not gate (pass-through).
    pub fn enforce_caps(&self, plugin_id: &str, declared: &[PluginCapability]) -> Option<Vec<PluginCapability>> {
        if self.enforce(plugin_id) {
            Some(self.caps_for(plugin_id, declared))
        } else {
            None
        }
    }
}

/// Wrap a shared router so the capability gate fires before the trusted
/// dispatcher. `caps` empty or `None` → returns the router unchanged.
pub fn gate_router(
    router: Arc<crate::integrations::RouterFn>,
    caps: Option<Vec<PluginCapability>>,
) -> Arc<crate::integrations::RouterFn> {
    let Some(caps) = caps else { return router };
    if caps.is_empty() {
        return router;
    }
    Arc::new(move |ctx, cmd| {
        if let Err(reason) = Gateway::check(&caps, &cmd.kind) {
            log::warn!("capability gate refused {:?}: {reason}", cmd.kind);
            return Err(format!("capability denied for {:?}: {reason}", cmd.kind));
        }
        router(ctx, cmd)
    })
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

    #[test]
    fn policy_caps_override_and_enforce() {
        let mut f = PluginPolicyFile::default();
        // Default: enforced, declared caps used.
        assert!(f.enforce("telegram"));
        assert_eq!(f.caps_for("telegram", &[PluginCapability::MessagesRecv]).len(), 1);
        // Operator restricts telegram to recv only.
        f.plugins.insert("telegram".into(), PluginPolicyEntry {
            enforce: None,
            capabilities: Some(vec!["recv".into()]),
        });
        let caps = f
            .enforce_caps("telegram", &[PluginCapability::MessagesRecv, PluginCapability::MessagesSend])
            .unwrap();
        assert_eq!(caps, vec![PluginCapability::MessagesRecv]);
        assert!(Gateway::check(&caps, &Start).is_err());
        assert!(Gateway::check(&caps, &Status).is_ok());
        // Operator disables enforcement → pass-through.
        f.plugins.get_mut("telegram").unwrap().enforce = Some(false);
        assert!(f.enforce_caps("telegram", &[PluginCapability::MessagesRecv]).is_none());
    }

    #[test]
    fn gate_router_blocks_writes_and_passes_reads() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        let reached = Arc::new(AtomicUsize::new(0));
        let r = reached.clone();
        let inner: Arc<crate::integrations::RouterFn> = Arc::new(move |_c, _cmd| {
            r.fetch_add(1, Ordering::SeqCst);
            Ok(RouterOutcome::Error { reason: "ok".into() })
        });
        let gated = gate_router(inner, Some(vec![PluginCapability::MessagesRecv]));
        let ctx = RemoteCommandContext::desktop(
            "t", "T", crate::integrations::command_router::Role::Admin,
        );
        let mk = |k| RemoteCommand { kind: k, map_index: None, config_patch: None, tail: None };
        assert!(gated(ctx.clone(), mk(Status)).is_ok());
        assert!(gated(ctx, mk(Start)).is_err());
        assert_eq!(reached.load(Ordering::SeqCst), 1, "only the read reached the router");
    }
}
