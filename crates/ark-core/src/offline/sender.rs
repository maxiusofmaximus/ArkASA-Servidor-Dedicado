//! Drains the outbox: takes due entries, attempts each one, and records the
//! outcome.
//!
//! The sender owns the *policy* (which entries, in what order, what to do on
//! failure) and delegates the *transport* to a [`Dispatcher`]. That split is the
//! point: the desktop dials a real socket, the phone dials a tunnel, and tests
//! use a scripted dispatcher — all through the same state machine, which is
//! where the correctness lives.
//!
//! Safety comes from the queue, not from this code: because the idempotency key
//! is the outbox primary key, a crash between "attempted" and "recorded" can
//! only cause a **retry**, never a duplicate effect.

use std::sync::Arc;

use crate::error::Result;
use crate::offline::outbox::{Outbox, OutboxEntry, OutboxState};

/// Performs one action. Returns a human-readable outcome on success, or a
/// failure reason that gets stored on the entry.
///
/// Implementors do the I/O; they must not touch the outbox.
#[async_trait::async_trait]
pub trait Dispatcher: Send + Sync {
    async fn dispatch(&self, entry: &OutboxEntry) -> std::result::Result<String, String>;
}

/// What happened to one entry during a drain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// Delivered and marked done.
    Delivered { key: String, detail: String },
    /// Attempt failed; scheduled for retry (still `Pending`).
    Retrying { key: String, error: String },
    /// Attempt failed and the budget is spent; now `Failed`.
    GaveUp { key: String, error: String },
}

/// Result of draining the queue once.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DrainReport {
    pub outcomes: Vec<DispatchOutcome>,
}

impl DrainReport {
    pub fn delivered(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, DispatchOutcome::Delivered { .. }))
            .count()
    }

    pub fn retrying(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, DispatchOutcome::Retrying { .. }))
            .count()
    }

    pub fn gave_up(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, DispatchOutcome::GaveUp { .. }))
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.outcomes.is_empty()
    }
}

/// Sends queued actions through a [`Dispatcher`].
pub struct OutboxSender {
    outbox: Arc<Outbox>,
    dispatcher: Arc<dyn Dispatcher>,
    /// Cap on entries attempted per drain, so one pass cannot monopolise a
    /// mobile radio or a console loop. Leftovers stay queued for the next pass.
    batch_limit: usize,
}

impl OutboxSender {
    pub fn new(outbox: Arc<Outbox>, dispatcher: Arc<dyn Dispatcher>) -> Self {
        Self {
            outbox,
            dispatcher,
            batch_limit: 20,
        }
    }

    /// Overrides how many entries a single [`Self::drain_once`] may attempt.
    pub fn with_batch_limit(mut self, limit: usize) -> Self {
        self.batch_limit = limit.max(1);
        self
    }

    /// Attempts every entry that is due right now.
    ///
    /// Deliberately does not loop on backoff: an entry that fails is rescheduled
    /// for a future pass, so this returns promptly and the caller decides when
    /// to run again (a timer on desktop, a WorkManager job on Android).
    pub async fn drain_once(&self) -> Result<DrainReport> {
        let due = self.outbox.due()?;
        let mut report = DrainReport::default();

        for entry in due.into_iter().take(self.batch_limit) {
            let key = entry.idempotency_key.clone();

            // Claim it first: a crash from here on leaves the entry InFlight,
            // which the next drain must reconcile (see `reclaim_stale_in_flight`).
            self.outbox.mark_in_flight(&key)?;

            match self.dispatcher.dispatch(&entry).await {
                Ok(detail) => {
                    self.outbox.mark_done(&key)?;
                    report.outcomes.push(DispatchOutcome::Delivered { key, detail });
                }
                Err(error) => {
                    let state = self.outbox.mark_failed(&key, error.clone())?;
                    let outcome = if state == OutboxState::Failed {
                        DispatchOutcome::GaveUp { key, error }
                    } else {
                        DispatchOutcome::Retrying { key, error }
                    };
                    report.outcomes.push(outcome);
                }
            }
        }

        Ok(report)
    }

    /// Returns entries abandoned in `InFlight` by a crash back to `Pending`.
    ///
    /// This is the counterpart of claiming before dispatching: without it, a
    /// process that died mid-send would leave the entry stuck forever. Safe
    /// because re-dispatching is safe — that is what the idempotency key buys.
    pub fn reclaim_stale_in_flight(&self) -> Result<u64> {
        let mut reclaimed = 0;
        for entry in self.outbox.all()? {
            if entry.state == OutboxState::InFlight {
                self.outbox.mark_failed(
                    &entry.idempotency_key,
                    "reclaimed after an interrupted attempt",
                )?;
                reclaimed += 1;
            }
        }
        Ok(reclaimed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offline::outbox::NewOutboxEntry;
    use std::sync::Mutex;

    /// Records what it was asked to send and replies with a canned result.
    struct ScriptedDispatcher {
        sent: Mutex<Vec<String>>,
        fail_with: Option<String>,
    }

    impl ScriptedDispatcher {
        fn ok() -> Self {
            Self {
                sent: Mutex::new(Vec::new()),
                fail_with: None,
            }
        }
        fn failing(reason: &str) -> Self {
            Self {
                sent: Mutex::new(Vec::new()),
                fail_with: Some(reason.to_string()),
            }
        }
        fn sent(&self) -> Vec<String> {
            self.sent.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl Dispatcher for ScriptedDispatcher {
        async fn dispatch(&self, entry: &OutboxEntry) -> std::result::Result<String, String> {
            self.sent.lock().unwrap().push(entry.idempotency_key.clone());
            match &self.fail_with {
                Some(reason) => Err(reason.clone()),
                None => Ok("ok".to_string()),
            }
        }
    }

    fn queued(outbox: &Outbox, key: &str) {
        outbox
            .enqueue(NewOutboxEntry::new(key, "rcon.command", "{}"))
            .unwrap();
    }

    #[tokio::test]
    async fn delivers_due_entries_and_clears_the_queue() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Arc::new(Outbox::open(dir.path()).unwrap());
        queued(&outbox, "cmd-1");
        queued(&outbox, "cmd-2");

        let dispatcher = Arc::new(ScriptedDispatcher::ok());
        let sender = OutboxSender::new(outbox.clone(), dispatcher.clone());

        let report = sender.drain_once().await.unwrap();
        assert_eq!(report.delivered(), 2);
        assert_eq!(report.retrying(), 0);
        assert_eq!(dispatcher.sent().len(), 2, "both were actually sent");

        let (pending, _, done, failed) = outbox.counts().unwrap();
        assert_eq!((pending, done, failed), (0, 2, 0));
        assert!(sender.drain_once().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_failure_schedules_a_retry_and_keeps_the_entry() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Arc::new(Outbox::open(dir.path()).unwrap());
        queued(&outbox, "cmd-1");

        let sender = OutboxSender::new(
            outbox.clone(),
            Arc::new(ScriptedDispatcher::failing("connection refused")),
        );

        let report = sender.drain_once().await.unwrap();
        assert_eq!(report.retrying(), 1);
        assert_eq!(report.delivered(), 0);

        let entry = &outbox.all().unwrap()[0];
        assert_eq!(entry.state, OutboxState::Pending, "still queued for retry");
        assert_eq!(entry.last_error.as_deref(), Some("connection refused"));
        assert!(entry.not_before_ms > 0, "backoff was scheduled");
    }

    #[tokio::test]
    async fn gives_up_once_the_budget_is_spent() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Arc::new(Outbox::open(dir.path()).unwrap());
        let mut new = NewOutboxEntry::new("cmd-1", "rcon.command", "{}");
        new.max_attempts = 1;
        outbox.enqueue(new).unwrap();

        let sender = OutboxSender::new(
            outbox.clone(),
            Arc::new(ScriptedDispatcher::failing("permanent")),
        );

        let report = sender.drain_once().await.unwrap();
        assert_eq!(report.gave_up(), 1);
        assert_eq!(outbox.all().unwrap()[0].state, OutboxState::Failed);
    }

    #[tokio::test]
    async fn reclaim_returns_interrupted_attempts_to_the_queue() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Arc::new(Outbox::open(dir.path()).unwrap());
        queued(&outbox, "stuck");
        // Simulate a crash that happened after claiming, before recording.
        outbox.mark_in_flight("stuck").unwrap();
        assert_eq!(outbox.all().unwrap()[0].state, OutboxState::InFlight);
        assert!(outbox.due().unwrap().is_empty(), "in-flight is not due");

        let sender = OutboxSender::new(outbox.clone(), Arc::new(ScriptedDispatcher::ok()));
        assert_eq!(sender.reclaim_stale_in_flight().unwrap(), 1);

        let entry = &outbox.all().unwrap()[0];
        assert_eq!(entry.state, OutboxState::Pending);
        assert!(entry.last_error.as_deref().unwrap().contains("reclaimed"));
    }

    #[tokio::test]
    async fn batch_limit_leaves_the_rest_queued() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Arc::new(Outbox::open(dir.path()).unwrap());
        for i in 0..5 {
            queued(&outbox, &format!("cmd-{i}"));
        }

        let dispatcher = Arc::new(ScriptedDispatcher::ok());
        let sender =
            OutboxSender::new(outbox.clone(), dispatcher.clone()).with_batch_limit(2);

        let report = sender.drain_once().await.unwrap();
        assert_eq!(report.delivered(), 2);
        assert_eq!(dispatcher.sent().len(), 2);
        assert_eq!(outbox.counts().unwrap().0, 3, "three stay pending");
    }

    #[tokio::test]
    async fn an_empty_queue_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let outbox = Arc::new(Outbox::open(dir.path()).unwrap());
        let dispatcher = Arc::new(ScriptedDispatcher::ok());
        let sender = OutboxSender::new(outbox, dispatcher.clone());

        let report = sender.drain_once().await.unwrap();
        assert!(report.is_empty());
        assert!(dispatcher.sent().is_empty());
    }
}
