//! Platform-agnostic core of the ARK ASA Configuration Manager.
//!
//! This crate holds everything that has no desktop/Tauri/OS-specific
//! dependency: the configuration schema, TOML/INI load + generation,
//! multi-server profile registry, declarative custom validators, config-drift
//! detection and presets.
//!
//! It is the foundation for sharing logic across surfaces — the desktop app
//! (`src-tauri`, which re-exports this as `crate::config` / `crate::error`),
//! the `ark-config` CLI, and the future mobile companion (via UniFFI).
//!
//! Keep this crate free of `tauri`, `axum`, `reqwest`, OS keyrings, and any
//! `std::process` spawns so it can be compiled for Android/iOS.
//!
//! ## `ARK_ASA_HOME`
//! Tests that need per-`ConfigStore` isolation set `ARK_ASA_HOME`; helpers in
//! [`profiles`] and [`presets`] are constructed with an explicit directory, so
//! the process-level env is only consulted by callers that opt in.

pub mod config;
pub mod error;
pub mod offline;
pub mod rcon;

// Convenience re-exports so consumers can `use ark_core::{ServerConfig, ...}`.
pub use config::{
    evaluate_rules, AppConfig, CompositeValidator, ConfigLoader, ConfigPersister, ConfigValidator,
    CustomRule, CustomValidatorStore, DriftEntry, PresetInfo, PresetStore, ProfileStore,
    ServerConfig, ServerProfile, ValidationResult,
};
pub use config::validator::ValidationError;
pub use offline::{NewOutboxEntry, OfflineStore, Outbox, OutboxEntry, OutboxState};
pub use offline::{DispatchOutcome, Dispatcher, DrainReport, OutboxSender};
pub use rcon::{decode_packet, encode_packet, Packet as RconPacket};
pub use error::{Error, Result};
