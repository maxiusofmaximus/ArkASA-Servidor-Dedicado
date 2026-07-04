//! SQLite audit log DAO. The default local backend — no network, no key.
//!
//! The DESKTOP app keeps one SQLite file per host. A cluster operator running
//! on a VPS can choose this backend or sync it via `litestream` to S3 for
//! cloud resilience, mirroring the same backend used by the database admin
//! the partner chose.

use super::{AuditDao, AuditRow};
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;

pub struct SqliteDao {
    // cloned-on-access via Arc<Mutex<Connection>>. Avoids holding a single
    // Mutex guard across async boundaries (which never works under Tauri).
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteDao {
    /// Open (or create) at `path`. Initialises schema on first run.
    pub async fn open(path: &str) -> Result<Self, String> {
        let owned = path.to_string();
        let conn = tokio::task::spawn_blocking(move || -> Result<rusqlite::Connection, String> {
            if let Some(parent) = Path::new(&owned).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
            }
            let conn = rusqlite::Connection::open(&owned).map_err(|e| e.to_string())?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS command_log (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    host_id     TEXT NOT NULL,
                    actor_id    TEXT NOT NULL,
                    actor_name  TEXT NOT NULL,
                    channel     TEXT NOT NULL,
                    kind        TEXT NOT NULL,
                    map_index   INTEGER,
                    result      TEXT,
                    at          INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_command_log_at ON command_log(at DESC);
                CREATE INDEX IF NOT EXISTS idx_command_log_host ON command_log(host_id, at DESC);
                "
            ).map_err(|e| e.to_string())?;
            Ok(conn)
        })
        .await
        .map_err(|e| e.to_string())??;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }
}

#[async_trait::async_trait]
impl AuditDao for SqliteDao {
    async fn append(&self, row: AuditRow) -> Result<(), String> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let g = conn.lock();
            g.execute(
                "INSERT INTO command_log
                 (host_id, actor_id, actor_name, channel, kind, map_index, result, at)
                 VALUES (?,?,?,?,?,?,?,?)",
                rusqlite::params![
                    row.host_id, row.actor_id, row.actor_name, row.channel,
                    row.kind, row.map_index, row.result, row.at,
                ],
            ).map_err(|e| e.to_string())?;
            Ok(())
        }).await.map_err(|e| e.to_string())?
    }

    async fn recent(&self, limit: u32, channel: Option<&str>)
        -> Result<Vec<AuditRow>, String>
    {
        let conn = Arc::clone(&self.conn);
        let channel_owned = channel.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || -> Result<Vec<AuditRow>, String> {
            let conn = conn.lock();
            let mut out: Vec<AuditRow> = Vec::new();
            let mapper = |r: &rusqlite::Row| -> rusqlite::Result<AuditRow> {
                Ok(AuditRow {
                    host_id:    r.get(0)?,
                    actor_id:   r.get(1)?,
                    actor_name: r.get(2)?,
                    channel:    r.get(3)?,
                    kind:       r.get(4)?,
                    map_index:  r.get(5)?,
                    result:     r.get(6)?,
                    at:         r.get(7)?,
                })
            };
            match &channel_owned {
                Some(c) => {
                    let mut stmt = conn.prepare(
                        "SELECT host_id, actor_id, actor_name, channel, kind, map_index, result, at
                         FROM command_log WHERE channel = ? ORDER BY at DESC LIMIT ?"
                    ).map_err(|e| e.to_string())?;
                    let rows = stmt.query_map(
                        rusqlite::params![c, limit as i64], mapper,
                    ).map_err(|e| e.to_string())?;
                    for row in rows { out.push(row.map_err(|e| e.to_string())?); }
                }
                None => {
                    let mut stmt = conn.prepare(
                        "SELECT host_id, actor_id, actor_name, channel, kind, map_index, result, at
                         FROM command_log ORDER BY at DESC LIMIT ?"
                    ).map_err(|e| e.to_string())?;
                    let rows = stmt.query_map(
                        rusqlite::params![limit as i64], mapper,
                    ).map_err(|e| e.to_string())?;
                    for row in rows { out.push(row.map_err(|e| e.to_string())?); }
                }
            }
            Ok(out)
        }).await.map_err(|e| e.to_string())?
    }

    async fn by_host(&self, host_id: &str, limit: u32)
        -> Result<Vec<AuditRow>, String>
    {
        let conn = Arc::clone(&self.conn);
        let host = host_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<AuditRow>, String> {
            let conn = conn.lock();
            let mut out: Vec<AuditRow> = Vec::new();
            let mut stmt = conn.prepare(
                "SELECT host_id, actor_id, actor_name, channel, kind, map_index, result, at
                 FROM command_log WHERE host_id = ? ORDER BY at DESC LIMIT ?"
            ).map_err(|e| e.to_string())?;
            let rows = stmt.query_map(
                rusqlite::params![host, limit as i64],
                |r| rusqlite::Result::Ok(AuditRow {
                    host_id:    r.get(0)?,
                    actor_id:   r.get(1)?,
                    actor_name: r.get(2)?,
                    channel:    r.get(3)?,
                    kind:       r.get(4)?,
                    map_index:  r.get(5)?,
                    result:     r.get(6)?,
                    at:         r.get(7)?,
                }),
            ).map_err(|e| e.to_string())?;
            for row in rows { out.push(row.map_err(|e| e.to_string())?); }
            Ok(out)
        }).await.map_err(|e| e.to_string())?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test(flavor = "current_thread")]
    async fn create_and_query_table() {
        let tmp = env::temp_dir().join(format!("ark-asa-test-{}.db",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let dao = SqliteDao::open(tmp.to_string_lossy().as_ref()).await.unwrap();
        dao.append(AuditRow {
            host_id: "h1".into(),
            actor_id: "tg-1".into(),
            actor_name: "Max".into(),
            channel: "telegram".into(),
            kind: "start".into(),
            map_index: Some(0),
            result: Some("ok".into()),
            at: chrono::Utc::now().timestamp_millis(),
        }).await.unwrap();
        let rows = dao.recent(10, Some("telegram")).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].channel, "telegram");

        // Cleanup
        let _ = std::fs::remove_file(&tmp);
    }
}
