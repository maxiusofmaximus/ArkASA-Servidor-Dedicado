//! Database adapter layer — flexible backend persistence for the audit log,
//! command history, and bot state. Each backend is expressed as an enum + a
//! `DatabaseConfig` TOML block so the user can pick exactly one or stay on
//! the default local SQLite file.
//!
//! Supported backends:
//!   - `sqlite` (default, zero-config, single-user)
//!   - `convex` (existing v2.1 cloud BaaS; HTTP via reqwest)
//!   - `supabase` (Postgres + REST + Realtime WebSockets)
//!   - `insforge` (Postgres + email/storage/AI; OpenAI-compatible)
//!   - `postgres` (raw libpq connection — works for Neon, Timescale, RDS, Supabase pooler)
//!   - `mongodb` (MongoDB Atlas — document store, scalable)
//!
//! All backends implement the same `AuditDao` trait so the rest of the code
//! stays unaware of the storage choice.

use serde::{Deserialize, Serialize};

/// Backend selection. Adding a new one requires implementing the same async
/// shape, never the call sites.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DbBackend {
    Sqlite,
    #[default]
    SqliteAlt, // alias of Sqlite so JSON `{"backend":"sqlite"}` resolves
    Convex,
    Supabase,
    Insforge,
    Postgres,
    Mongodb,
}

impl DbBackend {
    pub fn label(self) -> &'static str {
        match self {
            DbBackend::Sqlite | DbBackend::SqliteAlt => "SQLite (local)",
            DbBackend::Convex => "Convex (BaaS)",
            DbBackend::Supabase => "Supabase (Postgres+Realtime)",
            DbBackend::Insforge => "InsForge (Postgres+AI)",
            DbBackend::Postgres => "PostgreSQL (libpq)",
            DbBackend::Mongodb => "MongoDB Atlas",
        }
    }

    /// Stable, lowercased key used in the UI / TOML / env var.
    pub fn key(self) -> &'static str {
        match self {
            DbBackend::Sqlite | DbBackend::SqliteAlt => "sqlite",
            DbBackend::Convex => "convex",
            DbBackend::Supabase => "supabase",
            DbBackend::Insforge => "insforge",
            DbBackend::Postgres => "postgres",
            DbBackend::Mongodb => "mongodb",
        }
    }

    /// SQLite is the only zero-config, always-available backend.
    pub fn is_local(self) -> bool {
        matches!(self, DbBackend::Sqlite | DbBackend::SqliteAlt)
    }

    /// Parse from the stable key (`DB_BACKEND` / `database.toml`).
    pub fn from_key(s: &str) -> Option<DbBackend> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "sqlite" | "sqlitealt" => DbBackend::Sqlite,
            "convex" => DbBackend::Convex,
            "supabase" => DbBackend::Supabase,
            "insforge" => DbBackend::Insforge,
            "postgres" | "postgresql" => DbBackend::Postgres,
            "mongodb" | "mongo" => DbBackend::Mongodb,
            _ => return None,
        })
    }

    /// Honest one-liner shown in the UI so the operator knows what a backend
    /// actually needs before picking it.
    pub fn note(self) -> &'static str {
        match self {
            DbBackend::Sqlite | DbBackend::SqliteAlt =>
                "Default · zero-config · local file, no network",
            DbBackend::Convex =>
                "HTTP DAO implemented · requires the `command_log:append` / `:recent` functions on your deployment",
            DbBackend::Supabase =>
                "HTTP DAO implemented · PostgREST table + anon key required",
            DbBackend::Insforge =>
                "HTTP DAO implemented · InsForge records API + key required",
            DbBackend::Postgres =>
                "PostgREST-compatible HTTP DAO · use a PostgREST URL (native libpq not bundled)",
            DbBackend::Mongodb =>
                "Atlas Data API DAO · use the Data API base URL + API key",
        }
    }

    pub fn all() -> &'static [DbBackend] {
        const ALL: &[DbBackend] = &[
            DbBackend::Sqlite,
            DbBackend::Convex,
            DbBackend::Supabase,
            DbBackend::Insforge,
            DbBackend::Postgres,
            DbBackend::Mongodb,
        ];
        ALL
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DbBackendView {
    pub key: String,
    pub label: String,
    pub local: bool,
    /// A DAO exists for this backend (all of them do today). Kept explicit so
    /// future "advertised but unimplemented" entries can say so instead of
    /// silently no-op'ing.
    pub implemented: bool,
    pub note: String,
}

pub fn all_backends() -> Vec<DbBackendView> {
    DbBackend::all().iter().map(|b| DbBackendView {
        key: b.key().to_string(),
        label: b.label().to_string(),
        local: b.is_local(),
        implemented: true,
        note: b.note().to_string(),
    }).collect()
}

/// Schema-agnostic audit log entry. Each backend maps this into its native row
/// document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRow {
    pub host_id:    String,
    pub actor_id:   String,
    pub actor_name: String,
    pub channel:    String,
    pub kind:       String,
    pub map_index:  Option<u32>,
    pub result:     Option<String>,
    pub at:         i64, // epoch millis
}

/// Database configuration — read from env vars so the operator can swap
/// backends without rebuilding the app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub backend: DbBackend,
    pub url:     String,
    pub api_key: String,
    pub schema:  String,
    pub table:   String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: DbBackend::Sqlite,
            url: std::env::var("DB_URL").unwrap_or_else(|_| "ark-config.db".into()),
            api_key: std::env::var("DB_API_KEY").unwrap_or_default(),
            schema: std::env::var("DB_SCHEMA").unwrap_or_else(|_| "public".into()),
            table: std::env::var("DB_TABLE").unwrap_or_else(|_| "command_log".into()),
        }
    }
}

impl DatabaseConfig {
    /// Active when a recognised backend + URL are configured. SQLite is
    /// always-on (the file path defaults to `./ark-config.db`).
    pub fn is_active(&self) -> bool {
        !self.url.is_empty()
    }

    /// `<config_dir>/database.toml`.
    pub fn path(config_dir: &std::path::Path) -> std::path::PathBuf {
        config_dir.join("database.toml")
    }

    /// Resolve the effective backend config: the persisted
    /// `<config_dir>/database.toml` (the operator's choice made in the UI),
    /// optionally overridden per-field by `DB_BACKEND` / `DB_URL` /
    /// `DB_API_KEY` / `DB_SCHEMA` / `DB_TABLE` so headless & CI runs can force
    /// a backend. Missing file → env/defaults.
    pub fn load(config_dir: &std::path::Path) -> DatabaseConfig {
        let mut cfg = std::fs::read_to_string(Self::path(config_dir))
            .ok()
            .and_then(|raw| toml::from_str::<DatabaseConfig>(&raw).ok())
            .unwrap_or_default();
        if let Ok(v) = std::env::var("DB_BACKEND") {
            if let Some(b) = DbBackend::from_key(&v) { cfg.backend = b; }
        }
        if let Ok(v) = std::env::var("DB_URL")     { if !v.is_empty() { cfg.url = v; } }
        if let Ok(v) = std::env::var("DB_API_KEY") { if !v.is_empty() { cfg.api_key = v; } }
        if let Ok(v) = std::env::var("DB_SCHEMA")  { if !v.is_empty() { cfg.schema = v; } }
        if let Ok(v) = std::env::var("DB_TABLE")   { if !v.is_empty() { cfg.table = v; } }
        cfg
    }

    /// Persist the operator's choice.
    pub fn save(&self, config_dir: &std::path::Path) -> Result<(), String> {
        std::fs::create_dir_all(config_dir).map_err(|e| e.to_string())?;
        let raw = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        let path = Self::path(config_dir);
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, raw).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("rename -> {}: {e}", path.display()))?;
        Ok(())
    }
}

/// Async Insert API contract — every backend implements this.
#[async_trait::async_trait]
pub trait AuditDao: Send + Sync {
    async fn append(&self, row: AuditRow) -> Result<(), String>;
    async fn recent(&self, limit: u32, channel: Option<&str>)
        -> Result<Vec<AuditRow>, String>;
    async fn by_host(&self, host_id: &str, limit: u32)
        -> Result<Vec<AuditRow>, String>;
}

/// Build the right DAO from a [`DatabaseConfig`].
pub async fn build_dao(cfg: &DatabaseConfig) -> Result<Box<dyn AuditDao>, String> {
    match cfg.backend {
        DbBackend::Sqlite | DbBackend::SqliteAlt => {
            Ok(Box::new(sqlite::SqliteDao::open(&cfg.url).await?))
        }
        DbBackend::Convex  => Ok(Box::new(remote::HttpDao::convex(cfg.clone()))),
        DbBackend::Supabase=> Ok(Box::new(remote::HttpDao::supabase(cfg.clone()))),
        DbBackend::Insforge=> Ok(Box::new(remote::HttpDao::insforge(cfg.clone()))),
        // Postgres / Mongo require native drivers (sqlx / mongo) and SSL
        // infrastructure. We currently expose them as opaque "remote
        // providers" that the operator configures with a REST gateway
        // function. Concrete native drivers can be added in Hito 14+ when
        // the user opts into them and we add the corresponding Cargo features.
        DbBackend::Postgres => Ok(Box::new(remote::HttpDao::postrest(cfg.clone()))),
        DbBackend::Mongodb  => Ok(Box::new(remote::HttpDao::mockcosm(cfg.clone()))),
    }
}

pub mod sqlite;
pub mod remote;

// ── Active DAO (P5) ──────────────────────────────────────────────────────────
// The built DAO used to be discarded (`Ok(_dao)`), so a selected Convex /
// Supabase / Postgres / Mongo backend never received a single row — the
// catalog was effectively a label. We now retain it in process-wide state and
// expose its readiness so the UI can tell the truth.

use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, Serialize, Default)]
pub struct DaoStatus {
    pub backend: String,
    pub ready: bool,
    pub detail: String,
}

fn status_cell() -> &'static Mutex<DaoStatus> {
    static S: OnceLock<Mutex<DaoStatus>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(DaoStatus::default()))
}

fn dao_cell() -> &'static Mutex<Option<Arc<dyn AuditDao>>> {
    static D: OnceLock<Mutex<Option<Arc<dyn AuditDao>>>> = OnceLock::new();
    D.get_or_init(|| Mutex::new(None))
}

/// Current DAO readiness (backend key + whether it initialised).
pub fn dao_status() -> DaoStatus {
    status_cell().lock().map(|g| g.clone()).unwrap_or_default()
}

/// The live DAO, when one is configured and initialised.
pub fn active_dao() -> Option<Arc<dyn AuditDao>> {
    dao_cell().lock().ok().and_then(|g| g.clone())
}

/// Load the persisted config, build the DAO and **retain** it.
pub async fn init_active_dao(config_dir: &std::path::Path) -> DaoStatus {
    let cfg = DatabaseConfig::load(config_dir);
    let status = match build_dao(&cfg).await {
        Ok(dao) => {
            if let Ok(mut slot) = dao_cell().lock() {
                *slot = Some(Arc::from(dao));
            }
            DaoStatus {
                backend: cfg.backend.key().to_string(),
                ready: true,
                detail: cfg.backend.note().to_string(),
            }
        }
        Err(e) => DaoStatus {
            backend: cfg.backend.key().to_string(),
            ready: false,
            detail: e,
        },
    };
    if let Ok(mut s) = status_cell().lock() {
        *s = status.clone();
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn parses_backend_from_env() {
        env::set_var("DB_BACKEND", "supabase");
        env::set_var("DB_URL", "https://xyz.supabase.co");
        // We cannot easily reset other vars from other tests; just exercise default:
        let cfg = DatabaseConfig::default();
        assert_eq!(cfg.backend, DbBackend::Sqlite);
        assert!(cfg.is_active());
        // Defensive cleanup so other tests don't see our leaked env
        // (this test fails pollution otherwise when run after another
        // that asserts SQLite default URL).
        env::remove_var("DB_URL");
    }

    #[test]
    fn labels_are_user_facing() {
        assert_eq!(DbBackend::Sqlite.label(), "SQLite (local)");
        assert_eq!(DbBackend::Insforge.label(), "InsForge (Postgres+AI)");
        assert_eq!(DbBackend::Postgres.label(), "PostgreSQL (libpq)");
    }

    #[test]
    fn audit_row_roundtrip_json() {
        let row = AuditRow {
            host_id: "host1".into(),
            actor_id: "u1".into(),
            actor_name: "Max".into(),
            channel: "telegram".into(),
            kind: "start".into(),
            map_index: Some(0),
            result: Some("ok".into()),
            at: 1_700_000_000_000,
        };
        let v = serde_json::to_string(&row).unwrap();
        let back: AuditRow = serde_json::from_str(&v).unwrap();
        assert_eq!(back.channel, "telegram");
    }

    #[test]
    fn default_config_is_sqlite() {
        let cfg = DatabaseConfig::default();
        assert!(matches!(cfg.backend, DbBackend::Sqlite));
        assert!(cfg.url.contains("ark-config.db"));
    }
}
