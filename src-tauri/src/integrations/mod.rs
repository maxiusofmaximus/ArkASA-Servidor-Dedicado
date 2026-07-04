//! Integrations module — the v2.1 multi-channel remote-admin boundary.
//!
//! Each adapter in this directory receives an inbound message from a channel
//! (web UI, REST, Telegram, Discord, etc.), maps it to a `RemoteCommand` from
//! `@ark-asa/shared-types`, optionally checks an allowlist, and then forwards
//! to the same internal `CommandRouter` used by the desktop app itself.
//!
//! Top-level responsibilities:
//!  - `http_api.rs`     Axum-based server on `127.0.0.1:8765` for Convex.
//!  - `convex_push.rs`  Periodic push of ARK server state to Convex cloud.
//!  - `command_router.rs` Internal dispatcher (start / stop / restart / etc.).
//!  - `http_commands.rs` Public HTTP REST handlers (for Hito 11).
//!  - channel-specific adapters: `telegram`, `discord`, `whatsapp`, `signal`,
//!    `wechat`, `ssh` — Hitos 6-10 wire each in turn.
//!
//! Each adapter is registered from `lib.rs::run()` so the desktop app only
//! starts them when its configured TOML enables them.
pub mod ai;
pub mod bridge;
pub mod command_router;
pub mod convex_push;
pub mod database;
pub mod discord;
pub mod hosting;
pub mod http_api;
pub mod http_commands;
pub mod identity;
pub mod local_provision;
pub mod receipt_emit;
pub mod slack;
pub mod tailscale;
pub mod telegram;

pub use command_router::{CommandRouter, RouterError, RouterOutcome, RemoteCommandContext, Channel, Role};
pub use identity::{Identity, IdentityResolution, ChannelBinding, Platform, RuntimeClass, RejectionReason};
pub use telegram::{TelegramBot, TelegramConfig, spawn_looper};
pub use bridge::dispatch;
pub use receipt_emit::{Emitter, DeliveryStatus, ReceiptContext, try_global};
