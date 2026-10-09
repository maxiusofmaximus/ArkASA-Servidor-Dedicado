//! Durable offline layer: a local store that is the single source of truth,
//! plus a write-ahead outbox so no admin action is ever lost.
//!
//! Two concerns, deliberately separate:
//!
//! - [`store`] holds the *materialised state* the UI renders (profiles today,
//!   metrics and settings as they land). Reads always come from here, never
//!   from the network, which is what makes the app usable offline.
//! - [`outbox`] holds *pending writes*. An admin action (an RCON command, a
//!   config change) is appended here first and only then attempted, so it
//!   survives process death and a retried command cannot apply twice.
//!
//! The outbox key is the idempotency key itself. That is the whole trick: the
//! same key cannot be enqueued twice, and the consumer can retry freely without
//! deduplication logic of its own.
//!
//! Storage is [`redb`], a pure-Rust embedded key-value store. That choice is
//! deliberate: it has no C dependency, so the exact same code cross-compiles to
//! Android (and later iOS) with no extra toolchain, and it gives ACID writes and
//! crash safety without a SQL engine.

pub mod outbox;
pub mod sender;
pub mod store;

pub use outbox::{NewOutboxEntry, Outbox, OutboxEntry, OutboxState};
pub use sender::{DispatchOutcome, Dispatcher, DrainReport, OutboxSender};
pub use store::{OfflineStore, StoredProfile};
