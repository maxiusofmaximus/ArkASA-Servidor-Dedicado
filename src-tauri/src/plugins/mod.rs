//! Plugin trait — the v2.1 way to model Web/Telegram/Discord/WhatsApp/Signal/
//! WeChat/SSH/HTTP as an interchangeable feature module.
//!
//! Each plugin declares:
//!
//!  * `id()`            — stable identifier ("telegram", "discord", ...)
//!  * `capabilities()`  — what RPC verbs the plugin supports
//!  * `requires()`      — which secrets / OAuth flows it needs to start
//!  * `channel()`       — the Channel enum value used in audit logs
//!
//! Plugins are wired in `lib::run()` by reading the TOML
//! `[plugins]` section. Each plugin gets:
//!
//!  * a `Router` (closure) so it can issue commands
//!  * a `Secrets` (key/value map of the credentials the operator set)
//!  * a `PluginContext` (HTTP API client + persistent storage)
//!
//! Following the OpenClaw / Hermes Agent pattern: each external tool is
//! a thin adapter over the **same** `RemoteCommand` pipeline, so we only
//! ever have to test the command router + RBAC. The plugin's job is
//! channel-specific I/O — never to invent new command semantics.

use crate::integrations::command_router::{
    Channel, RemoteCommand, RemoteCommandContext, RouterOutcome,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a plugin can do once wired up.
///
///   - `MessagesRecv`        : can receive inbound messages from a user.
///   - `MessagesSend`        : can send outbound messages to that user.
///   - `RequiresOAuth`       : needs OAuth (the operator clicks "Connect").
///   - `RequiresSecrets`     : needs raw tokens (bot tokens, phone numbers).
///
/// Most messengers require a subset of these two pairs. SSH only has
/// `RequiresSecrets` (public-key list). HTTP REST only has `MessagesRecv`
/// (the operator calls POST themselves).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    MessagesRecv,
    MessagesSend,
    RequiresOAuth,
    RequiresSecrets,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    Web      = 0,
    Rest     = 1,
    Telegram = 2,
    Discord  = 3,
    Whatsapp = 4,
    Signal   = 5,
    Wechat   = 6,
    Ssh      = 7,
}

impl ChannelKind {
    pub fn to_router(self) -> Channel {
        match self {
            ChannelKind::Web      => Channel::Web,
            ChannelKind::Rest     => Channel::Rest,
            ChannelKind::Telegram => Channel::Telegram,
            ChannelKind::Discord  => Channel::Discord,
            ChannelKind::Whatsapp => Channel::Whatsapp,
            ChannelKind::Signal   => Channel::Signal,
            ChannelKind::Wechat   => Channel::Wechat,
            ChannelKind::Ssh      => Channel::Ssh,
        }
    }
}

/// Secrets the operator needs to set before a plugin can start.
///
/// Keys: arbitrary strings consumed by the plugin implementation.
/// Examples:
///   - telegram:  `bot_token`, `admin_chat_ids`
///   - discord :  `bot_token`, `guild_id`, `admin_user_ids`
///   - whatsapp:  `phone_id`, `business_id`, `admin_e164s`, `webhook_secret`
///   - signal  :  `phone_e164`, `admin_e164s` (signal-cli writes these)
///   - wechat  :  `app_id`, `app_secret`, `admin_openids`
///   - ssh     :  `authorized_keys` (multi-line)
///   - http    :  `bearer_tokens` (pipe-separated allowed bearer tokens)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginSecrets(pub BTreeMap<String, String>);

impl PluginSecrets {
    pub fn get(&self, key: &str) -> Option<&str> { self.0.get(key).map(String::as_str) }
    pub fn is_set(&self, key: &str) -> bool { self.0.contains_key(key) }
}

/// A plugin's static descriptor — returned by `id()` / `capabilities()`.
///
/// The Tauri app prints all descovered plugins and their capabilities at
/// startup so operators can `docs/PLUGINS.md` cross-reference what's
/// enabled.
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    pub id:             &'static str,
    pub label:          &'static str,
    pub channel:        ChannelKind,
    pub capabilities:   &'static [PluginCapability],
    pub required_secrets: &'static [&'static str],
    pub oauth_url:      Option<&'static str>, // if RequiresOAuth, the start URL
}

/// (De)serializable carrier for plugin descriptors that cross the HTTP
/// bridge. The static `PluginDescriptor` borrows lifetimes, which serde
/// can't see across the wire; this plain struct flattens it to Strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptorPlain {
    pub id:             String,
    pub label:          String,
    pub channel:        ChannelKind,
    pub capabilities:   Vec<PluginCapability>,
    pub required_secrets: Vec<String>,
    pub oauth_url:      Option<String>,
}

impl From<&'static PluginDescriptor> for PluginDescriptorPlain {
    fn from(d: &'static PluginDescriptor) -> Self {
        Self {
            id:             d.id.to_string(),
            label:          d.label.to_string(),
            channel:        d.channel.clone(),
            capabilities:   d.capabilities.to_vec(),
            required_secrets: d.required_secrets.iter().map(|s| s.to_string()).collect(),
            oauth_url:      d.oauth_url.map(|o| o.to_string()),
        }
    }
}

/// Per-plugin context handed to `start(ctx)`. Includes a typed Router
/// closure so the plugin can dispatch commands without depending on
/// Tauri globals.
#[derive(Clone)]
pub struct PluginContext {
    /// Stable id (= PluginDescriptor.id). Use it for log correlation.
    pub id: &'static str,
    /// Channel the plugin maps to — used when constructing the
    /// audit-log row for `command_log`.
    pub channel_kind: ChannelKind,
    /// Pre-loaded secrets.
    pub secrets: PluginSecrets,
    /// A typed command dispatcher. Plugins call this with the channel-actor
    /// context they derived from the inbound message.
    pub dispatch: std::sync::Arc<dyn Fn(RemoteCommandContext, RemoteCommand) -> Result<RouterOutcome, String> + Send + Sync + 'static>,
}

/// Plugin admission result returned by `start(ctx)`. Failed plugins emit
/// a `MissingSecrets` or `NetworkError` and the rest of the app keeps
/// running (per OpenClaw "fail-soft plugin isolation" principle).
#[derive(Debug, thiserror::Error)]
pub enum PluginStartError {
    #[error("plugin is enabled but missing secrets: {keys:?}")]
    MissingSecrets { id: &'static str, keys: Vec<String> },
    #[error("network error: {0}")]
    Network(String),
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// The plugin contract every adapter implements.
#[async_trait::async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Stable identifier — must match TOML `[plugins.telegram] enabled = true`
    /// style keys (a future Hito 12 PR enables dynamic plugin loading).
    fn id() -> &'static str;
    /// Static metadata. Used by the debug menu / docs autogeneration.
    fn descriptor() -> PluginDescriptor;
    /// Begin running. `start` returns once the plugin has been initiated
    /// but the polling loop continues inside the returned JoinHandle.
    /// When Ok(()) is returned the plugin is healthy; a PluginStartError
    /// is logged and the plugin removed from the active set.
    async fn start(ctx: PluginContext) -> Result<tokio::task::JoinHandle<()>, PluginStartError>;
}

/// Default helper: check that all required secrets are set.
pub fn validate_secrets(
    plugin_id: &'static str,
    required: &[&'static str],
    secrets: &PluginSecrets,
) -> Result<(), PluginStartError> {
    let missing: Vec<String> = required.iter()
        .filter(|k| !secrets.is_set(k))
        .map(|s| (*s).to_string()).collect();
    if !missing.is_empty() {
        return Err(PluginStartError::MissingSecrets { id: plugin_id, keys: missing });
    }
    Ok(())
}

/// Read the plugin secrets section from a ServerConfig-style TOML. Today
/// we use the `[plugins]` table. Tokens are stored in plaintext; for
/// production deployments you may want to encrypt-with-OS-keychain in
/// `Hito 12`.
pub fn secrets_from_toml(table: &toml::Table) -> PluginSecrets {
    let mut out = PluginSecrets::default();
    for (k, v) in table.iter() {
        if let Some(s) = v.as_str() {
            out.0.insert(k.clone(), s.to_string());
        }
    }
    out
}

/// Aggregator exposed to lib.rs. Build it once at startup; each enabled
/// plugin is queried via `start` concurrently.
///
/// Plugins are registered via explicit `register_*` calls (one per plugin
/// module). We don't use `Box<dyn Plugin>` because async-trait + sync-trait
/// make the dyn bound heavy here. Each plugin implements `Plugin` with
/// `async fn start(...)` returning a JoinHandle.
pub struct PluginRegistry {
    pub descriptors: Vec<PluginDescriptor>,
    pub convex: Option<convex::ConvexPlugin>,
    pub vercel: Option<vercel::VercelPlugin>,
    // Hito 7/8/9/10 add their enum variants here:
    // pub telegram: Option<telegram::TelegramPlugin>,
    // pub discord:  Option<discord::DiscordPlugin>,
    // … and so on.
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::with_capacity(8),
            convex: None,
            vercel: None,
        }
    }
}

pub mod secret_store;
pub mod convex;
pub mod vercel;

/// Plugin registration — called once at startup, fills `PluginRegistry`.
pub fn register_default_plugins(reg: &mut PluginRegistry) {
    // Hito 7+: register telegram/discord/whatsapp etc here conditionally on TOML.
    reg.descriptors.push(convex::DESCRIPTOR);
    reg.descriptors.push(vercel::DESCRIPTOR);
}

/**
 * Where per-plugin secrets are persisted on disk.
 *   Linux:  $HOME/.ark-asa/plugins/<plugin>.toml
 *   Win:    %APPDATA%/ark-asa/plugins/<plugin>.toml
 */
pub fn plugin_storage_dir() -> std::path::PathBuf {
    let mut p = crate::auth::AuthState::storage_dir();
    p.push("plugins");
    p
}

