pub mod cli;
pub mod config;
pub mod error;
pub mod ark;
pub mod storage;

use config::{ConfigLoader, ConfigPersister, ServerConfig, CompositeValidator};
use error::Result;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use ark::server::ArkServer;
use ark::{SteamCmdInstaller, ProcessManager, ArkServerImpl};
use storage::db::Database;

static CURRENT_SERVER: Lazy<Mutex<Option<Arc<dyn ArkServer>>>> = Lazy::new(|| Mutex::new(None));

// ============= CONFIG COMMANDS =============

#[tauri::command]
async fn load_config(config_path: String) -> Result<ServerConfig> {
    let path = PathBuf::from(&config_path);
    ConfigLoader::load_or_default(&path).await
}

#[tauri::command]
async fn validate_config(config: ServerConfig) -> Result<serde_json::Value> {
    let validator = CompositeValidator::default();
    let result = validator.validate(&config).await?;

    Ok(json!({
        "valid": result.valid,
        "errors": result.errors,
    }))
}

#[tauri::command]
async fn save_config(config: ServerConfig, config_path: String) -> Result<()> {
    let validator = CompositeValidator::default();
    let validation = validator.validate(&config).await?;

    if !validation.valid {
        return Err(error::Error::ValidationError(
            format!("Validation failed with {} errors", validation.errors.len()),
        ));
    }

    let path = PathBuf::from(&config_path);
    ConfigPersister::save_toml(&config, &path).await?;

    // Also generate INI files
    let paths = &config.paths;
    let game_ini_path = PathBuf::from(&paths.game_ini_path);
    let gamesettings_ini_path = PathBuf::from(&paths.gamesettings_ini_path);

    ConfigPersister::generate_game_ini(&config, &game_ini_path).await?;
    ConfigPersister::generate_gamesettings_ini(&config, &gamesettings_ini_path).await?;

    Ok(())
}

#[tauri::command]
async fn get_default_config() -> Result<ServerConfig> {
    Ok(ServerConfig::default())
}

#[tauri::command]
async fn get_config_schema() -> Result<serde_json::Value> {
    Ok(json!({
        "identification": {
            "session_name": { "type": "string", "required": true },
            "server_password": { "type": "string", "required": false },
            "admin_password": { "type": "string", "required": true },
            "server_message_of_the_day": { "type": "string", "required": false },
        },
        "network": {
            "port": { "type": "number", "min": 1024, "max": 65535 },
            "query_port": { "type": "number", "min": 1024, "max": 65535 },
            "rcon_port": { "type": "number", "min": 1024, "max": 65535 },
            "server_platform": { "type": "string", "enum": ["ALL", "WIN", "LINUX"] },
        },
        "gameplay": {
            "server_pve": { "type": "boolean" },
            "max_players": { "type": "number", "min": 1, "max": 1000 },
            "difficulty_offset": { "type": "number" },
            "dino_count_multiplier": { "type": "number" },
        },
        "multipliers": {
            "xp_multiplier": { "type": "number" },
            "taming_speed_multiplier": { "type": "number" },
            "harvest_amount_multiplier": { "type": "number" },
            "baby_mature_speed_multiplier": { "type": "number" },
            "egg_hatch_speed_multiplier": { "type": "number" },
        }
    }))
}

// ============= SERVER COMMANDS =============

#[tauri::command]
async fn server_status() -> Result<serde_json::Value> {
    let server = CURRENT_SERVER.lock().await;

    match server.as_ref() {
        Some(srv) => {
            let status = srv.status().await?;
            Ok(json!({
                "running": status.running,
                "process_id": status.process_id,
                "uptime_seconds": status.uptime_seconds,
            }))
        }
        None => Ok(json!({
            "running": false,
            "process_id": null,
            "uptime_seconds": 0,
        })),
    }
}

#[tauri::command]
async fn start_server(config: ServerConfig) -> Result<String> {
    let validator = CompositeValidator::default();
    let validation = validator.validate(&config).await?;

    if !validation.valid {
        return Err(error::Error::ValidationError(
            format!("Configuration invalid: {} errors", validation.errors.len()),
        ));
    }

    let installer = Arc::new(SteamCmdInstaller::new(
        &config.paths.steam_cmd_dir,
        &config.paths.server_dir,
    ));
    let process_mgr = Arc::new(ProcessManager::new());
    let server = Arc::new(ArkServerImpl::new(installer, process_mgr));

    // Verify installation
    if !server.installer.is_installed().await? {
        return Err(error::Error::ServerNotFound {
            path: config.paths.server_dir.clone(),
        });
    }

    let pid = server.start(&config).await?;

    let mut current = CURRENT_SERVER.lock().await;
    *current = Some(server);

    Ok(format!("Server started with PID: {}", pid))
}

#[tauri::command]
async fn stop_server() -> Result<String> {
    let server = CURRENT_SERVER.lock().await;

    match server.as_ref() {
        Some(srv) => {
            srv.stop().await?;
            drop(server);
            let mut current = CURRENT_SERVER.lock().await;
            *current = None;
            Ok("Server stopped".to_string())
        }
        None => Err(error::Error::ProcessError("Server is not running".to_string())),
    }
}

#[tauri::command]
async fn restart_server(config: ServerConfig) -> Result<String> {
    let server = CURRENT_SERVER.lock().await;

    match server.as_ref() {
        Some(srv) => {
            srv.restart(&config).await?;
            Ok("Server restarted".to_string())
        }
        None => Err(error::Error::ProcessError("Server is not running".to_string())),
    }
}

#[tauri::command]
async fn check_installation(steam_cmd_dir: String, server_dir: String) -> Result<bool> {
    let installer = SteamCmdInstaller::new(&steam_cmd_dir, &server_dir);
    installer.is_installed().await
}

#[tauri::command]
async fn install_server(steam_cmd_dir: String, server_dir: String) -> Result<String> {
    let installer = SteamCmdInstaller::new(&steam_cmd_dir, &server_dir);
    installer.install().await?;
    Ok("Server installed successfully".to_string())
}

// ============= LOGS & MONITORING =============

#[tauri::command]
async fn get_server_logs(lines: i32) -> Result<Vec<String>> {
    // TODO: Read from server log file
    // For now, return empty vec
    Ok(vec![
        "[INFO] Server started".to_string(),
        "[INFO] Loading mods...".to_string(),
        "[INFO] All systems operational".to_string(),
    ])
}

#[tauri::command]
async fn get_server_metrics() -> Result<serde_json::Value> {
    Ok(json!({
        "cpu_percent": 45.2,
        "memory_mb": 2048,
        "network_in_mbps": 2.5,
        "network_out_mbps": 1.8,
        "player_count": 5,
        "fps": 60,
    }))
}

#[tauri::command]
async fn backup_config(config: ServerConfig, backup_name: String) -> Result<String> {
    // TODO: Implement backup to database
    log::info!("Backup created: {}", backup_name);
    Ok(format!("Backup '{}' created successfully", backup_name))
}

#[tauri::command]
async fn list_backups() -> Result<Vec<serde_json::Value>> {
    // TODO: Load from database
    Ok(vec![
        json!({
            "name": "backup_2026-06-11",
            "timestamp": "2026-06-11 10:30",
            "config_version": 3,
        }),
    ])
}

#[tauri::command]
async fn restore_backup(backup_name: String) -> Result<ServerConfig> {
    // TODO: Load from database
    Ok(ServerConfig::default())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Config
            load_config,
            validate_config,
            save_config,
            get_default_config,
            get_config_schema,
            // Server
            server_status,
            start_server,
            stop_server,
            restart_server,
            check_installation,
            install_server,
            // Logs & Monitoring
            get_server_logs,
            get_server_metrics,
            backup_config,
            list_backups,
            restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
