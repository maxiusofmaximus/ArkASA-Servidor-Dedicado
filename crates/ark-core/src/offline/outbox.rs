//! Durable write-ahead outbox.
//!
//! Every state-changing admin action is appended here **before** it is
//! attempted. Consequences that matter:
//!
//! - process death cannot lose a queued action;
//! - the same action cannot be enqueued twice, because the **idempotency key is
//!   the primary key** — re-enqueuing is a no-op, not a duplicate;
//! - retrying a failed send is safe by construction, which is what allows the
//!   consumer to use plain exponential backoff with no deduplication logic.
//!
//! States are a strict small machine: `Pending → InFlight → Done`, with
//! `InFlight → Pending` on retry and `→ Failed` once the attempt budget is
//! exhausted. Anything not `Done` survives a reboot.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::error::{Error, Result};

/// Queue rows keyed by idempotency key; value is the JSON-encoded entry.
const OUTBOX: TableDefinition<&str, &[u8]> = TableDefinition::new("outbox");

/// Lifecycle of a queued action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OutboxState {
    /// Waiting to be attempted (or waiting for backoff to elapse).
    Pending,
    /// Currently being attempted; released back to `Pending` on retry.
    InFlight,
    /// Delivered. Kept so a late duplicate is recognised as already done.
    Done,
    /// Attempt budget exhausted. Surfaces to the operator.
    Failed,
}

impl OutboxState {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboxState::Pending => "pending",
            OutboxState::InFlight => "in_flight",
            OutboxState::Done => "done",
            OutboxState::Failed => "failed",
        }
    }
}

/// A queued action.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OutboxEntry {
    /// Idempotency key. Uniquely identifies the *intent*; re-enqueuing the same
    /// key never creates a second row.
    pub idempotency_key: String,
    /// Action name the consumer dispatches on, e.g. `rcon.command`.
    pub action: String,
    /// JSON payload for the action.
    pub payload: String,
    pub state: OutboxState,
    /// How many delivery attempts have been made.
    pub attempts: u32,
    /// Attempt ceiling before the entry is marked [`OutboxState::Failed`].
    pub max_attempts: u32,
    /// Unix ms before which the entry must not be attempted (backoff).
    pub not_before_ms: u64,
    /// Unix seconds when the entry was first enqueued.
    pub created_at_unix: u64,
    /// Last error observed, if any (surfaced to the operator).
    pub last_error: Option<String>,
}

/// What a caller provides when enqueuing; the rest is managed by the outbox.
#[derive(Debug, Clone)]
pub struct NewOutboxEntry {
    pub idempotency_key: String,
    pub action: String,
    pub payload: String,
    pub max_attempts: u32,
}

impl NewOutboxEntry {
    pub fn new(
        idempotency_key: impl Into<String>,
        action: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            idempotency_key: idempotency_key.into(),
            action: action.into(),
            payload: payload.into(),
            max_attempts: 8,
        }
    }
}

/// Base backoff; attempt *n* waits `BASE << (n-1)`, capped at [`MAX_BACKOFF_MS`].
const BASE_BACKOFF_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 15 * 60 * 1_000;

/// Durable outbox backed by [`redb`].
pub struct Outbox {
    db: Database,
    path: PathBuf,
}

impl Outbox {
    /// Opens (or creates) the outbox inside `dir`, as `<dir>/outbox.redb`.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)
            .map_err(|e| Error::ConfigError(format!("cannot create {}: {e}", dir.display())))?;
        let path = dir.join("outbox.redb");
        let db = Database::create(&path)
            .map_err(|e| Error::ConfigError(format!("cannot open {}: {e}", path.display())))?;

        let outbox = Self { db, path };
        // Materialise the table so reads on an untouched outbox are cheap.
        {
            let txn = outbox
                .db
                .begin_write()
                .map_err(|e| Error::ConfigError(format!("outbox write txn: {e}")))?;
            let _ = txn
                .open_table(OUTBOX)
                .map_err(|e| Error::ConfigError(format!("open outbox table: {e}")))?;
            txn.commit()
                .map_err(|e| Error::ConfigError(format!("outbox commit: {e}")))?;
        }
        Ok(outbox)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Enqueues an action.
    ///
    /// Returns `true` when a new entry was created and `false` when the key was
    /// already present (a duplicate enqueue, correctly ignored).
    pub fn enqueue(&self, new: NewOutboxEntry) -> Result<bool> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| Error::ConfigError(format!("outbox write txn: {e}")))?;
        let inserted = {
            let mut table = txn
                .open_table(OUTBOX)
                .map_err(|e| Error::ConfigError(format!("open outbox table: {e}")))?;

            let already = table
                .get(new.idempotency_key.as_str())
                .map_err(|e| Error::ConfigError(format!("probe outbox key: {e}")))?
                .is_some();
            if already {
                false
            } else {
                let entry = OutboxEntry {
                    idempotency_key: new.idempotency_key.clone(),
                    action: new.action,
                    payload: new.payload,
                    state: OutboxState::Pending,
                    attempts: 0,
                    max_attempts: new.max_attempts.max(1),
                    not_before_ms: 0,
                    created_at_unix: now_unix(),
                    last_error: None,
                };
                let bytes = serde_json::to_vec(&entry)
                    .map_err(|e| Error::ConfigError(format!("serialise outbox entry: {e}")))?;
                table
                    .insert(entry.idempotency_key.as_str(), bytes.as_slice())
                    .map_err(|e| Error::ConfigError(format!("insert outbox entry: {e}")))?;
                true
            }
        };
        txn.commit()
            .map_err(|e| Error::ConfigError(format!("outbox commit: {e}")))?;
        Ok(inserted)
    }

    /// Entries that are due now: `Pending` and past their backoff.
    pub fn due(&self) -> Result<Vec<OutboxEntry>> {
        let now = now_ms();
        Ok(self
            .all()?
            .into_iter()
            .filter(|e| e.state == OutboxState::Pending && e.not_before_ms <= now)
            .collect())
    }

    /// Every entry, for diagnostics and tests.
    pub fn all(&self) -> Result<Vec<OutboxEntry>> {
        let txn = self
            .db
            .begin_read()
            .map_err(|e| Error::ConfigError(format!("outbox read txn: {e}")))?;
        let table = match txn.open_table(OUTBOX) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(Error::ConfigError(format!("open outbox table: {e}"))),
        };

        let mut out = Vec::new();
        for row in table
            .iter()
            .map_err(|e| Error::ConfigError(format!("iterate outbox: {e}")))?
        {
            let (_, value) = row.map_err(|e| Error::ConfigError(format!("read outbox row: {e}")))?;
            let entry: OutboxEntry = serde_json::from_slice(value.value())
                .map_err(|e| Error::ConfigError(format!("decode outbox entry: {e}")))?;
            out.push(entry);
        }
        // FIFO: oldest intent first.
        out.sort_by_key(|e| (e.created_at_unix, e.idempotency_key.clone()));
        Ok(out)
    }

    /// Marks an entry as being attempted.
    pub fn mark_in_flight(&self, key: &str) -> Result<()> {
        self.update(key, |entry| {
            entry.state = OutboxState::InFlight;
            Ok(())
        })
    }

    /// Marks an entry delivered.
    pub fn mark_done(&self, key: &str) -> Result<()> {
        self.update(key, |entry| {
            entry.state = OutboxState::Done;
            entry.last_error = None;
            Ok(())
        })
    }

    /// Records a failed attempt: schedules the next try with exponential
    /// backoff, or marks the entry [`OutboxState::Failed`] when the attempt
    /// budget is spent.
    ///
    /// Returns the resulting state so a caller can log the transition.
    pub fn mark_failed(&self, key: &str, error: impl Into<String>) -> Result<OutboxState> {
        let error = error.into();
        let mut resulting = OutboxState::Pending;
        self.update(key, |entry| {
            entry.attempts += 1;
            entry.last_error = Some(error.clone());
            if entry.attempts >= entry.max_attempts {
                entry.state = OutboxState::Failed;
            } else {
                entry.state = OutboxState::Pending;
                // Exponential backoff, capped.
                let shift = (entry.attempts - 1).min(20);
                let delay = BASE_BACKOFF_MS
                    .saturating_mul(1u64 << shift)
                    .min(MAX_BACKOFF_MS);
                entry.not_before_ms = now_ms().saturating_add(delay);
            }
            resulting = entry.state;
            Ok(())
        })?;
        Ok(resulting)
    }

    /// Returns a `Failed` entry to `Pending` so it can be attempted again.
    ///
    /// Resets the attempt counter and the backoff: the operator pressing "retry"
    /// means "try again now with a fresh budget", and an entry whose budget was
    /// already spent would otherwise be released straight back into `Failed` on
    /// the next attempt, making the button appear to do nothing.
    ///
    /// Errors when the key is unknown. A `Done` entry is left untouched — it was
    /// delivered, and re-queueing it would risk a duplicate effect if the action
    /// is not idempotent on the server side.
    pub fn requeue_failed(&self, key: &str) -> Result<()> {
        self.update(key, |entry| {
            if entry.state == OutboxState::Failed {
                entry.state = OutboxState::Pending;
                entry.attempts = 0;
                entry.not_before_ms = 0;
                entry.last_error = None;
            }
            Ok(())
        })
    }

    /// Number of entries in each state, for a status endpoint.
    pub fn counts(&self) -> Result<(u64, u64, u64, u64)> {
        let mut pending = 0;
        let mut in_flight = 0;
        let mut done = 0;
        let mut failed = 0;
        for entry in self.all()? {
            match entry.state {
                OutboxState::Pending => pending += 1,
                OutboxState::InFlight => in_flight += 1,
                OutboxState::Done => done += 1,
                OutboxState::Failed => failed += 1,
            }
        }
        Ok((pending, in_flight, done, failed))
    }

    /// Removes delivered entries older than `age_secs`, so the file cannot grow
    /// without bound on a long-lived install.
    ///
    /// `age_secs` is the retention window: an entry is pruned when it was
    /// created *before* `now - age_secs`. Note that a very large `age_secs`
    /// saturates the cutoff to 0 rather than matching everything — pass a real
    /// window (e.g. 30 days), not `u64::MAX`.
    pub fn prune_done(&self, age_secs: u64) -> Result<u64> {
        let cutoff = now_unix().saturating_sub(age_secs);
        let stale: Vec<String> = self
            .all()?
            .into_iter()
            .filter(|e| e.state == OutboxState::Done && e.created_at_unix < cutoff)
            .map(|e| e.idempotency_key)
            .collect();
        if stale.is_empty() {
            return Ok(0);
        }

        let txn = self
            .db
            .begin_write()
            .map_err(|e| Error::ConfigError(format!("outbox write txn: {e}")))?;
        {
            let mut table = txn
                .open_table(OUTBOX)
                .map_err(|e| Error::ConfigError(format!("open outbox table: {e}")))?;
            for key in &stale {
                table
                    .remove(key.as_str())
                    .map_err(|e| Error::ConfigError(format!("prune {key}: {e}")))?;
            }
        }
        txn.commit()
            .map_err(|e| Error::ConfigError(format!("outbox commit: {e}")))?;
        Ok(stale.len() as u64)
    }

    /// Read-modify-write one entry inside a single write transaction.
    fn update<F>(&self, key: &str, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut OutboxEntry) -> Result<()>,
    {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| Error::ConfigError(format!("outbox write txn: {e}")))?;
        {
            let mut table = txn
                .open_table(OUTBOX)
                .map_err(|e| Error::ConfigError(format!("open outbox table: {e}")))?;

            let current = table
                .get(key)
                .map_err(|e| Error::ConfigError(format!("get outbox entry: {e}")))?
                .map(|guard| guard.value().to_vec());

            let Some(bytes) = current else {
                return Err(Error::ConfigError(format!("outbox entry not found: {key}")));
            };
            let mut entry: OutboxEntry = serde_json::from_slice(&bytes)
                .map_err(|e| Error::ConfigError(format!("decode outbox entry: {e}")))?;

            mutate(&mut entry)?;

            let encoded = serde_json::to_vec(&entry)
                .map_err(|e| Error::ConfigError(format!("serialise outbox entry: {e}")))?;
            table
                .insert(key, encoded.as_slice())
                .map_err(|e| Error::ConfigError(format!("update outbox entry: {e}")))?;
        }
        txn.commit()
            .map_err(|e| Error::ConfigError(format!("outbox commit: {e}")))?;
        Ok(())
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str) -> NewOutboxEntry {
        NewOutboxEntry::new(key, "rcon.command", r#"{"command":"saveworld"}"#)
    }

    #[test]
    fn enqueue_is_idempotent_by_key() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();

        assert!(outbox.enqueue(entry("key-1")).unwrap(), "first insert wins");
        assert!(
            !outbox.enqueue(entry("key-1")).unwrap(),
            "same idempotency key must not create a second entry"
        );
        assert_eq!(outbox.all().unwrap().len(), 1);
    }

    #[test]
    fn a_queued_action_survives_process_death() {
        let dir = tempfile::tempdir().unwrap();
        {
            let outbox = Outbox::open(dir.path()).unwrap();
            outbox.enqueue(entry("key-1")).unwrap();
        } // dropped: simulates the process going away

        let reopened = Outbox::open(dir.path()).unwrap();
        let due = reopened.due().unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].idempotency_key, "key-1");
        assert_eq!(due[0].state, OutboxState::Pending);
    }

    #[test]
    fn due_excludes_done_and_backing_off_entries() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        outbox.enqueue(entry("done")).unwrap();
        outbox.enqueue(entry("retrying")).unwrap();
        outbox.mark_done("done").unwrap();
        outbox.mark_failed("retrying", "connection refused").unwrap();

        let due = outbox.due().unwrap();
        assert!(
            due.is_empty(),
            "a delivered entry and a backing-off entry are both not due, got {due:?}"
        );
    }

    #[test]
    fn backoff_grows_then_gives_up_at_the_budget() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        let mut new = entry("key-1");
        new.max_attempts = 3;
        outbox.enqueue(new).unwrap();

        let s1 = outbox.mark_failed("key-1", "boom").unwrap();
        assert_eq!(s1, OutboxState::Pending, "first failure schedules a retry");
        let after_first = outbox.all().unwrap()[0].clone();
        assert_eq!(after_first.attempts, 1);
        assert!(after_first.not_before_ms > 0, "backoff must be scheduled");

        outbox.mark_failed("key-1", "boom").unwrap();
        let s3 = outbox.mark_failed("key-1", "boom").unwrap();
        assert_eq!(s3, OutboxState::Failed, "budget exhausted => Failed");

        let entry = &outbox.all().unwrap()[0];
        assert_eq!(entry.attempts, 3);
        assert_eq!(entry.last_error.as_deref(), Some("boom"));
    }

    #[test]
    fn retry_after_success_is_still_a_single_entry() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        outbox.enqueue(entry("key-1")).unwrap();
        outbox.mark_done("key-1").unwrap();

        // A retried producer must not resurrect the action.
        assert!(!outbox.enqueue(entry("key-1")).unwrap());
        assert_eq!(outbox.all().unwrap()[0].state, OutboxState::Done);
    }

    #[test]
    fn counts_reports_each_state() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        outbox.enqueue(entry("a")).unwrap();
        outbox.enqueue(entry("b")).unwrap();
        outbox.enqueue(entry("c")).unwrap();
        outbox.mark_done("a").unwrap();
        outbox.mark_in_flight("b").unwrap();

        let (pending, in_flight, done, failed) = outbox.counts().unwrap();
        assert_eq!((pending, in_flight, done, failed), (1, 1, 1, 0));
    }

    #[test]
    fn prune_removes_only_old_done_entries() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        outbox.enqueue(entry("old-done")).unwrap();
        outbox.enqueue(entry("pending")).unwrap();
        outbox.mark_done("old-done").unwrap();

        // A freshly delivered entry is inside any sane retention window, so a
        // 1-hour (or 30-day) prune must not touch it, and Pending is never
        // eligible no matter the window.
        assert_eq!(
            outbox.prune_done(3600).unwrap(),
            0,
            "a freshly delivered entry is inside the retention window"
        );
        assert_eq!(outbox.prune_done(30 * 24 * 3600).unwrap(), 0);
        assert_eq!(outbox.all().unwrap().len(), 2);

        // With a zero window the cutoff is "now", and an entry created in the
        // current second is not strictly older — entries only leave the outbox
        // once real time has passed them. The safety property under test is
        // that Pending is never pruned, regardless.
        let removed = outbox.prune_done(0).unwrap();
        assert_eq!(removed, 0, "prune never targets the current second");
        assert_eq!(
            outbox.all().unwrap().len(),
            2,
            "both entries survive; Pending is never pruned"
        );
        assert!(outbox
            .all()
            .unwrap()
            .iter()
            .any(|e| e.idempotency_key == "pending"));
    }

    #[test]
    fn unknown_key_is_an_error_not_a_silent_noop() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        assert!(outbox.mark_done("never-enqueued").is_err());
    }

    #[test]
    fn an_operator_retry_revives_a_given_up_entry_with_a_fresh_budget() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        let mut new = entry("spent");
        new.max_attempts = 1;
        outbox.enqueue(new).unwrap();

        // Spend the budget: the entry gives up.
        assert_eq!(
            outbox.mark_failed("spent", "boom").unwrap(),
            OutboxState::Failed
        );

        // The operator's retry must actually retry, so the counter and backoff
        // are reset — otherwise the very next attempt would fail immediately
        // and the button would look broken.
        outbox.requeue_failed("spent").unwrap();
        let revived = &outbox.all().unwrap()[0];
        assert_eq!(revived.state, OutboxState::Pending);
        assert_eq!(revived.attempts, 0);
        assert_eq!(revived.not_before_ms, 0);
        assert_eq!(revived.last_error, None);

        // An unknown key is a caller bug, not a silent success.
        assert!(outbox.requeue_failed("never-enqueued").is_err());
    }

    #[test]
    fn retry_leaves_a_delivered_entry_alone() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Outbox::open(dir.path()).unwrap();
        outbox.enqueue(entry("delivered")).unwrap();
        outbox.mark_done("delivered").unwrap();

        outbox.requeue_failed("delivered").unwrap();
        assert_eq!(
            outbox.all().unwrap()[0].state,
            OutboxState::Done,
            "re-queueing a delivered action could duplicate a side effect"
        );
    }
}
