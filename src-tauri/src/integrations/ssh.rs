//! SSH inbound dispatcher — v2.1 skeleton.
//!
//! The runtime exposes an SSH server on `127.0.0.1:2222` (configured
//! in `hosting.rs` / `network`). Operators upload their public Ed25519
//! key to the trusted-keystore file at `~/.ark-asa/ssh/authorized_keys`.
//! When the operator shells in with `ssh ark@127.0.0.1 -p 2222`,
//! the line is parsed as a slash-command and dispatched to the
//! command_router.
//!
//! This module is the parsing + filtering side. The actual SSH
//! server (we use `russh` as a pure-Rust server; or `sshd` as a
//! sidecar) lives in `lib::run()` because pulling in another
//! crate is out of v2.1 scope. We provide:
//!
//!   - `accept_command(line, allowed_fingerprints)` — pure fn that
//!     checks the command shape and the operator fingerprint allowlist
//!   - `parse_action(line)` — slash-command → CommandKind
//!   - Tests covering parsing + allowlist
//!
//! Until the SSH server hook is wired, the adapter is dormant for
//! runtime; only the tests validate behaviour. Operators use the
//! existing REST/HTTP admin API at `127.0.0.1:8765` in the meantime.

use crate::integrations::command_router::{
    default_chat_binding, Channel, CommandKind, RemoteCommandContext, Role,
    RouterOutcome,
};
use crate::integrations::RuntimeClass;
use crate::plugins::secret_store_v2 as secret_store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub listen_port:        u16,         // default 2222
    pub allowed_fingerprints: String,     // comma-separated ed25519 sha256 fingerprints
}

impl SshConfig {
    pub fn from_secrets_or_env() -> Self {
        if let Some(s) = secret_store::read("ssh") {
            let get = |k: &str| s.fields.get(k).cloned().unwrap_or_default();
            return Self {
                listen_port: get("listen_port").parse().unwrap_or(2222),
                allowed_fingerprints: get("allowed_fingerprints"),
            };
        }
        Self {
            listen_port: std::env::var("ARK_SSH_PORT").ok()
                .and_then(|v| v.parse().ok()).unwrap_or(2222),
            allowed_fingerprints: std::env::var("ARK_SSH_FPR").unwrap_or_default(),
        }
    }

    pub fn fingerprint_allowed(&self, fp: &str) -> bool {
        self.allowed_fingerprints.split(',')
            .map(|s| s.trim())
            .any(|s| !s.is_empty() && s == fp)
    }
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            listen_port: 2222,
            allowed_fingerprints: String::new(),
        }
    }
}

/// One inbound SSH command line. Optional fingerprint is the
/// client's public-key fingerprint (we wire this from russh's
/// `AuthInfo` callback in `lib::run()`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshCommandRequest {
    pub fingerprint: String,
    pub command_line: String,
}

pub struct SshServer {
    pub cfg: SshConfig,
}

impl SshServer {
    pub fn new(cfg: SshConfig) -> Self { Self { cfg } }

    pub fn parse_action(line: &str) -> Option<CommandKind> {
        // SSH channel lines don't go through "/" (operators just type
        // `start` or `stop server1`). Split on whitespace.
        let first = line.split_whitespace().next()?.trim();
        CommandKind::parse_slash(first)
    }

    pub fn accept_request(&self, req: &SshCommandRequest) -> Option<RemoteCommandContext> {
        if !self.cfg.fingerprint_allowed(&req.fingerprint) { return None; }
        Self::parse_action(&req.command_line)?;
        let binding = default_chat_binding(Channel::Ssh, &req.fingerprint);
        match RemoteCommandContext::from_binding(
            Channel::Ssh,
            &binding,
            &req.fingerprint,
            &format!("ssh-{}", &req.fingerprint[..8]),
            Role::Admin,
            RuntimeClass::Interactive,
            &format!("ssh:{}", req.fingerprint),
        ) {
            Ok(c)  => Some(c),
            Err(_) => None,
        }
    }

    pub fn render_outcome(o: &RouterOutcome) -> String {
        use crate::integrations::command_router::RouterOutcome as R;
        match o {
            R::Started { map, pid }  => format!("{map} started (pid {pid})"),
            R::Stopped { map }       => format!("{map} stopped"),
            R::Restarted { map }     => format!("{map} restarted"),
            R::Status { running, .. }=> if !running { "not running".into() } else { "running".into() },
            R::Logs { lines }        => lines.join("\n"),
            R::Ip { .. }             => "(ip payload)".into(),
            R::ConfigGet { toml }    => toml.clone(),
            R::ConfigSet { applied } => format!("applied {applied} entries"),
            R::Error { reason }      => format!("ERROR: {reason}"),
        }
    }
}

pub const DESCRIPTOR: crate::plugins::PluginDescriptor = crate::plugins::PluginDescriptor {
    id: "ssh",
    label: "SSH inbound dispatcher",
    channel: crate::plugins::ChannelKind::Ssh,
    capabilities: &[
        crate::plugins::PluginCapability::MessagesRecv,
        crate::plugins::PluginCapability::RequiresSecrets,
    ],
    required_secrets: &["listen_port", "allowed_fingerprints"],
    oauth_url: None,
};

pub struct SshPlugin;

#[async_trait::async_trait]
impl crate::plugins::Plugin for SshPlugin {
    fn id() -> &'static str { "ssh" }
    fn descriptor() -> crate::plugins::PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: crate::plugins::PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
        // Runtime hook (russh / sshd) lives in lib::run(). We park
        // here so the registry sees it as a running plugin.
        Ok(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_check_works() {
        let mut cfg = SshConfig::default();
        cfg.allowed_fingerprints = "SHA256:abc,SHA256:def".into();
        assert!(cfg.fingerprint_allowed("SHA256:abc"));
        assert!(cfg.fingerprint_allowed("SHA256:def"));
        assert!(!cfg.fingerprint_allowed("SHA256:xyz"));
        cfg.allowed_fingerprints.clear();
        assert!(!cfg.fingerprint_allowed("SHA256:abc"));
    }

    #[test]
    fn parse_action_unprefixed() {
        use CommandKind::*;
        let cases = [
            ("start",   Some(Start)),
            ("stop",    Some(Stop)),
            ("restart", Some(Restart)),
            ("status",  Some(Status)),
            ("logs",    Some(Logs)),
            ("ip",      Some(Ip)),
            ("unknown", None),
            ("",        None),
        ];
        for (input, expected) in cases {
            let parsed = SshServer::parse_action(input);
            match expected {
                Some(ek) => {
                    let ak = parsed.unwrap_or_else(|| panic!("{input} must parse"));
                    assert_eq!(format!("{ak:?}"), format!("{ek:?}"));
                }
                None => assert!(parsed.is_none()),
            }
        }
    }

    #[test]
    fn accept_request_filters_fingerprint() {
        let mut cfg = SshConfig::default();
        cfg.allowed_fingerprints = "SHA256:abc".into();
        let server = SshServer::new(cfg);
        let allow = SshCommandRequest {
            fingerprint: "SHA256:abc".into(),
            command_line: "start server1".into(),
        };
        assert!(server.accept_request(&allow).is_some());

        let denied = SshCommandRequest {
            fingerprint: "SHA256:xyz".into(),
            command_line: "start server1".into(),
        };
        assert!(server.accept_request(&denied).is_none());

        let unknown = SshCommandRequest {
            fingerprint: "SHA256:abc".into(),
            command_line: "unknown-thing".into(),
        };
        assert!(server.accept_request(&unknown).is_none());
    }

    #[test]
    fn default_config_uses_port_2222() {
        let cfg = SshConfig::default();
        assert_eq!(cfg.listen_port, 2222);
    }

    #[test]
    fn render_outcome_error_visible() {
        let s = SshServer::render_outcome(&RouterOutcome::Error {
            reason: "SSH dispatcher net stack down".into(),
        });
        assert!(s.contains("ERROR:"));
        assert!(s.contains("SSH dispatcher net stack down"));
    }
}
