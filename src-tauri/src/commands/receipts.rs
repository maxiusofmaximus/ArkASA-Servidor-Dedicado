#![allow(unused_imports)]

use crate::{auth, backup, config, integrations, plugins, receipts, stub};
use crate::ark;
use crate::config::{ConfigLoader, ConfigPersister, ServerConfig, CompositeValidator};
use crate::ark::{build_launch_args, RconClient};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, LazyLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::Command;
use tauri::{Manager, Emitter};
use std::sync::OnceLock;

#[tauri::command]
pub fn receipts_probe() -> Result<String, String> {
    let g = shared_ledger().read();
    let Some(l) = g.as_ref() else {
        return Err("ledger not initialised".into());
    };
    Ok(l.host_id().to_string())
}

#[tauri::command]
pub fn receipts_today_path() -> Result<String, String> {
    let g = shared_ledger().read();
    let Some(l) = g.as_ref() else {
        return Err("ledger not initialised".into());
    };
    Ok(l.today_path().to_string_lossy().to_string())
}

#[tauri::command]
pub fn receipts_tail(n: u32) -> Result<Vec<receipts::Receipt>, String> {
    let g = shared_ledger().read();
    let Some(l) = g.as_ref() else {
        return Err("ledger not initialised".into());
    };
    l.tail(n as usize)
}

/// Manually trigger the GDPR Article 5 janitor. Returns the number of
/// `rawText` rows that were redacted across all `YYYY-MM-DD.jsonl` files.
#[tauri::command]
pub fn receipts_sweep_expired() -> Result<u64, String> {
    let g = shared_ledger().read();
    let Some(l) = g.as_ref() else {
        return Err("ledger not initialised".into());
    };
    l.sweep_expired()
}

// ─── Ledger lifecycle ─────────────────────────────────────────────────────────

use std::path::Path;
// ─── Ledger lifecycle ─────────────────────────────────────────────────────────

static LEDGER: OnceLock<parking_lot::RwLock<Option<Arc<receipts::ReceiptLedger>>>> = OnceLock::new();

pub fn shared_ledger() -> &'static parking_lot::RwLock<Option<Arc<receipts::ReceiptLedger>>> {
    LEDGER.get_or_init(|| parking_lot::RwLock::new(None))
}

pub fn install_ledger(app_dir: &Path, host_id: &str) {
    let mut w = shared_ledger().write();
    if w.is_none() {
        *w = Some(Arc::new(receipts::ReceiptLedger::new(
            app_dir.join("receipts"),
            host_id.to_string(),
        )));
        log::info!("receipts ledger initialised at {}/receipts", app_dir.display());
    }
}

/// Extract config.toml from a backup .zip and parse it into a ServerConfig.
#[tauri::command]
pub fn parse_config_from_zip(zip_data: Vec<u8>) -> Result<config::ServerConfig, String> {
    use std::io::Cursor;
    let cursor = Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open zip: {}", e))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let name = file.name().to_lowercase();
        if name == "config.toml" || name.ends_with("/config.toml") {
            let mut contents = String::new();
            use std::io::Read;
            file.read_to_string(&mut contents)
                .map_err(|e| format!("Failed to read config.toml from zip: {}", e))?;
            let mut cfg: config::ServerConfig = toml::from_str(&contents)
                .map_err(|e| format!("Failed to parse config.toml: {}", e))?;
            cfg.network.migrate_legacy_connections();
            return Ok(cfg);
        }
    }
    Err("No config.toml found inside the zip file".to_string())
}
