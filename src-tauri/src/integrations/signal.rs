//! Signal adapter — v2.1 skeleton for the session-8 multi-bot expansion.
//!
//! Inbound delivery: Signal Desktop CLI (`signal-cli`), which talks
//! to a real phone via the Signal network and surfaces incoming
//! messages as JSON on stdout. The desktop app is expected to wrap
//! `signal-cli daemon` with `--json` and read from a sidecar stream.
//! Unlike other adapters we DON'T spawn signal-cli here — we
//! provide the parsing, allowlist, and command-routing shape; the
//! operator wires `signal-cli` to it via the
//! General → Cloud Services → Signal bridge.
//!
//! Channel annotation: the actor_id is the source phone E.164, the
//! chat_id is the group/recipient id. Outbound happens via a JsonRpc
//! request to `signal-cli -u <phone_id> send -m "<text>"` — that
//! responsibility belongs to `signal_bridge.rs`'s Tauri commands.
//!
//! Backwards rule: ZERO CLI integration here. The runtime loop is
//! just the parsing shape the existing runtime telemetry expects.

use crate::integrations::command_router::{
    default_chat_binding, gated_chat_binding, Channel, CommandKind,
    RemoteCommandContext, Role, RouterOutcome,
};
use crate::integrations::RuntimeClass;
use crate::plugins::secret_store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfig {
    pub phone_e164:    String,    // the Signal phone registered for this bot
    pub admin_e164s:   String,    // comma-separated E.164 allowlist
    pub signal_cli_bin: String,   // path to signal-cli binary
}

impl SignalConfig {
    pub fn from_secrets_or_env() -> Self {
        if let Some(s) = secret_store::read("signal") {
            let get = |k: &str| s.fields.get(k).cloned().unwrap_or_default();
            return Self {
                phone_e164:     get("phone_e164"),
                admin_e164s:    get("admin_e164s"),
                signal_cli_bin: get("signal_cli_bin"),
            };
        }
        Self {
            phone_e164:     std::env::var("SIGNAL_PHONE").unwrap_or_default(),
            admin_e164s:    std::env::var("SIGNAL_ADMIN_E164S").unwrap_or_default(),
            signal_cli_bin: std::env::var("SIGNAL_CLI_BIN").unwrap_or_default(),
        }
    }

    pub fn is_admin(&self, e164: &str) -> bool {
        self.admin_e164s.split(',')
            .map(|s| s.trim())
            .any(|s| !s.is_empty() && s == e164)
    }
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            phone_e164:     String::new(),
            admin_e164s:    String::new(),
            signal_cli_bin: String::new(),
        }
    }
}

/// JSON-line format emitted by `signal-cli daemon --json`. We only
/// care about the `envelope` shape with `dataMessage` body. Other
/// envelope types (typing, receipt, sync) are silently dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalJsonLine {
    #[serde(default)] pub envelope: Option<SignalEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEnvelope {
    #[serde(rename = "source", default)] pub source: String,
    #[serde(rename = "sourceNumber", default)] pub source_number: String,
    #[serde(rename = "sourceDevice", default)] pub source_device: String,
    #[serde(rename = "timestamp", default)] pub timestamp_ms: u64,
    #[serde(rename = "dataMessage", default)] pub data_message: Option<SignalDataMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDataMessage {
    #[serde(default)] pub message: Option<String>,
    #[serde(default)] pub group_info: Option<SignalGroupInfo>,
    #[serde(default)] pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalGroupInfo {
    #[serde(rename = "groupId", default)] pub group_id: String,
}

pub struct SignalBot {
    pub cfg: SignalConfig,
}

impl SignalBot {
    pub fn new(cfg: SignalConfig) -> Self { Self { cfg } }

    pub fn parse_action(text: &str) -> Option<CommandKind> {
        if !text.starts_with('/') { return None; }
        let mut iter = text.split_whitespace();
        let cmd = iter.next()?.trim().to_ascii_lowercase();
        Some(match cmd.as_str() {
            "/start"   => CommandKind::Start,
            "/stop"    => CommandKind::Stop,
            "/restart" => CommandKind::Restart,
            "/status"  => CommandKind::Status,
            "/logs"    => CommandKind::Logs,
            "/ip"      => CommandKind::Ip,
            _          => return None,
        })
    }

    /// Filter envelope:
    ///  - drop non-data messages
    ///  - require sender E.164 in admin allowlist
    ///  - require group_id if message came from a group, require
    ///    sender-in-group is admin (signal-cli surfaces that)
    pub fn accept_envelope(&self, env: &SignalEnvelope) -> Option<RemoteCommandContext> {
        let data = env.data_message.as_ref()?;
        let text = data.message.as_ref()?.trim();
        if text.is_empty() { return None; }
        if !self.cfg.is_admin(&env.source_number) { return None; }
        let admins: Vec<String> = self.cfg.admin_e164s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()).collect();
        let binding = if admins.is_empty() {
            default_chat_binding(Channel::Signal,
                data.group_info.as_ref().map(|g| g.group_id.as_str())
                    .unwrap_or(&env.source_number))
        } else {
            gated_chat_binding(Channel::Signal,
                data.group_info.as_ref().map(|g| g.group_id.as_str())
                    .unwrap_or(&env.source_number),
                admins)
        };
        match RemoteCommandContext::from_binding(
            Channel::Signal,
            &binding,
            &env.source_number,
            &format!("signal-{}", env.source_number),
            Role::Admin,
            RuntimeClass::Interactive,
            &format!("signal:{}", env.source_number),
        ) {
            Ok(c)  => Some(c),
            Err(_) => None,
        }
    }

    pub fn render_outcome(o: &RouterOutcome) -> String {
        use crate::integrations::command_router::RouterOutcome as R;
        match o {
            R::Started { map, pid }  => format!("✅ {map} started (pid {pid})"),
            R::Stopped { map }       => format!("⏹ {map} stopped"),
            R::Restarted { map }     => format!("♻ {map} restarted"),
            R::Status { running, .. }=> if !running { "⚠ not running".into() } else { "✓ running".into() },
            R::Logs { lines }        => lines.join("\n"),
            R::Ip { .. }             => "(ip payload — connect via /api/v1/ip)".into(),
            R::ConfigGet { toml }    => toml.clone(),
            R::ConfigSet { applied } => format!("✅ applied {applied} entries"),
            R::Error { reason }      => format!("⚠ {reason}"),
        }
    }
}

pub const DESCRIPTOR: crate::plugins::PluginDescriptor = crate::plugins::PluginDescriptor {
    id: "signal",
    label: "Signal (signal-cli)",
    channel: crate::plugins::ChannelKind::Signal,
    capabilities: &[
        crate::plugins::PluginCapability::MessagesRecv,
        crate::plugins::PluginCapability::MessagesSend,
        crate::plugins::PluginCapability::RequiresSecrets,
    ],
    required_secrets: &["phone_e164", "admin_e164s", "signal_cli_bin"],
    oauth_url: None,
};

pub struct SignalPlugin;

#[async_trait::async_trait]
impl crate::plugins::Plugin for SignalPlugin {
    fn id() -> &'static str { "signal" }
    fn descriptor() -> crate::plugins::PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: crate::plugins::PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
        // Same pattern as WhatsApp — the runtime wiring (signal-cli
        // daemon subprocess) lives in main.rs when the plugin is
        // enabled. We return a parked future.
        Ok(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_allowlist_filters() {
        let mut cfg = SignalConfig::default();
        cfg.admin_e164s = "+14155551234 , +14155559876".into();
        assert!(cfg.is_admin("+14155551234"));
        assert!(!cfg.is_admin("+14155550000"));
        cfg.admin_e164s.clear();
        assert!(!cfg.is_admin("+14155551234"));
    }

    #[test]
    fn parse_action_recognises_commands() {
        use CommandKind::*;
        let cases = [
            ("/start",   Some(Start)),
            ("/stop",    Some(Stop)),
            ("/restart", Some(Restart)),
            ("/status",  Some(Status)),
            ("/logs",    Some(Logs)),
            ("/ip",      Some(Ip)),
            ("/unknown", None),
            ("plain",    None),
        ];
        for (input, expected) in cases {
            let parsed = SignalBot::parse_action(input);
            match expected {
                Some(ek) => {
                    let ak = parsed.unwrap_or_else(|| panic!("{input} must parse"));
                    assert_eq!(format!("{ak:?}"), format!("{ek:?}"));
                }
                None => assert!(parsed.is_none(), "{input}"),
            }
        }
    }

    #[test]
    fn accept_envelope_filters_nonadmin() {
        let mut cfg = SignalConfig::default();
        cfg.admin_e164s = "+14155551234".into();
        let bot = SignalBot::new(cfg);

        // Admin data message — accept
        let env = SignalEnvelope {
            source: "12".into(),
            source_number: "+14155551234".into(),
            source_device: "1".into(),
            timestamp_ms: 0,
            data_message: Some(SignalDataMessage {
                message: Some("/status".into()),
                group_info: None,
                timestamp: "0".into(),
            }),
        };
        assert!(bot.accept_envelope(&env).is_some());

        // Non-admin sender — reject
        let bad = SignalEnvelope {
            source_number: "+14155559999".into(),
            ..env.clone()
        };
        assert!(bot.accept_envelope(&bad).is_none());

        // No data_message — reject
        let no_data = SignalEnvelope {
            data_message: None,
            ..env.clone()
        };
        assert!(bot.accept_envelope(&no_data).is_none());
    }

    #[test]
    fn render_outcome_includes_message() {
        let s = SignalBot::render_outcome(&RouterOutcome::Error {
            reason: "signal-cli disconnected".into(),
        });
        assert!(s.contains("⚠"));
        assert!(s.contains("signal-cli disconnected"));
    }

    #[test]
    fn default_config_is_empty_secure() {
        let cfg = SignalConfig::default();
        for s in [
            cfg.phone_e164.as_str(),
            cfg.admin_e164s.as_str(),
            cfg.signal_cli_bin.as_str(),
        ] {
            assert!(s.is_empty());
        }
    }
}
