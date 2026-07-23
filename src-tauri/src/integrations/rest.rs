//! REST/HTTP inbound triggers — v2.1 skeleton.
//!
//! Companion to `ssh.rs`: when the operator prefers cron jobs, REST
//! webhooks, or external script triggers over Telnet/SSL, they hit
//! `POST https://<host>/<endpoint>` with a Bearer token. This
//! module parses those payloads and binds a `RemoteCommandContext`
//! to the actor that holds the Bearer — i.e. real 7-axis identity
//! from existing `auth::AuthState.validate_with_claims`.
//!
//! Wire-shape contract from the existing http_api:
//!   POST /api/v1/start       body { map_index?: u32 }
//!   POST /api/v1/stop        body { map_index?: u32 }
//!   POST /api/v1/restart     body { map_index?: u32 }
//!   POST /api/v1/status
//!   POST /api/v1/logs
//!   POST /api/v1/ip
//!   POST /api/v1/config_get
//!   POST /api/v1/config_set  body { patch: { ... } }
//!
//! This module adds NOTHING to the http_api surface — it merely gives
//! parse_action() / accept_request() pure functions we can test
//! independently, plus a `DESCRIPTOR` so the PluginHub catalog shows
//! `rest-discord`-like siblings. Actuation is purely through,
//! nothing new in 127.0.0.1:8765 — it's already there.

use crate::integrations::command_router::{
    default_chat_binding, Channel, CommandKind, RemoteCommandContext, Role,
    RouterOutcome,
};
use crate::integrations::RuntimeClass;
use crate::plugins::secret_store_v2 as secret_store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestConfig {
    pub auth_required:  bool,         // Bearer required (default true)
    pub allowed_e164s_or_tokens: String, // comma-separated non-empty
}

impl RestConfig {
    pub fn from_secrets_or_env() -> Self {
        if let Some(s) = secret_store::read("rest") {
            let get = |k: &str| s.fields.get(k).cloned().unwrap_or_default();
            return Self {
                auth_required: get("auth_required").parse().unwrap_or(true),
                allowed_e164s_or_tokens: get("allowed_e164s_or_tokens"),
            };
        }
        Self {
            auth_required: std::env::var("REST_AUTH_REQUIRED").ok().map(|v| v != "false").unwrap_or(true),
            allowed_e164s_or_tokens: std::env::var("REST_ALLOWED").unwrap_or_default(),
        }
    }
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            auth_required: true,
            allowed_e164s_or_tokens: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestRequest {
    pub actor_id:    String,    // the actor (sub claim from JWT or operator id)
    pub actor_label: String,    // human-readable
    pub endpoint:    String,
    pub body:        Option<serde_json::Value>,
}

pub struct RestServer {
    pub cfg: RestConfig,
}

impl RestServer {
    pub fn new(cfg: RestConfig) -> Self { Self { cfg } }

    pub fn parse_action(endpoint: &str) -> Option<CommandKind> {
        Some(match endpoint {
            "/api/v1/start"   => CommandKind::Start,
            "/api/v1/stop"    => CommandKind::Stop,
            "/api/v1/restart" => CommandKind::Restart,
            "/api/v1/status"  => CommandKind::Status,
            "/api/v1/logs"    => CommandKind::Logs,
            "/api/v1/ip" => CommandKind::Ip,
            _ => return None,
        })
    }

    pub fn accept_request(&self, req: &RestRequest) -> Option<RemoteCommandContext> {
        if self.cfg.auth_required && req.actor_id.is_empty() { return None; }
        Self::parse_action(&req.endpoint)?;
        let binding = default_chat_binding(Channel::Rest, &req.actor_id);
        match RemoteCommandContext::from_binding(
            Channel::Rest,
            &binding,
            &req.actor_id,
            &req.actor_label,
            Role::Admin,
            RuntimeClass::Interactive,
            &format!("rest:{}", req.actor_id),
        ) {
            Ok(c)  => Some(c),
            Err(_) => None,
        }
    }

    pub fn render_outcome(o: &RouterOutcome) -> String {
        use crate::integrations::command_router::RouterOutcome as R;
        match o {
            R::Started { map, pid }  => format_started(map, *pid),
            R::Stopped { map }       => format_stopped(map),
            R::Restarted { map }     => format!("{map} restarted"),
            R::Status { running, .. }=> if !running { "not running" } else { "running" }.to_string(),
            R::Logs { lines }        => lines.join("\n"),
            R::Ip { .. }             => "see /api/v1/ip".into(),
            R::ConfigGet { toml }    => toml.clone(),
            R::ConfigSet { applied } => format!("applied {applied} entries"),
            R::Error { reason }      => format!("error: {reason}"),
        }
    }
}

// Small shape helpers keep tests easy.
fn format_started(map: &str, pid: u32) -> String { format!("started {map} pid={pid}") }
fn format_stopped(map: &str) -> String { format!("stopped {map}") }

pub const DESCRIPTOR: crate::plugins::PluginDescriptor = crate::plugins::PluginDescriptor {
    id: "rest",
    label: "REST/HTTP inbound",
    channel: crate::plugins::ChannelKind::Rest,
    capabilities: &[
        crate::plugins::PluginCapability::MessagesRecv,
        crate::plugins::PluginCapability::RequiresSecrets,
    ],
    required_secrets: &[", auth_required"],
    oauth_url: None,
};

pub struct RestPlugin;

#[async_trait::async_trait]
impl crate::plugins::Plugin for RestPlugin {
    fn id() -> &'static str { "rest" }
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
    fn parse_action_maps_endpoints() {
        use CommandKind::*;
        let cases = [
            ("/api/v1/start",   Some(Start)),
            ("/api/v1/stop",    Some(Stop)),
            ("/api/v1/restart", Some(Restart)),
            ("/api/v1/status",  Some(Status)),
            ("/api/v1/logs",    Some(Logs)),
            ("/api/v1/ip",      Some(Ip)),
            ("/api/v1/config_get", None),
            ("/",               None),
        ];
        for (input, expected) in cases {
            let parsed = RestServer::parse_action(input);
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
    fn accept_request_requires_actor_when_auth_required() {
        let cfg = RestConfig::default();
        assert!(cfg.auth_required);
        let server = RestServer::new(cfg);
        let req = RestRequest {
            actor_id: "alice".into(),
            actor_label: "Alice".into(),
            endpoint: "/api/v1/start".into(),
            body: None,
        };
        assert!(server.accept_request(&req).is_some());

        let no_actor = RestRequest { actor_id: "".into(), ..req.clone() };
        assert!(server.accept_request(&no_actor).is_none(),
            "auth_required=true + empty actor_id must reject");

        let unknown_endpoint = RestRequest {
            endpoint: "/api/v1/wat".into(),
            ..req.clone()
        };
        assert!(server.accept_request(&unknown_endpoint).is_none(),
            "unknown endpoint must reject");
    }

    #[test]
    fn accept_request_with_disabled_auth_trusts_actor() {
        let mut cfg = RestConfig::default();
        cfg.auth_required = false;
        let server = RestServer::new(cfg);
        let req = RestRequest {
            actor_id: "anyone".into(),
            actor_label: "anon".into(),
            endpoint: "/api/v1/status".into(),
            body: None,
        };
        assert!(server.accept_request(&req).is_some(),
            "auth_required=false admits anyone");
    }

    #[test]
    fn default_config_requires_auth() {
        let cfg = RestConfig::default();
        assert!(cfg.auth_required, "default RestConfig must require auth");
    }

    #[test]
    fn render_outcome_started() {
        let s = RestServer::render_outcome(&RouterOutcome::Started {
            map: "TheIsland".into(),
            pid: 98765,
        });
        assert!(s.contains("started TheIsland"));
        assert!(s.contains("98765"));
    }

    #[test]
    fn render_outcome_error() {
        let s = RestServer::render_outcome(&RouterOutcome::Error {
            reason: "router debug bridge".into(),
        });
        assert!(s.contains("error:"));
        assert!(s.contains("router debug bridge"));
    }
}
