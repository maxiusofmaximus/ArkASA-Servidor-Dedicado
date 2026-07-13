//! WeChat adapter — v2.1 skeleton.
//!
//! Inbound arrival: WeChat Work / WeCom bot hook. The wire format is
//! specific XML (`<xml>`) rather than JSON; we deserialize with
//! `serde-xml-rs` semantics but the cargo graph today ships with a
//! plain JSON field-by-field parser so the operator can wire any
//! `wechaty` or `wechat-work` bot SDK and synthesise a
//! `WeChatXmlPayload { from_user, content, msg_type, create_time }`.
//!
//! Outbound is symmetric: a `corpid/corpsecret/access_token` flow
//! returns an `access_token` (cached 2 hours). The actual outbound
//! POST is left to `wechat_bridge.rs::send_wechat_message` because
//! it's straightforward `reqwest` against the WeChat Work API.
//!
//! Backwards rule: ZERO WeChat SDK coupling here. We parse XML as
//! string fields — if you ship a richer parser, it goes in
//! `bridge.rs`.

use crate::integrations::command_router::{
    default_chat_binding, gated_chat_binding, Channel, CommandKind,
    RemoteCommandContext, Role, RouterOutcome,
};
use crate::integrations::RuntimeClass;
use crate::plugins::secret_store_v2 as secret_store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatConfig {
    pub corp_id:           String,    // corpid
    pub corp_secret:       String,    // corpsecret
    pub agent_id:          String,    // application id
    pub admin_user_ids:    String,    // comma-separated WeChat userIds
}

impl WeChatConfig {
    pub fn from_secrets_or_env() -> Self {
        if let Some(s) = secret_store::read("wechat") {
            let get = |k: &str| s.fields.get(k).cloned().unwrap_or_default();
            return Self {
                corp_id:        get("corp_id"),
                corp_secret:    get("corp_secret"),
                agent_id:       get("agent_id"),
                admin_user_ids: get("admin_user_ids"),
            };
        }
        Self {
            corp_id:        std::env::var("WECHAT_CORP_ID").unwrap_or_default(),
            corp_secret:    std::env::var("WECHAT_CORP_SECRET").unwrap_or_default(),
            agent_id:       std::env::var("WECHAT_AGENT_ID").unwrap_or_default(),
            admin_user_ids: std::env::var("WECHAT_ADMIN_USER_IDS").unwrap_or_default(),
        }
    }
    pub fn is_admin(&self, user_id: &str) -> bool {
        self.admin_user_ids.split(',')
            .map(|s| s.trim())
            .any(|s| !s.is_empty() && s == user_id)
    }
}

impl Default for WeChatConfig {
    fn default() -> Self {
        Self {
            corp_id:        String::new(),
            corp_secret:    String::new(),
            agent_id:       String::new(),
            admin_user_ids: String::new(),
        }
    }
}

/// Plain-fields WeChat Work inbound payload. Real WeChat XML is
/// <xml><ToUserName>...</ToUserName>…</xml>; this struct mirrors the
/// relevant subset (the bridge can pre-parse XML into this if it
/// wants). Tests use JSON-shaped fixtures built directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatXmlPayload {
    #[serde(rename = "ToUserName",  default)] pub to_user_name:  String,
    #[serde(rename = "FromUserName",default)] pub from_user_name: String,
    #[serde(rename = "CreateTime",   default)] pub create_time:    String,
    #[serde(rename = "MsgType",      default)] pub msg_type:       String,
    #[serde(rename = "Content",      default)] pub content:        String,
    #[serde(rename = "MsgId",        default)] pub msg_id:         String,
}

pub struct WeChatBot {
    pub cfg: WeChatConfig,
}

impl WeChatBot {
    pub fn new(cfg: WeChatConfig) -> Self { Self { cfg } }

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

    pub fn accept_message(&self, payload: &WeChatXmlPayload) -> Option<RemoteCommandContext> {
        if payload.msg_type.is_empty() || payload.msg_type != "text" { return None; }
        if !self.cfg.is_admin(&payload.from_user_name) { return None; }
        let admins: Vec<String> = self.cfg.admin_user_ids.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()).collect();
        let binding = if admins.is_empty() {
            default_chat_binding(Channel::Wechat, &payload.from_user_name)
        } else {
            gated_chat_binding(Channel::Wechat, &payload.from_user_name, admins)
        };
        match RemoteCommandContext::from_binding(
            Channel::Wechat,
            &binding,
            &payload.from_user_name,
            &format!("wechat-{}", payload.from_user_name),
            Role::Admin,
            RuntimeClass::Interactive,
            &format!("wechat:{}", payload.from_user_name),
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
    id: "wechat",
    label: "WeChat Work",
    channel: crate::plugins::ChannelKind::Wechat,
    capabilities: &[
        crate::plugins::PluginCapability::MessagesRecv,
        crate::plugins::PluginCapability::MessagesSend,
        crate::plugins::PluginCapability::RequiresSecrets,
    ],
    required_secrets: &["corp_id", "corp_secret", "agent_id", "admin_user_ids"],
    oauth_url: None,
};

pub struct WeChatPlugin;

#[async_trait::async_trait]
impl crate::plugins::Plugin for WeChatPlugin {
    fn id() -> &'static str { "wechat" }
    fn descriptor() -> crate::plugins::PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: crate::plugins::PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
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
        let mut cfg = WeChatConfig::default();
        cfg.admin_user_ids = "USER_001 , USER_002".into();
        assert!(cfg.is_admin("USER_001"));
        assert!(cfg.is_admin("USER_002"));
        assert!(!cfg.is_admin("USER_999"));
        cfg.admin_user_ids.clear();
        assert!(!cfg.is_admin("USER_001"));
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
        ];
        for (input, expected) in cases {
            let parsed = WeChatBot::parse_action(input);
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
    fn accept_message_filters_nontext() {
        let mut cfg = WeChatConfig::default();
        cfg.admin_user_ids = "USER_001".into();
        let bot = WeChatBot::new(cfg);
        let mut p = WeChatXmlPayload {
            to_user_name: "AGENT".into(),
            from_user_name: "USER_001".into(),
            create_time: "0".into(),
            msg_type: "text".into(),
            content: "/status".into(),
            msg_id: "wm.0".into(),
        };
        assert!(bot.accept_message(&p).is_some(), "admin + text + slash prefix");

        p.from_user_name = "USER_999".into();
        assert!(bot.accept_message(&p).is_none(), "non-admin rejected");

        p.from_user_name = "USER_001".into();
        p.msg_type = "image".into();
        assert!(bot.accept_message(&p).is_none(), "non-text rejected");
    }

    #[test]
    fn render_outcome_error() {
        let s = WeChatBot::render_outcome(&RouterOutcome::Error {
            reason: "wechat ack failed".into(),
        });
        assert!(s.contains("⚠"));
        assert!(s.contains("wechat ack failed"));
    }

    #[test]
    fn default_config_is_empty_secure() {
        let cfg = WeChatConfig::default();
        for s in [
            cfg.corp_id.as_str(),
            cfg.corp_secret.as_str(),
            cfg.agent_id.as_str(),
            cfg.admin_user_ids.as_str(),
        ] {
            assert!(s.is_empty());
        }
    }
}
