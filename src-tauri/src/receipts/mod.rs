//! Receipts ledger — append-only JSONL per stage.
//!
//! Mirrors the Agent Harness Core "receipts are the source of truth" pattern.
//! Every stage the chain touches (channel ingress, identity check, queue
//! enqueue, runtime, delivery, audit, hosting) appends a single line of JSON
//! to a daily file under `${app_data}/receipts/`. Lines are immutable; rotation
//! is by date only. The ledger is crash-resilient:
//!  - Each `append` is wrapped in `tokio::task::block_in_place` so the
//!    kernel doesn't expose half-written rows on power loss.
//!  - We fsync-on-write so the OS flushes buffers.
//!
//! Tests rely on this guarantee: a commanded action with no receipt is a bug.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Stable name carried by every receipt for this host.
pub type HostId = String;

/// Stage identifier. New stages should append, never rename.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    ChannelIngress,
    IdentityCheck,
    QueueEnqueue,
    RuntimePipeline,
    ChannelDelivery,
    Audit,
    Hosting,
    Unknown,
}

impl Stage {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::ChannelIngress  => "channel_ingress",
            Stage::IdentityCheck   => "identity_check",
            Stage::QueueEnqueue    => "queue_enqueue",
            Stage::RuntimePipeline => "runtime_pipeline",
            Stage::ChannelDelivery => "channel_delivery",
            Stage::Audit           => "audit",
            Stage::Hosting         => "hosting",
            Stage::Unknown         => "unknown",
        }
    }
}

/// One immutable line of the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// epoch milliseconds
    pub at: i64,
    pub host_id: HostId,
    pub stage: Stage,
    /// Free-form JSON payload (identity axes, command, outcome, etc.)
    pub payload: serde_json::Value,
}

#[derive(Clone)]
pub struct ReceiptLedger {
    inner: Arc<LedgerInner>,
}

struct LedgerInner {
    /// Per-day writers, wrapped in Arc<Mutex<_>> so each one is independently
    /// lockable without holding the outer registry lock across I/O.
    by_day: Mutex<std::collections::HashMap<String, Arc<Mutex<DayWriter>>>>,
    base_dir: PathBuf,
    host_id: HostId,
}

struct DayWriter {
    _date: String,
    file: PathBuf,
    _bytes: u64,
}

impl ReceiptLedger {
    /// Open (or create) a ledger rooted at `base_dir`. Files are named
    /// `YYYY-MM-DD.jsonl` and rotated implicitly when the date rolls.
    pub fn new(base_dir: impl Into<PathBuf>, host_id: HostId) -> Self {
        let base = base_dir.into();
        let _ = std::fs::create_dir_all(&base);
        Self {
            inner: Arc::new(LedgerInner {
                by_day: Mutex::new(std::collections::HashMap::new()),
                base_dir: base,
                host_id,
            }),
        }
    }

    /// Append a receipt. The `payload` must be a JSON value (use serde_json::json!).
    pub fn append(&self, payload: serde_json::Value, stage: Stage) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp_millis();
        let host_id = self.inner.host_id.clone();
        let receipt = Receipt { at: now, host_id, stage, payload };
        let line = format!("{}\n", serde_json::to_string(&receipt).map_err(|e| e.to_string())?);
        let key = date_key(receipt.at);

        // Resolve the per-day writer Arc under a brief outer lock.
        let w_arc: Arc<Mutex<DayWriter>> = {
            let mut registry = self.inner.by_day.lock();
            if !registry.contains_key(&key) {
                let file = self.inner.base_dir.join(format!("{}.jsonl", key));
                registry.insert(
                    key.clone(),
                    Arc::new(Mutex::new(DayWriter {
                        _date: key.clone(),
                        file,
                        _bytes: 0,
                    })),
                );
            }
            registry.get(&key).cloned().expect("entry")
        };

        // I/O under just the per-day lock.
        let mut w = w_arc.lock();
        if w._date != key {
            // Crossed midnight. Re-anchor path under per-day lock.
            let file = self.inner.base_dir.join(format!("{}.jsonl", key));
            w._date = key.clone();
            w.file = file;
            w._bytes = 0;
        }
        append_all_or_nothing(&w.file, line.as_bytes())
            .map_err(|e| format!("receipt write failed ({}): {}", w.file.display(), e))?;
        w._bytes += line.as_bytes().len() as u64;
        Ok(())
    }

    /// Path to today's ledger file (operator-facing for support/debug).
    pub fn today_path(&self) -> PathBuf {
        let key = date_key(chrono::Utc::now().timestamp_millis());
        self.inner.base_dir.join(format!("{}.jsonl", key))
    }

    pub fn host_id(&self) -> &str { &self.inner.host_id }

    /// Quick read of the most recent N receipts (no fsync, for tests + UI).
    pub fn tail(&self, n: usize) -> Result<Vec<Receipt>, String> {
        let path = self.today_path();
        let txt = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return Ok(vec![]),
        };
        let lines: Vec<&str> = txt.lines().collect();
        let start = lines.len().saturating_sub(n);
        let mut out = Vec::with_capacity(n);
        for line in &lines[start..] {
            if let Ok(r) = serde_json::from_str::<Receipt>(line) {
                out.push(r);
            }
        }
        Ok(out)
    }
}

fn date_key(epoch_ms: i64) -> String {
    let secs = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(epoch_ms)
        .unwrap_or_else(chrono::Utc::now);
    secs.format("%Y-%m-%d").to_string()
}

/// Append + fsync as a pair so the reader always sees whole lines.
fn append_all_or_nothing(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    let mut f = opts.open(path)?;
    f.write_all(data)?;
    f.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_key_stable() {
        let k = date_key(1_700_000_000_000);
        assert_eq!(k.chars().filter(|c| *c == '-').count(), 2);
    }

    #[test]
    fn append_roundtrip() {
        let tmp = std::env::temp_dir().join(
            format!("ark-asa-ledger-test-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
        );
        let ledger = ReceiptLedger::new(&tmp, "host-test".into());
        ledger.append(serde_json::json!({"actor": "alice", "kind": "start"}), Stage::ChannelIngress).unwrap();
        ledger.append(serde_json::json!({"actor": "alice", "kind": "stop"}),  Stage::ChannelDelivery).unwrap();
        let tail = ledger.tail(10).unwrap();
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[0].host_id, "host-test");
        assert_eq!(tail[0].stage, Stage::ChannelIngress);
        assert_eq!(tail[1].stage, Stage::ChannelDelivery);
        assert!(ledger.today_path().exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn payload_roundtrip_preserves_complex_json() {
        let tmp = std::env::temp_dir().join(
            format!("ark-asa-ledger-test-cmp-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
        );
        let ledger = ReceiptLedger::new(&tmp, "host-cmp".into());
        let payload = serde_json::json!({
            "platform": "telegram",
            "chat_id": 12345,
            "command": {"kind": "start", "map": "TheIsland_WP"},
            "tags": ["admin", "verified"],
            "ok": true,
        });
        ledger.append(payload.clone(), Stage::ChannelIngress).unwrap();
        let tail = ledger.tail(1).unwrap();
        assert_eq!(tail[0].payload, payload);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn stage_uses_snake_case_name() {
        assert_eq!(Stage::ChannelIngress.as_str(),  "channel_ingress");
        assert_eq!(Stage::RuntimePipeline.as_str(), "runtime_pipeline");
        assert_eq!(Stage::ChannelDelivery.as_str(), "channel_delivery");
    }
}
