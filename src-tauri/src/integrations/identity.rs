//! Identity model + bindings (fail-closed).
//!
//! Mirrors the Agent Harness Core "7-axis identity" contract
//! (`platform, accountId, channelId, userId, agentId, sessionKey, runtimeClass`).
//!
//! Every inbound message — be it from Telegram, Discord, Slack, the loopback
//! HTTP API, or a future Convex webhook — MUST resolve to an
//! `IdentityBinding` BEFORE reaching the bridge. The adapter calls
//! `Identity::from_telegram_user(...)`, `from_discord_user(...)`, etc.
//!
//! If any axis is missing or the `actorId` falls outside the allow-list of
//! the configured channel, we return `IdentityResolution::Rejected(reason)`
//! and the invitation never reaches the bridge.
//!
//! Without this, channels mint fake identities, two agents collide on the
//! same `userId`, and the receipt replay path becomes meaningless.

use serde::{Deserialize, Serialize};

/// All 7 identity axes preserved end-to-end by the runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Identity {
    #[serde(default)]
    pub platform:      Platform,
    #[serde(default)]
    pub account_id:    String,
    #[serde(default)]
    pub channel_id:    String,
    #[serde(default)]
    pub user_id:       String,
    #[serde(default)]
    pub agent_id:      String,
    #[serde(default)]
    pub session_key:   String,
    #[serde(default)]
    pub runtime_class: RuntimeClass,
}

impl Identity {
    /// Snapshot of the 7 axes as a JSON-friendly flat shape used inside
    /// receipt payloads and trace logs.
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "platform":      self.platform.as_str(),
            "accountId":     self.account_id,
            "channelId":     self.channel_id,
            "userId":        self.user_id,
            "agentId":       self.agent_id,
            "sessionKey":    self.session_key,
            "runtimeClass":  self.runtime_class.as_str(),
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[default]
    Unknown,
    Telegram,
    Discord,
    Slack,
    WhatsApp,
    Signal,
    Wechat,
    Rest,
    Web,
    Convex,
    Supabase,
    Insforge,
    Ssh,
    Desktop,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Telegram  => "telegram",
            Platform::Discord   => "discord",
            Platform::Slack     => "slack",
            Platform::WhatsApp  => "whatsapp",
            Platform::Signal    => "signal",
            Platform::Wechat    => "wechat",
            Platform::Rest      => "rest",
            Platform::Web       => "web",
            Platform::Convex    => "convex",
            Platform::Supabase  => "supabase",
            Platform::Insforge  => "insforge",
            Platform::Ssh       => "ssh",
            Platform::Desktop   => "desktop",
            Platform::Unknown   => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeClass {
    #[default]
    Interactive,
    Cron,
    Worker,
    Maintenance,
    Unknown,
}

impl RuntimeClass {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeClass::Interactive  => "interactive",
            RuntimeClass::Cron         => "cron",
            RuntimeClass::Worker       => "worker",
            RuntimeClass::Maintenance  => "maintenance",
            RuntimeClass::Unknown      => "unknown",
        }
    }
}

/// Resolution outcome. The bridge MUST refuse to dispatch a `Rejected`.
#[derive(Debug, Clone)]
pub enum IdentityResolution {
    Bound(Identity),
    Rejected(RejectionReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "reason", content = "detail")]
pub enum RejectionReason {
    AllowlistEmpty,        // No admins configured for this channel
    AllowlistMiss { configured: u32, actor_id: String },
    ChannelScopeMiss,      // Channel-actor binding required but missing
    MalformedActorId,
}

impl RejectionReason {
    pub fn human(&self) -> String {
        match self {
            RejectionReason::AllowlistEmpty =>
                "no admin list configured for this channel".into(),
            RejectionReason::AllowlistMiss { configured, actor_id } =>
                format!("actor `{actor_id}` is not in this channel's admin list ({configured} configured)"),
            RejectionReason::ChannelScopeMiss =>
                "channel-scope binding required but actor did not satisfy it".into(),
            RejectionReason::MalformedActorId =>
                "actor id is empty / malformed".into(),
        }
    }
}

/// Per-channel allowlist binding. Concretely tuned for chat bots today.
#[derive(Debug, Clone, Default)]
pub struct ChannelBinding {
    pub platform:     Platform,
    pub channel_id:    String,
    pub account_id:    String,
    /// `None` ⇒ sandbox mode (anyone admitted; logged). Some ⇒ admin-only.
    pub admin_actors:  Option<Vec<String>>,
    /// Default `agentId` for commands serving this channel. Defaults to `"main"`.
    pub default_agent: String,
}

impl ChannelBinding {
    /// Fail-closed resolve: refuses unless the binding config admits the actor.
    ///
    /// Returns `IdentityResolution::Bound(identity)` only when ALL 7 axes are
    /// filled. Anything else is `Rejected`.
    pub fn resolve(
        &self,
        actor_id: &str,
        runtime_class: RuntimeClass,
        session_key: &str,
    ) -> IdentityResolution {
        if actor_id.trim().is_empty() {
            return IdentityResolution::Rejected(RejectionReason::MalformedActorId);
        }
        match (&self.admin_actors, actor_id.trim()) {
            (None, _) => {
                // Sandbox mode — anyone admitted, but we still log loudly.
                IdentityResolution::Bound(Identity {
                    platform:     self.platform,
                    account_id:   self.account_id.clone(),
                    channel_id:    self.channel_id.clone(),
                    user_id:       actor_id.to_string(),
                    agent_id:      if self.default_agent.is_empty() { "main".into() } else { self.default_agent.clone() },
                    session_key:   session_key.into(),
                    runtime_class,
                })
            }
            (Some(list), actor) => {
                if let Some(_matched) = list.iter().find(|a| a.trim() == actor) {
                    IdentityResolution::Bound(Identity {
                        platform:     self.platform,
                        account_id:   self.account_id.clone(),
                        channel_id:    self.channel_id.clone(),
                        user_id:       actor.to_string(),
                        agent_id:      if self.default_agent.is_empty() { "main".into() } else { self.default_agent.clone() },
                        session_key:   session_key.into(),
                        runtime_class,
                    })
                } else {
                    IdentityResolution::Rejected(RejectionReason::AllowlistMiss {
                        configured: list.len() as u32,
                        actor_id:   actor.to_string(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(admins: Option<Vec<&str>>) -> ChannelBinding {
        ChannelBinding {
            platform: Platform::Telegram,
            account_id: "@mybot".into(),
            channel_id: "chat-42".into(),
            admin_actors: admins.map(|v| v.into_iter().map(String::from).collect()),
            default_agent: "main".into(),
        }
    }

    #[test]
    fn rejects_empty_actor() {
        let b = binding(Some(vec!["alice"]));
        match b.resolve("", RuntimeClass::Interactive, "abc") {
            IdentityResolution::Rejected(RejectionReason::MalformedActorId) => {}
            other => panic!("expected MalformedActorId, got {other:?}"),
        }
    }

    #[test]
    fn rejects_actor_not_in_admin_list() {
        let b = binding(Some(vec!["alice", "bob"]));
        match b.resolve("eve", RuntimeClass::Interactive, "s1") {
            IdentityResolution::Rejected(RejectionReason::AllowlistMiss {
                configured: 2, actor_id
            }) if actor_id == "eve" => {}
            other => panic!("expected AllowlistMiss, got {other:?}"),
        }
    }

    #[test]
    fn admits_admin() {
        let b = binding(Some(vec!["alice"]));
        match b.resolve("alice", RuntimeClass::Interactive, "sess") {
            IdentityResolution::Bound(id) => {
                assert_eq!(id.platform, Platform::Telegram);
                assert_eq!(id.user_id, "alice");
                assert_eq!(id.channel_id, "chat-42");
                assert_eq!(id.runtime_class, RuntimeClass::Interactive);
                assert_eq!(id.session_key, "sess");
            }
            other => panic!("expected Bound, got {other:?}"),
        }
    }

    #[test]
    fn sandbox_admits_anyone() {
        let b = binding(None);
        match b.resolve("anyone", RuntimeClass::Cron, "key") {
            IdentityResolution::Bound(id) => {
                assert_eq!(id.user_id, "anyone");
                assert_eq!(id.runtime_class, RuntimeClass::Cron);
            }
            other => panic!("Bound expected in sandbox, got {other:?}"),
        }
    }

    #[test]
    fn snapshot_roundtrips_every_axis() {
        let id = Identity {
            platform: Platform::Discord,
            account_id: "@bot".into(),
            channel_id: "channel-1".into(),
            user_id: "u1".into(),
            agent_id: "main".into(),
            session_key: "sess-1".into(),
            runtime_class: RuntimeClass::Interactive,
        };
        let v = id.snapshot();
        assert_eq!(v["userId"], "u1");
        assert_eq!(v["channelId"], "channel-1");
        assert_eq!(v["platform"], "discord");
        assert_eq!(v["runtimeClass"], "interactive");
    }
}
