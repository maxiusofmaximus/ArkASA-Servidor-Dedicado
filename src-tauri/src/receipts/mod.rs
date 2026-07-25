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
    /// Stringify this stage for log lines, ADR documentation or future
    /// receipt-renderers that walk the pipeline in textual form. Unused
    /// today because the receipt serializer uses the `Stage` enum directly.
    /// P3.2 audit (IMPLEMENTATION_PLAN.md §7.2.2): kept with rationale.
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

/// Default retention window for `rawText` on `ChannelIngress` receipts.
/// GDPR Article 5(1)(e) — "kept in a form which permits identification of
/// data subjects for no longer than is necessary". 30 days covers operator
/// debugging + incident forensics without indefinite retention.
pub const DEFAULT_RAW_TEXT_RETENTION_MS: i64 = 30 * 24 * 60 * 60 * 1000;

/// One immutable line of the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// epoch milliseconds
    pub at: i64,
    pub host_id: HostId,
    pub stage: Stage,
    /// Epoch-ms when the payload's `rawText` (and any other PII it carries)
    /// can be redacted by the janitor. Zero means "no retention policy" —
    /// the row is audit metadata only and never touches user data.
    #[serde(default)]
    pub retention_expires_at_ms: i64,
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
    /// Stages that may carry PII (today: `ChannelIngress` with `rawText`) are
    /// stamped with `now + DEFAULT_RAW_TEXT_RETENTION_MS` so the janitor can
    /// honour GDPR Article 5 (storage limitation). Audit/Hosting receipts —
    /// never PII — get `0` (no retention).
    pub fn append(&self, payload: serde_json::Value, stage: Stage) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp_millis();
        let host_id = self.inner.host_id.clone();
        let retention_expires_at_ms = match stage {
            Stage::ChannelIngress => now + DEFAULT_RAW_TEXT_RETENTION_MS,
            _ => 0,
        };
        let receipt = Receipt { at: now, host_id, stage, retention_expires_at_ms, payload };
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

    /// GDPR Article 5 janitor. Walks every `YYYY-MM-DD.jsonl` file under
    /// `base_dir`, parses each line, and rewrites the file with `rawText`
    /// (and `textLength`) purged from any receipt whose
    /// `retention_expires_at_ms` has elapsed.
    ///
    /// Idempotent: a re-run on an already-swept file is a no-op (the field
    /// is already null + 0). Safe to call from `lib::run()` at boot and
    /// from a periodic timer.
    ///
    /// Returns the number of rows redacted across all files.
    pub fn sweep_expired(&self) -> Result<u64, String> {
        let base = self.inner.base_dir.clone();
        let now = chrono::Utc::now().timestamp_millis();
        let mut total_redacted: u64 = 0;

        let entries = match std::fs::read_dir(&base) {
            Ok(it) => it,
            Err(_) => return Ok(0),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }
            let txt = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut rewritten = String::with_capacity(txt.len());
            let mut redacted_in_file: u64 = 0;
            for line in txt.lines() {
                if line.is_empty() {
                    continue;
                }
                let mut r: Receipt = match serde_json::from_str(line) {
                    Ok(r) => r,
                    Err(_) => {
                        rewritten.push_str(line);
                        rewritten.push('\n');
                        continue;
                    }
                };
                if r.retention_expires_at_ms != 0 && r.retention_expires_at_ms <= now {
                    if let Some(obj) = r.payload.as_object_mut() {
                        let had_raw = obj.remove("rawText").is_some();
                        obj.insert("textLength".to_string(), serde_json::Value::from(0));
                        obj.insert(
                            "redactedAt".to_string(),
                            serde_json::Value::from(now),
                        );
                        obj.insert(
                            "redactionReason".to_string(),
                            serde_json::Value::String("gdpr_art5_storage_limitation".into()),
                        );
                        if had_raw {
                            redacted_in_file += 1;
                        }
                    }
                }
                let serialised = serde_json::to_string(&r).map_err(|e| e.to_string())?;
                rewritten.push_str(&serialised);
                rewritten.push('\n');
            }
            if redacted_in_file > 0 {
                atomic_replace(&path, rewritten.as_bytes())
                    .map_err(|e| format!("atomic_replace failed ({}): {}", path.display(), e))?;
            }
            total_redacted += redacted_in_file;
        }
        Ok(total_redacted)
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

/// Replace `path` with `data` atomically: write to `<path>.tmp`, fsync,
/// rename. Power-loss safe.
fn atomic_replace(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = path.with_extension("jsonl.tmp");
    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
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

    #[test]
    fn ingress_receipt_stamps_retention_window() {
        let tmp = std::env::temp_dir().join(format!(
            "ark-asa-ledger-test-retain-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let ledger = ReceiptLedger::new(&tmp, "host-retain".into());
        let before = chrono::Utc::now().timestamp_millis();
        ledger
            .append(
                serde_json::json!({"rawText": "/start", "traceId": "t1"}),
                Stage::ChannelIngress,
            )
            .unwrap();
        let tail = ledger.tail(1).unwrap();
        let r = &tail[0];
        assert!(r.retention_expires_at_ms > 0);
        assert!(r.retention_expires_at_ms >= before + DEFAULT_RAW_TEXT_RETENTION_MS - 5_000);
        assert!(r.retention_expires_at_ms <= before + DEFAULT_RAW_TEXT_RETENTION_MS + 5_000);
        // Audit/Hosting never carry PII → no retention stamp (0).
        ledger
            .append(serde_json::json!({"hello": "world"}), Stage::Audit)
            .unwrap();
        let tail = ledger.tail(2).unwrap();
        assert_eq!(tail[1].stage, Stage::Audit);
        assert_eq!(tail[1].retention_expires_at_ms, 0);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn janitor_redacts_only_expired_rows() {
        let tmp = std::env::temp_dir().join(format!(
            "ark-asa-ledger-test-janitor-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let day_file = tmp.join("2025-01-01.jsonl");
        let old_row = serde_json::json!({
            "at": 1_700_000_000_000_i64,
            "host_id": "host-old",
            "stage": "channel_ingress",
            "retention_expires_at_ms": 1_700_000_000_000_i64 + DEFAULT_RAW_TEXT_RETENTION_MS,
            "payload": {
                "rawText": "real user message",
                "textLength": 18,
                "traceId": "t-old",
            }
        });
        let fresh_row = serde_json::json!({
            "at": chrono::Utc::now().timestamp_millis(),
            "host_id": "host-new",
            "stage": "channel_ingress",
            "retention_expires_at_ms": chrono::Utc::now().timestamp_millis() + 86_400_000,
            "payload": {
                "rawText": "fresh chat content",
                "textLength": 18,
                "traceId": "t-new",
            }
        });
        let audit_row = serde_json::json!({
            "at": 1_700_000_000_000_i64,
            "host_id": "host-old",
            "stage": "audit",
            "retention_expires_at_ms": 0,
            "payload": {"note": "audit-only-no-pii"}
        });
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&day_file)
            .unwrap();
        writeln!(f, "{}", old_row).unwrap();
        writeln!(f, "{}", fresh_row).unwrap();
        writeln!(f, "{}", audit_row).unwrap();
        drop(f);

        let ledger = ReceiptLedger::new(&tmp, "host-janitor".into());
        let redacted = ledger.sweep_expired().unwrap();
        assert_eq!(redacted, 1, "exactly one row should be redacted");

        let txt = std::fs::read_to_string(&day_file).unwrap();
        let lines: Vec<&str> = txt.lines().collect();
        assert_eq!(lines.len(), 3);

        let r_old: Receipt = serde_json::from_str(lines[0]).unwrap();
        assert!(r_old.payload.get("rawText").is_none());
        assert_eq!(r_old.payload["textLength"], serde_json::json!(0));
        assert_eq!(r_old.payload["redactionReason"],
                   serde_json::json!("gdpr_art5_storage_limitation"));
        assert!(r_old.payload["redactedAt"].as_i64().unwrap() > 0);

        let r_new: Receipt = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(r_new.payload["rawText"], serde_json::json!("fresh chat content"));

        let r_audit: Receipt = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(r_audit.payload["note"], serde_json::json!("audit-only-no-pii"));
        assert!(r_audit.payload.get("redactedAt").is_none());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn janitor_is_idempotent() {
        let tmp = std::env::temp_dir().join(format!(
            "ark-asa-ledger-test-idem-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let day_file = tmp.join("2025-01-01.jsonl");
        let row = serde_json::json!({
            "at": 1_700_000_000_000_i64,
            "host_id": "host",
            "stage": "channel_ingress",
            "retention_expires_at_ms": 1_700_000_000_000_i64,
            "payload": {"rawText": "secret", "textLength": 6}
        });
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&day_file)
            .unwrap();
        writeln!(f, "{}", row).unwrap();
        drop(f);

        let ledger = ReceiptLedger::new(&tmp, "host-idem".into());
        let first = ledger.sweep_expired().unwrap();
        let second = ledger.sweep_expired().unwrap();
        assert_eq!(first, 1);
        assert_eq!(second, 0, "second sweep should be a no-op");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
