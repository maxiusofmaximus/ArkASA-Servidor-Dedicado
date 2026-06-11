use crate::error::Result;
use crate::storage::models::{ConfigSnapshot, AuditLog, ServerActivityLog};
use sqlx::SqlitePool;

pub struct ConfigRepository {
    pool: SqlitePool,
}

impl ConfigRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn save_snapshot(&self, snapshot: &ConfigSnapshot) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO config_snapshots (id, version, config_json, checksum, created_at, created_by, description)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&snapshot.id)
        .bind(snapshot.version)
        .bind(&snapshot.config_json)
        .bind(&snapshot.checksum)
        .bind(snapshot.created_at.to_rfc3339())
        .bind(&snapshot.created_by)
        .bind(&snapshot.description)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        log::info!("Config snapshot saved: version {}", snapshot.version);
        Ok(())
    }

    pub async fn get_latest_snapshot(&self) -> Result<Option<ConfigSnapshot>> {
        let snapshot = sqlx::query_as::<_, ConfigSnapshot>(
            "SELECT id, version, config_json, checksum, created_at, created_by, description FROM config_snapshots ORDER BY version DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(snapshot)
    }

    pub async fn get_snapshot_by_version(&self, version: i32) -> Result<Option<ConfigSnapshot>> {
        let snapshot = sqlx::query_as::<_, ConfigSnapshot>(
            "SELECT id, version, config_json, checksum, created_at, created_by, description FROM config_snapshots WHERE version = ?"
        )
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(snapshot)
    }

    pub async fn list_snapshots(&self, limit: i32) -> Result<Vec<ConfigSnapshot>> {
        let snapshots = sqlx::query_as::<_, ConfigSnapshot>(
            "SELECT id, version, config_json, checksum, created_at, created_by, description FROM config_snapshots ORDER BY version DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(snapshots)
    }

    pub async fn log_audit_event(&self, log: &AuditLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, action, resource_type, resource_id, changes, status, user, ip_address, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&log.id)
        .bind(&log.action)
        .bind(&log.resource_type)
        .bind(&log.resource_id)
        .bind(&log.changes)
        .bind(&log.status)
        .bind(&log.user)
        .bind(&log.ip_address)
        .bind(log.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(())
    }

    pub async fn get_audit_logs(&self, limit: i32) -> Result<Vec<AuditLog>> {
        let logs = sqlx::query_as::<_, AuditLog>(
            "SELECT id, action, resource_type, resource_id, changes, status, user, ip_address, created_at FROM audit_logs ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(logs)
    }

    pub async fn log_server_activity(&self, log: &ServerActivityLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO server_activity_logs (id, event_type, message, severity, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&log.id)
        .bind(&log.event_type)
        .bind(&log.message)
        .bind(&log.severity)
        .bind(log.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(())
    }

    pub async fn get_activity_logs(&self, limit: i32) -> Result<Vec<ServerActivityLog>> {
        let logs = sqlx::query_as::<_, ServerActivityLog>(
            "SELECT id, event_type, message, severity, created_at FROM server_activity_logs ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(logs)
    }

    pub async fn get_next_version(&self) -> Result<i32> {
        let result: (Option<i32>,) = sqlx::query_as("SELECT MAX(version) FROM config_snapshots")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(result.0.unwrap_or(0) + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::Database;

    #[tokio::test]
    async fn test_save_and_get_snapshot() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        let repo = ConfigRepository::new(db.get_pool().clone());

        let snapshot = ConfigSnapshot::new(
            1,
            r#"{"test": "config"}"#.to_string(),
            "abc123".to_string(),
            Some("Test snapshot".to_string()),
        );

        assert!(repo.save_snapshot(&snapshot).await.is_ok());

        let retrieved = repo.get_latest_snapshot().await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().version, 1);
    }

    #[tokio::test]
    async fn test_get_next_version() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        let repo = ConfigRepository::new(db.get_pool().clone());

        let version = repo.get_next_version().await.unwrap();
        assert_eq!(version, 1);

        let snapshot = ConfigSnapshot::new(
            version,
            "{}".to_string(),
            "abc".to_string(),
            None,
        );
        repo.save_snapshot(&snapshot).await.unwrap();

        let next_version = repo.get_next_version().await.unwrap();
        assert_eq!(next_version, 2);
    }
}
