use crate::error::Result;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| crate::error::Error::DatabaseError(e))?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        // Create config_snapshots table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS config_snapshots (
                id TEXT PRIMARY KEY,
                version INTEGER NOT NULL,
                config_json TEXT NOT NULL,
                checksum TEXT NOT NULL,
                created_at TEXT NOT NULL,
                created_by TEXT NOT NULL,
                description TEXT,
                UNIQUE(version)
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        // Create audit_logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                changes TEXT,
                status TEXT NOT NULL,
                user TEXT NOT NULL,
                ip_address TEXT,
                created_at TEXT NOT NULL,
                INDEX idx_created_at (created_at),
                INDEX idx_resource (resource_type, resource_id)
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        // Create server_activity_logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS server_activity_logs (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                message TEXT NOT NULL,
                severity TEXT NOT NULL,
                created_at TEXT NOT NULL,
                INDEX idx_created_at (created_at),
                INDEX idx_severity (severity)
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        // Create settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        log::info!("Database migrations completed");
        Ok(())
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| crate::error::Error::DatabaseError(e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_migrations_run() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        let pool = db.get_pool();

        // Check if tables exist
        let result: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='config_snapshots'"
        )
        .fetch_one(pool)
        .await
        .unwrap();

        assert_eq!(result.0, 1);
    }
}
