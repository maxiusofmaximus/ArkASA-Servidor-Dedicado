//! HTTP-backed AuditDao implementations for BaaS providers.
//!
//! Each provider exposes a different REST shape and its own auth flavour:
//!   - Convex    : POST `/{deployment}/api/mutation` body `{path, args}` Bearer
//!   - Supabase  : PostgREST `POST {url}/rest/v1/{table}` Bearer
//!   - InsForge  : InsForge SDK-shaped POST `/api/database/records` Bearer
//!   - Postgres  : PostgREST compatibility (use the same shape as Supabase)
//!   - MongoDB   : Atlas Data API POST `/action/insertOne` + Mongo API Key
//!
//! All have the same async signature so callers stay unaware.

use super::{AuditDao, AuditRow, DatabaseConfig};
use serde_json::json;
use std::time::Duration;

#[derive(Clone)]
struct Http {
    base_url: String,
    api_key:  String,
    table:    String,
    schema:   String,
    client:   reqwest::Client,
    mode:     RemoteMode,
}

#[derive(Clone, Copy, Debug)]
enum RemoteMode {
    Convex,
    Supabase,
    Insforge,
    Postgrest,
    MongoDataApi,
}

impl Http {
    fn new(config: DatabaseConfig, mode: RemoteMode) -> Self {
        Self {
            base_url: config.url.trim_end_matches('/').to_string(),
            api_key:  config.api_key,
            table:    config.table,
            schema:   config.schema,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build().expect("http"),
            mode,
        }
    }

    fn header_pairs(&self) -> Vec<(String, String)> {
        let mut h: Vec<(String, String)> = Vec::new();
        if !self.api_key.is_empty() {
            h.push(("Authorization".into(), format!("Bearer {}", self.api_key)));
        }
        match self.mode {
            RemoteMode::Supabase | RemoteMode::Postgrest => {
                h.push(("apikey".into(), self.api_key.clone()));
                h.push(("Content-Type".into(), "application/json".into()));
                h.push(("Prefer".into(), "return=minimal".into()));
            }
            RemoteMode::Convex | RemoteMode::Insforge | RemoteMode::MongoDataApi => {
                h.push(("Content-Type".into(), "application/json".into()));
            }
        }
        h
    }

    async fn insert(&self, row: &AuditRow) -> Result<(), String> {
        let row_json = serde_json::to_value(row).map_err(|e| e.to_string())?;
        let (url, body) = match self.mode {
            RemoteMode::Convex => {
                (
                    format!("{}/api/mutation", self.base_url),
                    json!({
                        "path": "command_log:append",
                        "args": row
                    }),
                )
            }
            RemoteMode::Supabase | RemoteMode::Postgrest => {
                (
                    format!("{}/rest/v1/{}", self.base_url, self.table),
                    row_json,
                )
            }
            RemoteMode::Insforge => {
                (
                    format!("{}/api/database/records/insert", self.base_url),
                    json!({
                        "table": self.table,
                        "data": row,
                    }),
                )
            }
            RemoteMode::MongoDataApi => {
                (
                    format!("{}/action/insertOne", self.base_url),
                    json!({
                        "dataSource": "Cluster0",
                        "database": self.schema,
                        "collection": self.table,
                        "document": row,
                    }),
                )
            }
        };

        let mut req = self.client.post(&url);
        for (k, v) in self.header_pairs() {
            req = req.header(&k, &v);
        }
        let resp = req.json(&body).send().await.map_err(|e| e.to_string())?;
        let status = resp.status();
        if !status.is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, txt));
        }
        Ok(())
    }

    async fn query_recent(&self, limit: u32, channel: Option<&str>)
        -> Result<Vec<AuditRow>, String>
    {
        let select_cols = "host_id,actor_id,actor_name,channel,kind,map_index,result,at";
        let extra = match channel {
            Some(c) => format!("&channel=eq.{}", c),
            None => "".to_string(),
        };
        let (url, body, verb) = match self.mode {
            RemoteMode::Supabase | RemoteMode::Postgrest => {
                (
                    format!("{}/rest/v1/{}?select={}&order=at.desc&limit={}{}",
                        self.base_url, self.table, select_cols, limit, extra),
                    None,
                    "GET",
                )
            }
            RemoteMode::Convex => {
                (
                    format!("{}/api/query", self.base_url),
                    Some(json!({
                        "path": "command_log:recent",
                        "args": { "limit": limit, "channel": channel.unwrap_or("") }
                    })),
                    "POST",
                )
            }
            RemoteMode::Insforge => {
                (
                    format!("{}/api/database/records/select", self.base_url),
                    Some(json!({
                        "table": self.table,
                        "select": select_cols,
                        "order": "at DESC",
                        "limit": limit as i64,
                        "channel": channel.unwrap_or(""),
                    })),
                    "POST",
                )
            }
            RemoteMode::MongoDataApi => {
                (
                    format!("{}/action/find", self.base_url),
                    Some(json!({
                        "dataSource": "Cluster0",
                        "database": self.schema,
                        "collection": self.table,
                        "filter": {},
                        "limit": limit,
                        "sort": { "at": -1 },
                    })),
                    "POST",
                )
            }
        };

        let mut req = match verb {
            "GET"  => self.client.get(&url),
            _      => self.client.post(&url),
        };
        for (k, v) in self.header_pairs() {
            req = req.header(&k, &v);
        }
        if let Some(b) = body { req = req.json(&b); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        let status = resp.status();
        if !status.is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, txt));
        }
        let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let arr = v
            .get("value").cloned()
            .or_else(|| v.get("documents").cloned())
            .or_else(|| v.as_array().cloned().map(serde_json::Value::Array))
            .unwrap_or_else(|| json!([]));
        let mut out: Vec<AuditRow> = Vec::new();
        if let Some(arr) = arr.as_array() {
            for el in arr {
                if let Ok(row) = serde_json::from_value::<AuditRow>(el.clone()) {
                    out.push(row);
                }
            }
        }
        Ok(out)
    }
}

pub struct HttpDao {
    inner: Http,
}

impl HttpDao {
    pub fn convex(cfg: DatabaseConfig)   -> Self { Self { inner: Http::new(cfg, RemoteMode::Convex) } }
    pub fn supabase(cfg: DatabaseConfig) -> Self { Self { inner: Http::new(cfg, RemoteMode::Supabase) } }
    pub fn insforge(cfg: DatabaseConfig) -> Self { Self { inner: Http::new(cfg, RemoteMode::Insforge) } }
    pub fn postrest(cfg: DatabaseConfig) -> Self { Self { inner: Http::new(cfg, RemoteMode::Postgrest) } }
    pub fn mockcosm(cfg: DatabaseConfig) -> Self { Self { inner: Http::new(cfg, RemoteMode::MongoDataApi) } }
}

#[async_trait::async_trait]
impl AuditDao for HttpDao {
    async fn append(&self, row: AuditRow) -> Result<(), String> {
        self.inner.insert(&row).await
    }
    async fn recent(&self, limit: u32, channel: Option<&str>)
        -> Result<Vec<AuditRow>, String>
    {
        self.inner.query_recent(limit, channel).await
    }
    async fn by_host(&self, host_id: &str, limit: u32)
        -> Result<Vec<AuditRow>, String>
    {
        let url = format!("{}/rest/v1/{}?select=*&host_id=eq.{host_id}&order=at.desc&limit={limit}",
            self.inner.base_url, self.inner.table);
        let mut req = self.inner.client.get(&url);
        for (k, v) in self.inner.header_pairs() { req = req.header(&k, &v); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        if let Some(arr) = v.as_array() {
            for el in arr {
                if let Ok(row) = serde_json::from_value::<AuditRow>(el.clone()) {
                    out.push(row);
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with(backend: &str) -> DatabaseConfig {
        DatabaseConfig {
            backend: match backend {
                "convex"  => super::super::DbBackend::Convex,
                "sbstor"  => super::super::DbBackend::Supabase,
                "insf"    => super::super::DbBackend::Insforge,
                "pgrest"  => super::super::DbBackend::Postgres,
                "mongo"   => super::super::DbBackend::Mongodb,
                _         => super::super::DbBackend::Sqlite,
            },
            url: "https://example.com".into(),
            api_key: "test-key".into(),
            schema: "public".into(),
            table: "command_log".into(),
        }
    }

    #[test]
    fn builds_singleton_each() {
        let _ = HttpDao::convex(cfg_with("convex"));
        let _ = HttpDao::supabase(cfg_with("sbstor"));
        let _ = HttpDao::insforge(cfg_with("insf"));
        let _ = HttpDao::postrest(cfg_with("pgrest"));
        let _ = HttpDao::mockcosm(cfg_with("mongo"));
    }

    #[test]
    fn headers_set_apikey_when_present() {
        let mut cfg = cfg_with("sbstor");
        cfg.api_key = "MYKEY".into();
        let h = Http::new(cfg, RemoteMode::Supabase);
        let headers = h.header_pairs();
        assert!(headers.iter().any(|(k, _)| k == "apikey"));
        assert!(headers.iter().any(|(k, _)| k == "Authorization"));
    }
}
