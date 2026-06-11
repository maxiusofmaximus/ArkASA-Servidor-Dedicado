use crate::error::Result;
use crate::storage::models::{ConfigSnapshot, AuditLog, ServerActivityLog};
use sqlx::SqlitePool;
use chrono::Utc;

pub struct ConfigRepository {
    pool: SqlitePool,
}

impl ConfigRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ============= CONFIG SNAPSHOTS =============

    pub async fn save_snapshot(&self, snapshot: &ConfigSnapshot) -> Result<()> {
        let config_json_str = serde_json::to_string(&snapshot.config_json)
            .map_err(|e| crate::error::Error::SerializationError(e))?;

        sqlx::query(
            r#"
            INSERT INTO config_snapshots
            (id, version, config_json, checksum, created_at, created_by, description, size_bytes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&snapshot.id)
        .bind(snapshot.version)
        .bind(&config_json_str)
        .bind(&snapshot.checksum)
        .bind(snapshot.created_at.to_rfc3339())
        .bind(&snapshot.created_by)
        .bind(&snapshot.description)
        .bind(config_json_str.len() as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        log::info!("Config snapshot saved: version {} by {}", snapshot.version, snapshot.created_by);
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

    pub async fn get_next_version(&self) -> Result<i32> {
        let result: (Option<i32>,) = sqlx::query_as("SELECT MAX(version) FROM config_snapshots")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(result.0.unwrap_or(0) + 1)
    }

    // ============= AUDIT LOGS =============

    pub async fn log_audit_event(&self, log: &AuditLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs
            (id, action, resource_type, resource_id, changes, status, user, ip_address, created_at)
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

        log::debug!("Audit log: {} on {} by {}", log.action, log.resource_type, log.user);
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

    // ============= SERVER ACTIVITY LOGS =============

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

    // ============= METRICS =============

    pub async fn save_metrics(
        &self,
        cpu_percent: Option<f32>,
        memory_mb: Option<i32>,
        player_count: Option<i32>,
        fps: Option<i32>,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO server_metrics
            (id, timestamp, cpu_percent, memory_mb, player_count, fps)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&id)
        .bind(&timestamp)
        .bind(cpu_percent)
        .bind(memory_mb)
        .bind(player_count)
        .bind(fps)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(())
    }

    pub async fn get_recent_metrics(&self, minutes: i32) -> Result<Vec<serde_json::Value>> {
        let cutoff = Utc::now() - chrono::Duration::minutes(minutes as i64);

        let metrics = sqlx::query_as::<_, (String, String, Option<f32>, Option<i32>, Option<i32>, Option<i32>)>(
            "SELECT id, timestamp, cpu_percent, memory_mb, player_count, fps FROM server_metrics WHERE timestamp > ? ORDER BY timestamp DESC"
        )
        .bind(cutoff.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e))?;

        Ok(metrics.into_iter().map(|(_, ts, cpu, mem, players, fps)| {
            serde_json::json!({
                "timestamp": ts,
                "cpu_percent": cpu,
                "memory_mb": mem,
                "player_count": players,
                "fps": fps
            })
        }).collect())
    }

    // ============= CLEANUP =============

    pub async fn cleanup_old_logs(&self, days_to_keep: i32) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(days_to_keep as i64);

        sqlx::query("DELETE FROM server_activity_logs WHERE created_at < ?")
            .bind(cutoff.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(|e| crate::error::Error::DatabaseError(e))?;

        sqlx::query("DELETE FROM server_metrics WHERE timestamp < ?")
            .bind(cutoff.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(|e| crate::error::Error::DatabaseError(e))?;

        log::info!("Cleaned up logs older than {} days", days_to_keep);
        Ok(())
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

    #[tokio::test]
    async fn test_save_metrics() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        let repo = ConfigRepository::new(db.get_pool().clone());

        let result = repo.save_metrics(Some(45.5), Some(2048), Some(5), Some(60)).await;
        assert!(result.is_ok());

        let metrics = repo.get_recent_metrics(60).await.unwrap();
        assert_eq!(metrics.len(), 1);
    }
}
