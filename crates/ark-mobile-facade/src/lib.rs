//! UniFFI facade over [`ark_core`].
//!
//! This crate exists for exactly one reason: to be the **only** place in the
//! workspace carrying `#[uniffi::export]`, so the Kotlin and Swift bindings are
//! generated from one definition. It contains no business logic — every call
//! delegates straight into `ark-core`.
//!
//! Contract:
//! - Keep it thin. If a rule needs implementing, it belongs in `ark-core`.
//! - Types crossing the FFI boundary are owned `record`s or `Arc`-wrapped
//!   objects; `ark-core` types are converted at the edge so `ark-core` stays
//!   free of any FFI knowledge.
//! - Read paths first, then writes. No `async` until the sync path is proven.
//!
//! Verification on a developer machine does **not** require the Android SDK:
//! UniFFI's Kotlin output is plain JVM code and JNA can load the Windows
//! `.dll` built here, so the bridge is exercised by host-JVM tests.

use std::sync::Arc;

uniffi::setup_scaffolding!();

/// A server profile as seen by the mobile app.
///
/// A plain owned record, deliberately decoupled from
/// [`ark_core::config::ServerProfile`] so the core's type can evolve without
/// breaking the published Kotlin/Swift API.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ProfileView {
    /// Stable, filesystem-safe identifier (also the `servers/<id>/` folder).
    pub id: String,
    /// Human label shown in the UI.
    pub name: String,
    /// Absolute path to this profile's `server-config.toml`.
    pub config_path: String,
    /// Creation time (Unix seconds).
    pub created_at_unix: u64,
    /// Whether this is the profile the app is currently pointed at.
    pub is_active: bool,
}

/// A server's configuration, projected down to the fields an operator edits
/// from a phone.
///
/// Deliberately **not** the whole `ServerConfig`: that type has sixteen sections
/// and hundreds of fields, most of which (path layouts, plugin registries,
/// per-level stat curves) are desktop concerns that no one will type on a touch
/// keyboard. Exposing a narrow record keeps the Kotlin side honest — there is no
/// chance of silently dropping a field, because fields that must survive an edit
/// are listed here and only here.
///
/// Round-tripping is safe: `save_config` loads the full file, overwrites only
/// the fields present in this record, and writes the rest back untouched.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ConfigView {
    /// Session name players see in the server browser.
    pub session_name: String,
    /// Password that lets someone join the server at all.
    pub server_password: String,
    /// RCON password, also used by the desktop app's console.
    pub admin_password: String,
    /// Message of the day, shown to players on join.
    pub motd: String,
    /// Game port (UDP).
    pub port: u16,
    /// Query port (Steam server browser).
    pub query_port: u16,
    /// RCON port (TCP).
    pub rcon_port: u16,
    /// Maximum concurrent players.
    pub max_players: u16,
    /// Maps in this server/cluster, in launch order.
    pub cluster_maps: Vec<String>,
    /// Whether anti-cheat is disabled (`-NoBattlEye`).
    pub no_battleye: bool,
}

/// One validation complaint, as reported by the core's validators.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ValidationIssueView {
    /// Which field the validator objected to.
    pub field: String,
    /// Human-readable explanation.
    pub message: String,
    /// Stable machine-readable code, when the validator provides one.
    pub code: Option<String>,
}

/// Where the app should dial to reach a running server's console.
///
/// Resolved from the full configuration on the Rust side on purpose: which
/// connection entry is primary, whether it is a tunnel, and how that combines
/// with the RCON port are all config rules, and duplicating them in Kotlin would
/// be the same class of drift this project keeps avoiding. The app receives a
/// ready address and just opens a socket.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ConnectionTarget {
    /// Hostname or IP to dial (a tunnel address, a Tailscale IP, a LAN IP…).
    pub host: String,
    /// TCP port to dial.
    pub port: u16,
    /// The label of the connection entry used, for display.
    pub label: String,
}
///
/// The field is called `detail`, not `message`: UniFFI generates each variant
/// as a Kotlin exception subclass, and a field named `message` would collide
/// with `Throwable.message` and fail to compile.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FacadeError {
    /// The configured directory could not be read or is not a directory.
    #[error("profile store error: {detail}")]
    ProfileStore { detail: String },
}

impl From<ark_core::Error> for FacadeError {
    fn from(e: ark_core::Error) -> Self {
        FacadeError::ProfileStore {
            detail: e.to_string(),
        }
    }
}

/// The entry point handed to the foreign language.
///
/// Constructing it pins the config directory; every subsequent call reads from
/// that directory. It holds no mutable shared state yet, which keeps the
/// `Send + Sync` contract trivially satisfied — the property UniFFI is strict
/// about for shared objects.
#[derive(uniffi::Object)]
pub struct MobileCore {
    config_dir: std::path::PathBuf,
}

#[uniffi::export]
impl MobileCore {
    /// Open the mobile core against an explicit configuration directory.
    ///
    /// Android callers pass `context.filesDir`; iOS callers pass the app
    /// container's Documents directory. On desktop this is the same directory
    /// the Tauri app uses (`%APPDATA%\ARK ASA Config Manager`).
    #[uniffi::constructor]
    pub fn new(config_dir: String) -> Arc<Self> {
        Arc::new(Self {
            config_dir: config_dir.into(),
        })
    }

    /// All registered server profiles, with the active flag resolved.
    pub fn list_profiles(&self) -> Result<Vec<ProfileView>, FacadeError> {
        let store = self.store()?;
        let active = store.active_id();
        Ok(store
            .list()
            .into_iter()
            .map(|p| ProfileView {
                is_active: p.id == active,
                id: p.id,
                name: p.name,
                config_path: p.config_path,
                created_at_unix: p.created_at_unix,
            })
            .collect())
    }

    /// The id of the currently active profile.
    pub fn active_profile_id(&self) -> Result<String, FacadeError> {
        Ok(self.store()?.active_id())
    }

    /// Look up a single profile by id (`None` when it does not exist).
    pub fn get_profile(&self, id: String) -> Result<Option<ProfileView>, FacadeError> {
        let store = self.store()?;
        let active = store.active_id();
        Ok(store.get(&id).map(|p| ProfileView {
            is_active: p.id == active,
            id: p.id,
            name: p.name,
            config_path: p.config_path,
            created_at_unix: p.created_at_unix,
        }))
    }

    /// Absolute path to the configuration directory this instance is bound to.
    pub fn config_dir(&self) -> String {
        self.config_dir.to_string_lossy().into_owned()
    }

    /// Resolves where to dial for the console, from the active configuration.
    ///
    /// Preference order mirrors the desktop: the primary connection entry wins;
    /// failing that the first entry; failing that `server_ip`; failing that
    /// loopback (which is only meaningful when the app and the server share a
    /// host, so the app surfaces the label to make that visible).
    ///
    /// A `:port` suffix on the entry's address is stripped, because the RCON
    /// port is a separate, explicit field — the same rule the desktop launcher
    /// applies at the last moment.
    pub fn connection_target(&self) -> Result<ConnectionTarget, FacadeError> {
        let config = self.load_active_config()?;
        let rcon_port = config.network.rcon_port;

        // Skip entries with no address: the default config ships a blank
        // `Manual` placeholder marked primary, and dialing "" is not a target.
        let usable = |e: &&ark_core::config::schema::ConnectionEntry| !e.effective_ip().is_empty();
        let chosen = config
            .network
            .connection_entries
            .iter()
            .filter(usable)
            .find(|e| e.is_primary)
            .or_else(|| config.network.connection_entries.iter().find(usable));

        let (host, label) = match chosen {
            Some(entry) => (strip_port(entry.effective_ip()), entry.label.clone()),
            None if !config.network.server_ip.trim().is_empty() => {
                (strip_port(&config.network.server_ip), "server_ip".to_string())
            }
            None => ("127.0.0.1".to_string(), "loopback".to_string()),
        };

        Ok(ConnectionTarget {
            host,
            port: rcon_port,
            label,
        })
    }

    /// Loads the active profile's configuration, projected to [`ConfigView`].
    pub fn load_config(&self) -> Result<ConfigView, FacadeError> {
        let config = self.load_active_config()?;
        Ok(config_to_view(&config))
    }

    /// Runs the core's validators over an edited configuration.
    ///
    /// Returns one entry per complaint; an empty list means the edit is valid.
    /// Validation happens against the real composite validator (the same one the
    /// desktop uses), so the phone cannot accept a configuration the desktop
    /// would reject — the rules have a single implementation.
    pub fn validate_config(&self, config: ConfigView) -> Result<Vec<ValidationIssueView>, FacadeError> {
        let mut full = self.load_active_config()?;
        apply_view(&mut full, &config);

        let result = block_on(async move {
            ark_core::config::validator::CompositeValidator::default()
                .validate(&full)
                .await
        })
        .map_err(|e| FacadeError::ProfileStore {
            detail: format!("validation failed to run: {e}"),
        })?;

        Ok(result
            .errors
            .into_iter()
            .map(|e| ValidationIssueView {
                field: e.validator,
                message: e.message,
                code: None,
            })
            .collect())
    }

    /// Writes an edited configuration to the active profile, then regenerates
    /// the ARK `.ini` files the same way the desktop does.
    ///
    /// Only the fields in [`ConfigView`] are overwritten; everything else in the
    /// TOML is preserved, so a phone edit can never silently drop a setting the
    /// desktop wrote.
    pub fn save_config(&self, config: ConfigView) -> Result<(), FacadeError> {
        let mut full = self.load_active_config()?;
        apply_view(&mut full, &config);

        // Refuse to persist something the core itself rejects: writing an
        // invalid config would leave the server unlaunchable.
        let issues = self.validate_config(config)?;
        if !issues.is_empty() {
            return Err(FacadeError::ProfileStore {
                detail: format!("refusing to save invalid configuration: {}", issues[0].message),
            });
        }

        let path = self.active_config_path()?;
        let ini_dir = path.parent().map(|p| p.to_path_buf());
        block_on(async move {
            ark_core::config::ConfigPersister::save_toml(&full, &path).await?;
            if let Some(dir) = ini_dir {
                let game_ini = dir.join("GameUserSettings.ini");
                let game_settings = dir.join("Game.ini");
                ark_core::config::ConfigPersister::generate_game_ini(&full, &game_ini).await?;
                ark_core::config::ConfigPersister::generate_gamesettings_ini(
                    &full,
                    &game_settings,
                )
                .await?;
            }
            Ok::<(), ark_core::Error>(())
        })
        .map_err(|e| FacadeError::ProfileStore {
            detail: e.to_string(),
        })
    }
}

/// Bridges `ark-core`'s async config operations to the synchronous UniFFI
/// surface.
///
/// A current-thread runtime per call is intentional: these run off the UI thread
/// on the Kotlin side, are infrequent (a load or a save), and building a runtime
/// is far cheaper than negotiating a shared one across the FFI boundary. If a
/// call ever becomes hot, cache the runtime rather than reaching for
/// `block_in_place`, which would require the caller to already be inside tokio.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build a runtime to run a core operation")
        .block_on(future)
}

/// Projects a full `ServerConfig` down to the phone-editable fields.
fn config_to_view(config: &ark_core::config::ServerConfig) -> ConfigView {
    ConfigView {
        session_name: config.identification.session_name.clone(),
        server_password: config.identification.server_password.clone(),
        admin_password: config.identification.admin_password.clone(),
        motd: config.identification.server_message_of_the_day.clone(),
        port: config.network.port,
        query_port: config.network.query_port,
        rcon_port: config.network.rcon_port,
        max_players: config.gameplay.max_players,
        cluster_maps: config.cluster_maps.clone(),
        no_battleye: config.network.no_battleye,
    }
}

/// Copies the editable fields back onto a full config, leaving the rest alone.
fn apply_view(config: &mut ark_core::config::ServerConfig, view: &ConfigView) {
    config.identification.session_name = view.session_name.clone();
    config.identification.server_password = view.server_password.clone();
    config.identification.admin_password = view.admin_password.clone();
    config.identification.server_message_of_the_day = view.motd.clone();
    config.network.port = view.port;
    config.network.query_port = view.query_port;
    config.network.rcon_port = view.rcon_port;
    config.gameplay.max_players = view.max_players;
    config.cluster_maps = view.cluster_maps.clone();
    config.network.no_battleye = view.no_battleye;
}

/// Drops a trailing `:port` from an address, leaving IPv6 literals intact.
///
/// The ambiguity is real: `host:port` and a bare IPv6 literal both contain a
/// colon. The disambiguation is the *count* — an address with more than one
/// colon is an IPv6 literal and is returned untouched; exactly one colon means
/// `host:port` and is split.
fn strip_port(address: &str) -> String {
    let trimmed = address.trim();
    if let Some(close) = trimmed.rfind(']') {
        // Bracketed IPv6, optionally followed by `:port` — keep the brackets.
        return trimmed[..=close].to_string();
    }
    if trimmed.matches(':').count() == 1 {
        return trimmed.split(':').next().unwrap_or(trimmed).to_string();
    }
    // Zero colons (a bare host) or many (a bare IPv6 literal): unchanged.
    trimmed.to_string()
}

impl MobileCore {
    fn store(&self) -> Result<ark_core::config::ProfileStore, FacadeError> {
        if !self.config_dir.is_dir() {
            return Err(FacadeError::ProfileStore {
                detail: format!(
                    "configuration directory does not exist: {}",
                    self.config_dir.display()
                ),
            });
        }
        Ok(ark_core::config::ProfileStore::new(self.config_dir.clone()))
    }

    /// Absolute path to the active profile's `server-config.toml`.
    fn active_config_path(&self) -> Result<std::path::PathBuf, FacadeError> {
        let path = self
            .store()?
            .resolve_config_path(None)
            .map_err(|e| FacadeError::ProfileStore {
                detail: e.to_string(),
            })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| FacadeError::ProfileStore {
                detail: format!("could not create {}: {e}", parent.display()),
            })?;
        }
        Ok(path)
    }

    /// Loads the active profile's configuration from disk.
    fn load_active_config(&self) -> Result<ark_core::config::ServerConfig, FacadeError> {
        let path = self.active_config_path()?;
        block_on(async move { ark_core::config::ConfigLoader::load_or_default(&path).await })
            .map_err(|e| FacadeError::ProfileStore {
                detail: e.to_string(),
            })
    }
}

/// Counts of outbox entries per state, for a status row in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct OutboxCounts {
    pub pending: u64,
    pub in_flight: u64,
    pub done: u64,
    pub failed: u64,
}

/// One action the host should attempt, handed to the app by
/// [`OfflineCore::due_actions`].
///
/// The host performs the network call and reports back with
/// [`OfflineCore::mark_action_done`] or [`OfflineCore::mark_action_failed`].
/// Splitting it this way keeps the durable state machine in Rust — one
/// implementation for every platform — while the transport (an RCON socket,
/// a tunnel) stays where it belongs, with the app.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DueAction {
    pub idempotency_key: String,
    pub action: String,
    pub payload: String,
    pub attempts: u32,
}

/// Outcome of a drain pass, so the UI can report it without re-querying.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct DrainOutcome {
    pub reclaimed: u64,
    pub claimed: u64,
}

/// One entry of the durable queue, as the UI displays it.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct QueuedAction {
    pub idempotency_key: String,
    pub action: String,
    pub payload: String,
    /// `pending` | `in_flight` | `done` | `failed`.
    pub state: String,
    pub attempts: u32,
    pub max_attempts: u32,
    /// Why the last attempt failed, when it did.
    pub last_error: Option<String>,
}

/// Offline surface: local state + the durable write queue.
///
/// Kept as a second object rather than more methods on `MobileCore` so the
/// lifetime and purpose of each is obvious: `MobileCore` is configuration and
/// profiles, `OfflineCore` is "what we know and what we still owe the server".
#[derive(uniffi::Object)]
pub struct OfflineCore {
    config_dir: std::path::PathBuf,
}

#[uniffi::export]
impl OfflineCore {
    #[uniffi::constructor]
    pub fn new(config_dir: String) -> Arc<Self> {
        Arc::new(Self {
            config_dir: config_dir.into(),
        })
    }

    /// Seeds the local store from the filesystem and returns what it holds now.
    ///
    /// This is the sync path: read the profiles the core manages, persist them
    /// into the local store (the single source of truth), and hand back the
    /// stored rows. The UI then renders from the store without touching disk.
    pub fn sync_profiles(&self) -> Result<Vec<ProfileView>, FacadeError> {
        let profiles = self.list_profiles_now()?;
        let stored: Vec<ark_core::offline::StoredProfile> = profiles
            .iter()
            .map(|p| ark_core::offline::StoredProfile {
                id: p.id.clone(),
                name: p.name.clone(),
                config_path: p.config_path.clone(),
                created_at_unix: p.created_at_unix,
                is_active: p.is_active,
            })
            .collect();

        let store = ark_core::offline::OfflineStore::open(&self.config_dir).map_err(|e| {
            FacadeError::ProfileStore {
                detail: e.to_string(),
            }
        })?;
        store.replace_profiles(&stored)?;

        Ok(profiles)
    }

    /// Profiles as held by the local store — no filesystem access.
    pub fn cached_profiles(&self) -> Result<Vec<ProfileView>, FacadeError> {
        let store = self.offline_store()?;
        Ok(store
            .profiles()?
            .into_iter()
            .map(|p| ProfileView {
                id: p.id,
                name: p.name,
                config_path: p.config_path,
                created_at_unix: p.created_at_unix,
                is_active: p.is_active,
            })
            .collect())
    }

    /// Queues a write-ahead action.
    ///
    /// Returns `true` when newly queued, `false` when the idempotency key was
    /// already present — the caller can treat `false` as "already on its way",
    /// which makes a retried user action harmless.
    pub fn enqueue_action(
        &self,
        idempotency_key: String,
        action: String,
        payload: String,
    ) -> Result<bool, FacadeError> {
        let outbox = self.outbox_store()?;
        let queued = outbox.enqueue(ark_core::offline::NewOutboxEntry::new(
            idempotency_key,
            action,
            payload,
        ))?;
        Ok(queued)
    }

    /// How many actions are waiting, in flight, delivered or given up.
    pub fn outbox_counts(&self) -> Result<OutboxCounts, FacadeError> {
        let (pending, in_flight, done, failed) = self.outbox_store()?.counts()?;
        Ok(OutboxCounts {
            pending,
            in_flight,
            done,
            failed,
        })
    }

    /// Marks a queued action delivered (called by the sender on success).
    pub fn mark_action_done(&self, idempotency_key: String) -> Result<(), FacadeError> {
        self.outbox_store()?.mark_done(&idempotency_key)?;
        Ok(())
    }

    /// Records a failed attempt, scheduling a backoff retry or giving up.
    pub fn mark_action_failed(
        &self,
        idempotency_key: String,
        error: String,
    ) -> Result<String, FacadeError> {
        let state = self.outbox_store()?.mark_failed(&idempotency_key, error)?;
        Ok(state.as_str().to_string())
    }

    /// Recovers actions abandoned by a crash, returning how many were reclaimed.
    ///
    /// Call this **once** at the beginning of a drain, before `due_actions`.
    /// It is deliberately separate: if listing due work also reclaimed in-flight
    /// entries, then merely *observing* the queue would hand an in-progress
    /// action to a second sender, defeating the claim. Recovery must be an
    /// explicit decision made when no drain is running.
    pub fn reclaim_interrupted_actions(&self) -> Result<u64, FacadeError> {
        let outbox = self.outbox_store()?;
        let mut reclaimed = 0;
        for entry in outbox.all()? {
            if entry.state == ark_core::offline::OutboxState::InFlight {
                outbox.mark_failed(
                    &entry.idempotency_key,
                    "reclaimed after an interrupted attempt",
                )?;
                reclaimed += 1;
            }
        }
        Ok(reclaimed)
    }

    /// Actions that are due for an attempt right now.
    ///
    /// Read-only: it never changes an entry's state, so calling it from a UI
    /// poll cannot disturb a drain that is in progress.
    pub fn due_actions(&self) -> Result<Vec<DueAction>, FacadeError> {
        Ok(self
            .outbox_store()?
            .due()?
            .into_iter()
            .map(|e| DueAction {
                idempotency_key: e.idempotency_key,
                action: e.action,
                payload: e.payload,
                attempts: e.attempts,
            })
            .collect())
    }

    /// Claims one due action, so a concurrent drain will not attempt it twice.
    ///
    /// Returns `true` when the caller now owns the attempt and must report back
    /// via `mark_action_done` / `mark_action_failed`.
    pub fn claim_action(&self, idempotency_key: String) -> Result<bool, FacadeError> {
        let outbox = self.outbox_store()?;
        outbox.mark_in_flight(&idempotency_key)?;
        Ok(true)
    }

    /// Every queued action, so the UI can show what is waiting and why.
    pub fn outbox_entries(&self) -> Result<Vec<QueuedAction>, FacadeError> {
        Ok(self
            .outbox_store()?
            .all()?
            .into_iter()
            .map(|e| QueuedAction {
                idempotency_key: e.idempotency_key,
                action: e.action,
                payload: e.payload,
                state: e.state.as_str().to_string(),
                attempts: e.attempts,
                max_attempts: e.max_attempts,
                last_error: e.last_error,
            })
            .collect())
    }

    /// Deletes delivered entries older than `age_secs`, reclaiming disk.
    pub fn prune_done(&self, age_secs: u64) -> Result<u64, FacadeError> {
        Ok(self.outbox_store()?.prune_done(age_secs)?)
    }

    /// Returns `Failed` entries to `Pending` so a retry can be attempted.
    ///
    /// Resets the attempt counter: "give up" is a terminal state the operator
    /// clears by hand, and re-queueing with the budget already spent would make
    /// the button do nothing.
    pub fn retry_action(&self, idempotency_key: String) -> Result<(), FacadeError> {
        let outbox = self.outbox_store()?;
        outbox.requeue_failed(&idempotency_key)?;
        Ok(())
    }

    fn list_profiles_now(&self) -> Result<Vec<ProfileView>, FacadeError> {
        if !self.config_dir.is_dir() {
            return Err(FacadeError::ProfileStore {
                detail: format!(
                    "configuration directory does not exist: {}",
                    self.config_dir.display()
                ),
            });
        }
        let store = ark_core::config::ProfileStore::new(self.config_dir.clone());
        let active = store.active_id();
        Ok(store
            .list()
            .into_iter()
            .map(|p| ProfileView {
                is_active: p.id == active,
                id: p.id,
                name: p.name,
                config_path: p.config_path,
                created_at_unix: p.created_at_unix,
            })
            .collect())
    }
}

/// Private helpers kept outside the exported `impl` block: UniFFI only lifts
/// types that appear in exported signatures, and `OfflineStore`/`Outbox` are
/// internal implementation details that must not cross the FFI boundary.
impl OfflineCore {
    fn offline_store(&self) -> Result<ark_core::offline::OfflineStore, FacadeError> {
        ark_core::offline::OfflineStore::open(&self.config_dir)
            .map_err(|e| FacadeError::ProfileStore {
                detail: e.to_string(),
            })
    }

    fn outbox_store(&self) -> Result<ark_core::offline::Outbox, FacadeError> {
        ark_core::offline::Outbox::open(&self.config_dir).map_err(|e| {
            FacadeError::ProfileStore {
                detail: e.to_string(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_directory_is_a_typed_error_not_a_panic() {
        let core = MobileCore::new("Z:\\definitely\\not\\here".into());
        match core.list_profiles() {
            Err(FacadeError::ProfileStore { detail }) => {
                assert!(detail.contains("does not exist"), "got: {detail}");
            }
            other => panic!("expected ProfileStore error, got {other:?}"),
        }
    }

    #[test]
    fn empty_directory_yields_the_default_profile() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = MobileCore::new(dir.path().to_string_lossy().into_owned());
        let profiles = core.list_profiles().expect("list_profiles");
        // `ProfileStore` adopts a legacy config as the `default` profile.
        assert_eq!(profiles.len(), 1, "got: {profiles:?}");
        assert_eq!(profiles[0].id, "default");
        assert!(profiles[0].is_active);
        assert_eq!(core.active_profile_id().unwrap(), "default");
    }

    #[test]
    fn roundtrips_a_real_profile_through_the_facade() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = MobileCore::new(dir.path().to_string_lossy().into_owned());
        let created = ark_core::config::ProfileStore::new(dir.path())
            .create("Mi Servidor")
            .expect("create profile");
        let id = created.id.clone();

        let fetched = core.get_profile(id.clone()).unwrap().expect("profile exists");
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.name, "Mi Servidor");
        assert!(fetched.config_path.ends_with(".toml"));

        // A second profile does not steal the active flag.
        let active = core.active_profile_id().unwrap();
        assert_eq!(
            core.get_profile(active).unwrap().unwrap().is_active,
            true
        );
        assert_eq!(fetched.is_active, false);

        assert!(core.get_profile("nope".into()).unwrap().is_none());
    }

    #[test]
    fn config_dir_is_echoed_verbatim() {
        let dir = tempfile::tempdir().expect("tempdir");
        let as_string = dir.path().to_string_lossy().into_owned();
        assert_eq!(MobileCore::new(as_string.clone()).config_dir(), as_string);
    }

    #[test]
    fn offline_sync_persists_and_reads_back_from_the_local_store() {
        let dir = tempfile::tempdir().expect("tempdir");
        let offline = OfflineCore::new(dir.path().to_string_lossy().into_owned());

        // Nothing stored yet.
        assert!(offline.cached_profiles().unwrap().is_empty());

        let synced = offline.sync_profiles().expect("sync");
        assert_eq!(synced.len(), 1, "the adopted default profile");
        assert_eq!(synced[0].id, "default");

        // The cached read must match what the filesystem said, without touching
        // the filesystem again.
        let cached = offline.cached_profiles().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].id, synced[0].id);
        assert_eq!(cached[0].name, synced[0].name);
        assert_eq!(cached[0].is_active, synced[0].is_active);
    }

    #[test]
    fn a_queued_action_is_idempotent_through_the_bridge() {
        let dir = tempfile::tempdir().expect("tempdir");
        let offline = OfflineCore::new(dir.path().to_string_lossy().into_owned());

        assert!(offline
            .enqueue_action(
                "cmd-1".into(),
                "rcon.command".into(),
                r#"{"command":"saveworld"}"#.into()
            )
            .unwrap());
        // Re-sending the same intent must not queue a second copy.
        assert!(!offline
            .enqueue_action(
                "cmd-1".into(),
                "rcon.command".into(),
                r#"{"command":"saveworld"}"#.into()
            )
            .unwrap());

        let counts = offline.outbox_counts().unwrap();
        assert_eq!(counts.pending, 1);
        assert_eq!(counts.done, 0);
    }

    #[test]
    fn outbox_lifecycle_is_visible_through_the_bridge() {
        let dir = tempfile::tempdir().expect("tempdir");
        let offline = OfflineCore::new(dir.path().to_string_lossy().into_owned());
        offline
            .enqueue_action("cmd-2".into(), "rcon.command".into(), "{}".into())
            .unwrap();

        offline.mark_action_done("cmd-2".into()).unwrap();
        let counts = offline.outbox_counts().unwrap();
        assert_eq!(counts.done, 1);
        assert_eq!(counts.pending, 0);

        // A failing key surfaces its terminal state as a string the UI can show.
        offline
            .enqueue_action("cmd-3".into(), "rcon.command".into(), "{}".into())
            .unwrap();
        let state = offline
            .mark_action_failed("cmd-3".into(), "connection refused".into())
            .unwrap();
        assert_eq!(state, "pending", "first failure schedules a retry");
    }

    #[test]
    fn the_mobile_drain_cycle_claims_then_reports() {
        let dir = tempfile::tempdir().expect("tempdir");
        let offline = OfflineCore::new(dir.path().to_string_lossy().into_owned());
        offline
            .enqueue_action(
                "cmd-9".into(),
                "rcon.command".into(),
                r#"{"command":"saveworld"}"#.into(),
            )
            .unwrap();

        // 1. What to attempt: the host gets the payload it must send.
        let due = offline.due_actions().unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].idempotency_key, "cmd-9");
        assert_eq!(due[0].payload, r#"{"command":"saveworld"}"#);

        // 2. Claim it, so a second drain pass will not send it again...
        assert!(offline.claim_action("cmd-9".into()).unwrap());
        assert!(
            offline.due_actions().unwrap().is_empty(),
            "a claimed action is not due"
        );
        assert_eq!(offline.outbox_counts().unwrap().in_flight, 1);

        // 3. ...and report the transport result.
        offline.mark_action_done("cmd-9".into()).unwrap();
        let counts = offline.outbox_counts().unwrap();
        assert_eq!((counts.done, counts.in_flight, counts.pending), (1, 0, 0));
    }

    #[test]
    fn a_crash_while_in_flight_is_recovered_by_an_explicit_reclaim() {
        let dir = tempfile::tempdir().expect("tempdir");
        let offline = OfflineCore::new(dir.path().to_string_lossy().into_owned());
        offline
            .enqueue_action("cmd-10".into(), "rcon.command".into(), "{}".into())
            .unwrap();
        // Simulate dying after claiming, before reporting.
        offline.claim_action("cmd-10".into()).unwrap();

        // Reading the queue must NOT disturb the claim: an in-progress action
        // is not due, and observing it cannot hand it to a second sender.
        assert!(offline.due_actions().unwrap().is_empty());

        // Recovery is an explicit decision, made when no drain is running.
        assert_eq!(offline.reclaim_interrupted_actions().unwrap(), 1);

        // The reclaimed action is back in `Pending` — the queue owns it again,
        // so it cannot be lost. It is not *due* immediately, because reclaiming
        // goes through the normal failure path and that schedules a backoff; the
        // next drain picks it up once that elapses.
        let counts = offline.outbox_counts().unwrap();
        assert_eq!((counts.pending, counts.in_flight), (1, 0));
    }

    #[test]
    fn config_loads_with_the_documented_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = MobileCore::new(dir.path().to_string_lossy().into_owned());
        ark_core::config::ProfileStore::new(dir.path())
            .create("Servidor")
            .expect("create profile");

        let view = core.load_config().expect("load");
        assert_eq!(view.port, 7777, "ARK's default game port");
        assert_eq!(view.cluster_maps.len(), 1, "one map by default");
    }

    #[test]
    fn saving_preserves_fields_the_phone_does_not_edit() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = MobileCore::new(dir.path().to_string_lossy().into_owned());
        ark_core::config::ProfileStore::new(dir.path())
            .create("Servidor")
            .expect("create profile");

        // A freshly created profile has no admin password, and the core refuses
        // to save without one — so seed a valid baseline first. This mirrors
        // what the app must do before it can persist anything.
        let mut baseline = core.load_config().expect("load");
        baseline.admin_password = "s3cret".into();
        baseline.session_name = "Servidor".into();
        core.save_config(baseline).expect("seed baseline");

        // Write a marker into a section the phone's editor never touches.
        let path = core.active_config_path().expect("path");
        let mut original = block_on(async {
            ark_core::config::ConfigLoader::load_from_toml(&path).await
        })
        .expect("seed file");
        original.performance.enable_debug_logging = true;
        block_on(async {
            ark_core::config::ConfigPersister::save_toml(&original, &path).await
        })
        .expect("seed file");

        // The phone edits only what it can see.
        let mut edited = core.load_config().expect("load");
        edited.session_name = "Editado desde el móvil".into();
        edited.max_players = 20;
        core.save_config(edited).expect("save");

        // The untouched section survived the round trip.
        let reloaded = block_on(async {
            ark_core::config::ConfigLoader::load_from_toml(&path).await
        })
        .expect("reload");
        assert_eq!(reloaded.identification.session_name, "Editado desde el móvil");
        assert_eq!(reloaded.gameplay.max_players, 20);
        assert!(
            reloaded.performance.enable_debug_logging,
            "a field outside ConfigView must not be erased by a phone save"
        );
    }

    #[test]
    fn an_invalid_edit_is_reported_and_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = MobileCore::new(dir.path().to_string_lossy().into_owned());
        ark_core::config::ProfileStore::new(dir.path())
            .create("Servidor")
            .expect("create profile");

        let mut broken = core.load_config().expect("load");
        broken.admin_password = "s3cret".into();
        broken.port = 80; // below 1024: the port validator rejects this
        broken.query_port = 80;
        broken.rcon_port = 80;

        let issues = core.validate_config(broken.clone()).expect("validate runs");
        assert!(!issues.is_empty(), "the core's validators object");
        assert!(
            issues.iter().any(|i| i.message.to_lowercase().contains("port")),
            "the complaint names the port, got: {issues:?}"
        );

        // Saving is refused, so the server cannot be left unlaunchable.
        let err = core.save_config(broken).expect_err("must refuse");
        assert!(format!("{err:?}").contains("invalid"), "got: {err:?}");
    }

    #[test]
    fn strip_port_keeps_ipv6_intact() {
        assert_eq!(strip_port("192.168.1.10:7777"), "192.168.1.10");
        assert_eq!(strip_port("playit.gg"), "playit.gg");
        assert_eq!(strip_port("  host.example:1234 "), "host.example");
        // A bare IPv6 literal must not be truncated at its first colon.
        assert_eq!(strip_port("2001:db8::1"), "2001:db8::1");
        assert_eq!(strip_port("[2001:db8::1]:7777"), "[2001:db8::1]");
    }

    #[test]
    fn connection_target_falls_back_to_loopback_with_no_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = MobileCore::new(dir.path().to_string_lossy().into_owned());
        ark_core::config::ProfileStore::new(dir.path())
            .create("Servidor")
            .expect("create profile");

        let target = core.connection_target().expect("resolve");
        // A fresh config has no connection entries and no server_ip, so the
        // only address that could work is the local one.
        assert_eq!(target.host, "127.0.0.1");
        assert_eq!(target.port, 27020, "ARK's default RCON port");
        assert_eq!(target.label, "loopback");
    }

    #[test]
    fn connection_target_prefers_the_primary_entry_without_its_port() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = MobileCore::new(dir.path().to_string_lossy().into_owned());
        ark_core::config::ProfileStore::new(dir.path())
            .create("Servidor")
            .expect("create profile");

        let mut config = core.load_active_config().expect("load");
        config.network.connection_entries = vec![
            ark_core::config::schema::ConnectionEntry::new(
                ark_core::config::schema::ConnectionType::LocalIp,
                "Lan".into(),
                "192.168.1.50:7777".into(),
            ),
            ark_core::config::schema::ConnectionEntry::new(
                ark_core::config::schema::ConnectionType::Tailscale,
                "Tailscale".into(),
                "100.64.0.9".into(),
            ),
        ];
        config.network.connection_entries[1].is_primary = true;
        let path = core.active_config_path().expect("path");
        block_on(async { ark_core::config::ConfigPersister::save_toml(&config, &path).await })
            .expect("seed");

        let target = core.connection_target().expect("resolve");
        assert_eq!(target.host, "100.64.0.9", "the primary entry wins");
        assert_eq!(target.label, "Tailscale");
    }
}
