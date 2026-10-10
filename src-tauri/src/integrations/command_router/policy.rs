//! Authorization and channel-policy helpers for the internal command router.
//!
//! Split out of the original `command_router.rs` (P23) without behaviour or
//! public-API changes — every public item here is re-exported by the parent
//! `command_router` module (`pub use policy::*`).

use super::types::{Channel, CommandKind, RemoteCommand, RemoteCommandContext, RouterError};
use crate::auth::Role;
use crate::integrations::identity::{ChannelBinding, Platform, RuntimeClass};

/// Builds a default authorization policy. Admin can do anything. Viewer can
/// only read (`status`, `logs`, `ip`, `config_get`). All write/config_set
/// are restricted to admins. Centralised here so all 8 channel adapters
/// apply the same gate.
pub fn authorize(ctx: &RemoteCommandContext, cmd: &RemoteCommand) -> Result<(), RouterError> {
    use CommandKind::*;
    if ctx.role == Role::Admin { return Ok(()); }
    match cmd.kind {
        Status | Logs | Ip | ConfigGet => Ok(()),
        _ => Err(RouterError::Forbidden(cmd.kind.clone())),
    }
}

/// Maps our router `Channel` to the identity `Platform`. Centralised so
/// receipts and trace logs see a single source of truth.
pub fn platform_for_channel(c: Channel) -> Platform {
    match c {
        Channel::Desktop   => Platform::Desktop,
        Channel::Web       => Platform::Web,
        Channel::Rest      => Platform::Rest,
        Channel::Telegram  => Platform::Telegram,
        Channel::Discord   => Platform::Discord,
        Channel::Slack     => Platform::Slack,
        Channel::Whatsapp  => Platform::WhatsApp,
        Channel::Signal    => Platform::Signal,
        Channel::Wechat    => Platform::Wechat,
        Channel::Ssh       => Platform::Ssh,
    }
}

/// Build a `ChannelBinding` for a single chat thread, sandbox mode (anyone admitted).
pub fn default_chat_binding(channel: Channel, chat_id: &str) -> ChannelBinding {
    ChannelBinding {
        platform:     platform_for_channel(channel),
        account_id:   "@bot".to_string(),
        channel_id:    chat_id.to_string(),
        admin_actors:  None,
        default_agent: "main".to_string(),
    }
}

/// Build a `ChannelBinding` for a single chat thread with an explicit allowlist.
pub fn gated_chat_binding(channel: Channel, chat_id: &str, admins: Vec<String>) -> ChannelBinding {
    ChannelBinding {
        platform:     platform_for_channel(channel),
        account_id:   "@bot".to_string(),
        channel_id:    chat_id.to_string(),
        admin_actors:  Some(admins),
        default_agent: "main".to_string(),
    }
}

/// Default runtime class new adapters should use for human-issued commands.
pub fn default_runtime_class() -> RuntimeClass { RuntimeClass::Interactive }

/// Convenience: extract a session key from an adapter's update ID / message ID.
/// `None` ⇒ caller didn't supply one (we still mint a synthetic key).
pub fn session_key_from(parts: &[&str]) -> String {
    parts.iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(":")
}
