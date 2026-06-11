pub mod config;
pub mod error;

use config::{ConfigLoader, ConfigPersister, ServerConfig, CompositeValidator};
use serde_json::json;
use std::path::PathBuf;

#[tauri::command]
async fn load_config(config_path: String) -> std::result::Result<ServerConfig, String> {
    let path = PathBuf::from(&config_path);
    ConfigLoader::load_or_default(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_config(config: ServerConfig) -> std::result::Result<serde_json::Value, String> {
    let validator = CompositeValidator::default();
    let result = validator.validate(&config).await.map_err(|e| e.to_string())?;
    Ok(json!({ "valid": result.valid, "errors": result.errors }))
}

#[tauri::command]
async fn save_config(config: ServerConfig, config_path: String) -> std::result::Result<(), String> {
    let validator = CompositeValidator::default();
    validator.validate(&config).await.map_err(|e| e.to_string())?;

    let path = PathBuf::from(&config_path);
    ConfigPersister::save_toml(&config, &path).await.map_err(|e| e.to_string())?;

    let paths = &config.paths;
    ConfigPersister::generate_game_ini(&config, &PathBuf::from(&paths.game_ini_path)).await.map_err(|e| e.to_string())?;
    ConfigPersister::generate_gamesettings_ini(&config, &PathBuf::from(&paths.gamesettings_ini_path)).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_default_config() -> std::result::Result<ServerConfig, String> {
    Ok(ServerConfig::default())
}

#[tauri::command]
fn get_config_schema() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({
        "identification": { "session_name": { "type": "string" }, "admin_password": { "type": "string" } },
        "network": { "port": { "type": "number", "min": 1024, "max": 65535 } },
        "gameplay": { "max_players": { "type": "number" } },
        "multipliers": { "xp_multiplier": { "type": "number" } }
    }))
}

#[tauri::command]
fn server_status() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({"running": false, "process_id": null, "uptime_seconds": 0}))
}

#[tauri::command]
fn start_server(_config: ServerConfig) -> std::result::Result<String, String> {
    Ok("Server management coming soon".to_string())
}

#[tauri::command]
fn stop_server() -> std::result::Result<String, String> {
    Ok("Server management coming soon".to_string())
}

#[tauri::command]
fn restart_server(_config: ServerConfig) -> std::result::Result<String, String> {
    Ok("Server management coming soon".to_string())
}

#[tauri::command]
fn get_server_logs(_lines: i32) -> std::result::Result<Vec<String>, String> {
    Ok(vec!["[INFO] Server started".to_string()])
}

#[tauri::command]
fn get_server_metrics() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({"cpu": 0, "memory": 0, "fps": 0}))
}

#[tauri::command]
fn backup_config(_config: ServerConfig, name: String) -> std::result::Result<String, String> {
    Ok(format!("Backup '{}' created", name))
}

#[tauri::command]
fn list_backups() -> std::result::Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
fn restore_backup(_name: String) -> std::result::Result<ServerConfig, String> {
    Ok(ServerConfig::default())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_config, validate_config, save_config, get_default_config, get_config_schema,
            server_status, start_server, stop_server, restart_server,
            get_server_logs, get_server_metrics, backup_config, list_backups, restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
