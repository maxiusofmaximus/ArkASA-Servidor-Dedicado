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

/// Aggregator exposed to lib.rs. Built once at startup; the catalog
/// (`Vec<PluginEntry>`) is queried by `start` once the operator
/// enables a plugin through the Plugin Hub UI.
///
/// Why a `Vec` of trait objects? Because the registry has to support
/// dynamic add/remove at runtime (Session 6 / P1): the operator
/// flips a config switch, a new entry shows up in the Vec, and
/// `start()` spins up the matching `Plugin` on demand. Static
/// `Option<ConvexPlugin>` / `Option<VercelPlugin>` doesn't scale
/// past 2-3 hand-picked plugins; the catalog pattern matches the
/// OpenClaw "plugin manifest" pattern.
pub struct PluginRegistry {
    pub descriptors: Vec<PluginDescriptor>,
    /// All known plugins — built-in (convex, vercel) + dynamic (P2/P3 later).
    pub catalog: Vec<PluginEntry>,
    /// Currently-enabled plugin IDs (the rest of catalog is loaded but idle).
    pub enabled: std::collections::BTreeSet<String>,
}

/// A wrapper around any concrete plugin implementing the `Plugin` trait.
/// Catalog uses `Box<dyn AnyPlugin>` so add/remove works without rewriting
/// the registry for each new plugin type. The wrapper exposes a stable
/// interface (id / descriptor / start / stop) via the inner trait object.
pub struct PluginEntry {
    pub id:       &'static str,
    pub descriptor: PluginDescriptor,
    /// The concrete plugin. Calls are dyn-dispatched.
    pub inner:    Box<dyn AnyPlugin>,
}

/// Object-safe trait every plugin implements as `dyn AnyPlugin`. The
/// underlying `Plugin` trait uses associated functions (better safety
/// for `register_default_plugins`), so we add a thin method-based
/// adapter here so the catalog can hold heterogeneous plugin types.
#[async_trait::async_trait]
pub trait AnyPlugin: Send + Sync + 'static {
    /// Returns the stable id (used for catalog lookups).
    fn id(&self) -> &'static str;
    /// Returns the descriptor for the UI.
    fn descriptor(&self) -> PluginDescriptor;
    /// Spawn the plugin loop. Returns JoinHandle for monitoring.
    async fn start(&self, ctx: PluginContext) -> Result<tokio::task::JoinHandle<()>, PluginStartError>;
}

/// Blanket adapter: anything that implements the `Plugin` associated-
/// function trait automatically implements `AnyPlugin` so it can sit
/// in the dyn registry. We delegate method calls to associated calls.
pub struct ConvexPluginAdapter<T: Plugin>(pub T);
impl<T: Plugin> ConvexPluginAdapter<T> {
    pub fn new(inner: T) -> Self { Self(inner) }
}

// Note: `PluginDescriptor` is `Copy`-able (each variant is static-data),
// so cloning out of `DESCRIPTOR` is free. We just hand a fresh copy.
fn descriptor_clone(d: &PluginDescriptor) -> PluginDescriptor {
    PluginDescriptor {
        id:               d.id,
        label:            d.label,
        channel:          d.channel,
        capabilities:     d.capabilities,
        required_secrets: d.required_secrets,
        oauth_url:        d.oauth_url,
    }
}

// Adapt every `Plugin` type (associated functions) to the object-safe
// `AnyPlugin` (methods). Implementation per concrete type — there are
// only two today (Convex, Vercel). To avoid a hand-written adapter per
// type, we use a tiny generic over the descriptor id.
#[async_trait::async_trait]
impl AnyPlugin for convex::ConvexPlugin {
    fn id(&self) -> &'static str { <Self as Plugin>::id() }
    fn descriptor(&self) -> PluginDescriptor {
        descriptor_clone(&<Self as Plugin>::descriptor())
    }
    async fn start(&self, ctx: PluginContext)
        -> Result<tokio::task::JoinHandle<()>, PluginStartError>
    {
        <Self as Plugin>::start(ctx).await
    }
}

#[async_trait::async_trait]
impl AnyPlugin for vercel::VercelPlugin {
    fn id(&self) -> &'static str { <Self as Plugin>::id() }
    fn descriptor(&self) -> PluginDescriptor {
        descriptor_clone(&<Self as Plugin>::descriptor())
    }
    async fn start(&self, ctx: PluginContext)
        -> Result<tokio::task::JoinHandle<()>, PluginStartError>
    {
        <Self as Plugin>::start(ctx).await
    }
}

#[async_trait::async_trait]
impl AnyPlugin for crate::integrations::whatsapp::WhatsAppPlugin {
    fn id(&self) -> &'static str { <Self as Plugin>::id() }
    fn descriptor(&self) -> PluginDescriptor {
        descriptor_clone(&<Self as Plugin>::descriptor())
    }
    async fn start(&self, ctx: PluginContext)
        -> Result<tokio::task::JoinHandle<()>, PluginStartError>
    {
        <Self as Plugin>::start(ctx).await
    }
}

#[async_trait::async_trait]
impl AnyPlugin for crate::integrations::signal::SignalPlugin {
    fn id(&self) -> &'static str { <Self as Plugin>::id() }
    fn descriptor(&self) -> PluginDescriptor {
        descriptor_clone(&<Self as Plugin>::descriptor())
    }
    async fn start(&self, ctx: PluginContext)
        -> Result<tokio::task::JoinHandle<()>, PluginStartError>
    {
        <Self as Plugin>::start(ctx).await
    }
}

#[async_trait::async_trait]
impl AnyPlugin for crate::integrations::wechat::WeChatPlugin {
    fn id(&self) -> &'static str { <Self as Plugin>::id() }
    fn descriptor(&self) -> PluginDescriptor {
        descriptor_clone(&<Self as Plugin>::descriptor())
    }
    async fn start(&self, ctx: PluginContext)
        -> Result<tokio::task::JoinHandle<()>, PluginStartError>
    {
        <Self as Plugin>::start(ctx).await
    }
}

#[async_trait::async_trait]
impl AnyPlugin for crate::integrations::ssh::SshPlugin {
    fn id(&self) -> &'static str { <Self as Plugin>::id() }
    fn descriptor(&self) -> PluginDescriptor {
        descriptor_clone(&<Self as Plugin>::descriptor())
    }
    async fn start(&self, ctx: PluginContext)
        -> Result<tokio::task::JoinHandle<()>, PluginStartError>
    {
        <Self as Plugin>::start(ctx).await
    }
}

#[async_trait::async_trait]
impl AnyPlugin for crate::integrations::rest::RestPlugin {
    fn id(&self) -> &'static str { <Self as Plugin>::id() }
    fn descriptor(&self) -> PluginDescriptor {
        descriptor_clone(&<Self as Plugin>::descriptor())
    }
    async fn start(&self, ctx: PluginContext)
        -> Result<tokio::task::JoinHandle<()>, PluginStartError>
    {
        <Self as Plugin>::start(ctx).await
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::with_capacity(8),
            catalog:     Vec::new(),
            enabled:     std::collections::BTreeSet::new(),
        }
    }

    /// Register a known plugin into the catalog. Idempotent — duplicate
    /// ids are ignored (the operator can't add the same plugin twice).
    pub fn register(&mut self, entry: PluginEntry) {
        if self.catalog.iter().any(|e| e.id == entry.id) {
            return;
        }
        // Capture a clone of the descriptor before moving entry.
        let descriptor = entry.descriptor.clone();
        self.descriptors.push(descriptor);
        self.catalog.push(entry);
    }

    /// Enable by id and start it. Returns the spawned JoinHandle. If the
    /// plugin is already enabled this is a no-op.
    pub async fn enable(
        &mut self,
        id: &str,
        ctx_for: impl Fn(&'static str) -> PluginContext,
    ) -> Result<Option<tokio::task::JoinHandle<()>>, PluginStartError> {
        if self.enabled.contains(id) { return Ok(None); }
        let entry = match self.catalog.iter().find(|e| e.id == id) {
            Some(e) => e,
            None => return Err(PluginStartError::Internal(format!("plugin `{id}` not in catalog"))),
        };
        let ctx = ctx_for(entry.id);
        let h = entry.inner.start(ctx).await?;
        self.enabled.insert(id.to_string());
        Ok(Some(h))
    }

    /// Disable a plugin — we don't currently support stopping an in-flight
    /// JoinHandle because each plugin owns its own shutdown; this just
    /// removes from the enabled set so subsequent `start()` calls won't
    /// spawn it without an explicit re-enable.
    pub fn disable(&mut self, id: &str) {
        self.enabled.remove(id);
    }

    /// Iterate enabled IDs.
    pub fn enabled_ids(&self) -> impl Iterator<Item = &str> {
        self.enabled.iter().map(|s| s.as_str())
    }

    /// Iterate catalog entries (enabled + disabled).
    pub fn catalog_iter(&self) -> impl Iterator<Item = &PluginEntry> {
        self.catalog.iter()
    }

    /// Mark an id as enabled without spawning the actual plugin's
    /// `start(...)` loop. Used by the registry-loader path at startup
    /// to replay the persisted enabled-set without a runnable Tauri
    /// runtime (the runtime hook in `lib::run()` does the actual
    /// spawn once the operator's secrets are present).
    pub fn enable_id_no_start(&mut self, id: &str) {
        if self.catalog.iter().any(|e| e.id == id) {
            self.enabled.insert(id.to_string());
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self { Self::new() }
}

pub mod secret_store_v2;
pub mod convex;
pub mod vercel;
pub mod registry;
pub mod pluginhub;
pub mod connection;
pub mod model;
pub mod whatsapp_bridge;
pub mod extra_bridges;
pub mod runtime_hooks;

/// Plugin registration — called once at startup, fills `PluginRegistry`.
/// Built-in plugins (Convex, Vercel, WhatsApp) get registered here.
/// Operators enable/disable at runtime through the Plugin Hub UI
/// without recompiling.
pub fn register_default_plugins(reg: &mut PluginRegistry) {
    if !reg.catalog.iter().any(|e| e.id == convex::DESCRIPTOR.id) {
        let entry = PluginEntry {
            id:        convex::DESCRIPTOR.id,
            descriptor: convex::DESCRIPTOR,
            inner:     Box::new(convex::ConvexPlugin),
        };
        reg.register(entry);
    }
    if !reg.catalog.iter().any(|e| e.id == vercel::DESCRIPTOR.id) {
        let entry = PluginEntry {
            id:        vercel::DESCRIPTOR.id,
            descriptor: vercel::DESCRIPTOR,
            inner:     Box::new(vercel::VercelPlugin),
        };
        reg.register(entry);
    }
    // WhatsApp Business Cloud (Session 7) — webhook-based inbound.
    if !reg.catalog.iter().any(|e| e.id == "whatsapp") {
        let entry = PluginEntry {
            id:        "whatsapp",
            descriptor: crate::integrations::whatsapp::DESCRIPTOR,
            inner:     Box::new(crate::integrations::whatsapp::WhatsAppPlugin),
        };
        reg.register(entry);
    }
    // Signal (Session 8) — signal-cli JSON daemon.
    if !reg.catalog.iter().any(|e| e.id == "signal") {
        let entry = PluginEntry {
            id:        "signal",
            descriptor: crate::integrations::signal::DESCRIPTOR,
            inner:     Box::new(crate::integrations::signal::SignalPlugin),
        };
        reg.register(entry);
    }
    // WeChat Work (Session 8) — XML webhook relay.
    if !reg.catalog.iter().any(|e| e.id == "wechat") {
        let entry = PluginEntry {
            id:        "wechat",
            descriptor: crate::integrations::wechat::DESCRIPTOR,
            inner:     Box::new(crate::integrations::wechat::WeChatPlugin),
        };
        reg.register(entry);
    }
    // SSH dispatcher (Session 8) — public-key authenticated channels.
    if !reg.catalog.iter().any(|e| e.id == "ssh") {
        let entry = PluginEntry {
            id:        "ssh",
            descriptor: crate::integrations::ssh::DESCRIPTOR,
            inner:     Box::new(crate::integrations::ssh::SshPlugin),
        };
        reg.register(entry);
    }
    // REST/HTTP dispatcher (Session 8) — Bearer-only admin bridge.
    if !reg.catalog.iter().any(|e| e.id == "rest") {
        let entry = PluginEntry {
            id:        "rest",
            descriptor: crate::integrations::rest::DESCRIPTOR,
            inner:     Box::new(crate::integrations::rest::RestPlugin),
        };
        reg.register(entry);
    }
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

// Unit tests share the same on-disk plugin directory and the in-memory
// keyring fallback. Keep those tests in one process-wide critical section so
// migration sweeps cannot observe another test's fixture halfway through.
#[cfg(test)]
static PLUGIN_STORAGE_TEST_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> =
    std::sync::OnceLock::new();

#[cfg(test)]
pub(crate) fn lock_plugin_storage_for_test() -> std::sync::MutexGuard<'static, ()> {
    PLUGIN_STORAGE_TEST_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .expect("plugin storage test lock poisoned")
}

