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

use crate::commands::get_config_dir;

#[tauri::command]
pub async fn load_config(config_path: String) -> std::result::Result<ServerConfig, String> {
    ConfigLoader::load_or_default(&PathBuf::from(config_path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_config_or_default(app: tauri::AppHandle) -> std::result::Result<ServerConfig, String> {
    let config_dir = get_config_dir(&app)?;
    tokio::fs::create_dir_all(&config_dir).await.map_err(|e| e.to_string())?;
    let toml_path = config_dir.join("server-config.toml");
    log::info!("Loading config from {:?}", toml_path);
    ConfigLoader::load_or_default(&toml_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_config(config: ServerConfig) -> std::result::Result<serde_json::Value, String> {
    let result = CompositeValidator::default()
        .validate(&config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({ "valid": result.valid, "errors": result.errors }))
}

#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, config: ServerConfig) -> std::result::Result<(), String> {
    use tokio::fs;

    let config_dir = get_config_dir(&app)?;
    fs::create_dir_all(&config_dir).await
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

    let toml_path = config_dir.join("server-config.toml");
    log::info!("Saving config to {:?}", toml_path);
    ConfigPersister::save_toml(&config, &toml_path).await.map_err(|e| e.to_string())?;

    // Backup existing INI files before overwriting
    let backup_dir = PathBuf::from(&config.paths.backup_dir);
    fs::create_dir_all(&backup_dir).await.ok();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let game_ini = PathBuf::from(&config.paths.game_ini_path);
    let gus_ini  = PathBuf::from(&config.paths.gamesettings_ini_path);
    if game_ini.exists() {
        fs::copy(&game_ini, backup_dir.join(format!("Game.ini.{}.bak", ts))).await.ok();
    }
    if gus_ini.exists() {
        fs::copy(&gus_ini, backup_dir.join(format!("GameUserSettings.ini.{}.bak", ts))).await.ok();
    }

    if let Err(e) = ConfigPersister::generate_game_ini(&config, &game_ini).await {
        log::warn!("Could not write Game.ini (path may not exist yet): {}", e);
    }
    if let Err(e) = ConfigPersister::generate_gamesettings_ini(&config, &gus_ini).await {
        log::warn!("Could not write GameUserSettings.ini (path may not exist yet): {}", e);
    }
    Ok(())
}

#[tauri::command]
pub fn get_default_config() -> std::result::Result<ServerConfig, String> {
    Ok(ServerConfig::default())
}

#[tauri::command]
pub fn get_config_schema() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({
        "identification": { "session_name": { "type": "string" }, "admin_password": { "type": "string" } },
        "network":        { "port": { "type": "number", "min": 1024, "max": 65535 } },
        "gameplay":       { "max_players": { "type": "number" } },
        "multipliers":    { "xp_multiplier": { "type": "number" } }
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// CurseForge API key management
// ─────────────────────────────────────────────────────────────────────────────
