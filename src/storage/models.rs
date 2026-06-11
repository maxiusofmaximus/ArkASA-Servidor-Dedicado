use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigSnapshot {
    pub id: String,
    pub version: i32,
    pub config_json: String,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub changes: Option<String>,
    pub status: String,
    pub user: String,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerActivityLog {
    pub id: String,
    pub event_type: String,
    pub message: String,
    pub severity: String,
    pub created_at: DateTime<Utc>,
}

impl ConfigSnapshot {
    pub fn new(
        version: i32,
        config_json: String,
        checksum: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version,
            config_json,
            checksum,
            created_at: Utc::now(),
            created_by: "system".to_string(),
            description,
        }
    }
}

impl AuditLog {
    pub fn new(
        action: String,
        resource_type: String,
        resource_id: String,
        status: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action,
            resource_type,
            resource_id,
            changes: None,
            status,
            user: "system".to_string(),
            ip_address: None,
            created_at: Utc::now(),
        }
    }
}

impl ServerActivityLog {
    pub fn new(event_type: String, message: String, severity: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            message,
            severity,
            created_at: Utc::now(),
        }
    }
}
