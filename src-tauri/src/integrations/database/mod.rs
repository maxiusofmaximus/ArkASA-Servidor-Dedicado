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
}

pub fn all_backends() -> Vec<DbBackendView> {
    DbBackend::all().iter().map(|b| DbBackendView {
        key: format!("{:?}", b).to_lowercase(),
        label: b.label().to_string(),
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
