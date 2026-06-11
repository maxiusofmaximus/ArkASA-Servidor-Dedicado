pub mod config;
pub mod error;

use config::{ConfigLoader, ConfigPersister, ServerConfig, CompositeValidator};
use error::Result;
use serde_json::json;
use std::path::PathBuf;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_config,
            validate_config,
            save_config,
            get_default_config,
            get_config_schema,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
