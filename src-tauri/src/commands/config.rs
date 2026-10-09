#![allow(unused_imports)]

use crate::{auth, backup, config, integrations, plugins, receipts, stub};
use crate::ark;
use crate::config::{
    CompositeValidator, ConfigLoader, ConfigPersister, CustomRule, CustomValidatorStore,
    PresetInfo, PresetStore, ServerConfig,
};
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
    let store = crate::config::ProfileStore::new(config_dir);
    let toml_path = store.resolve_config_path(None)?;
    if let Some(parent) = toml_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    log::info!("Loading active profile config from {:?}", toml_path);
    ConfigLoader::load_or_default(&toml_path).await.map_err(|e| e.to_string())
}

/// Load the config of a specific profile (`id = None` → the active profile).
#[tauri::command]
pub async fn load_config_for_profile(
    app: tauri::AppHandle,
    id: Option<String>,
) -> std::result::Result<ServerConfig, String> {
    let config_dir = get_config_dir(&app)?;
    tokio::fs::create_dir_all(&config_dir).await.map_err(|e| e.to_string())?;
    let store = crate::config::ProfileStore::new(config_dir);
    let toml_path = store.resolve_config_path(id.as_deref())?;
    if let Some(parent) = toml_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    ConfigLoader::load_or_default(&toml_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_config(
    app: tauri::AppHandle,
    config: ServerConfig,
) -> std::result::Result<serde_json::Value, String> {
    let mut result = CompositeValidator::default()
        .validate(&config)
        .await
        .map_err(|e| e.to_string())?;

    // Operator-defined declarative validators ("validator plugins").
    if let Ok(dir) = get_config_dir(&app) {
        let store = CustomValidatorStore::new(dir);
        let extra = crate::config::evaluate_rules(&config, &store.list());
        if !extra.is_empty() {
            result.valid = false;
            result.errors.extend(extra);
        }
    }

    Ok(json!({ "valid": result.valid, "errors": result.errors }))
}

// ── Custom validators (declarative "validator plugins") ──────────────────────

#[tauri::command]
pub fn list_custom_validators(app: tauri::AppHandle) -> std::result::Result<Vec<CustomRule>, String> {
    Ok(CustomValidatorStore::new(get_config_dir(&app)?).list())
}

#[tauri::command]
pub fn upsert_custom_validator(app: tauri::AppHandle, rule: CustomRule) -> std::result::Result<(), String> {
    CustomValidatorStore::new(get_config_dir(&app)?).upsert(rule)
}

#[tauri::command]
pub fn remove_custom_validator(app: tauri::AppHandle, id: String) -> std::result::Result<(), String> {
    CustomValidatorStore::new(get_config_dir(&app)?).remove(&id)
}

#[tauri::command]
pub fn custom_validators_path(app: tauri::AppHandle) -> std::result::Result<String, String> {
    Ok(CustomValidatorStore::new(get_config_dir(&app)?)
        .path()
        .to_string_lossy()
        .to_string())
}

// ── Config drift detection ───────────────────────────────────────────────────

/// Load the active profile's config from the registry.
async fn active_profile_config(app: &tauri::AppHandle) -> std::result::Result<ServerConfig, String> {
    let dir = get_config_dir(app)?;
    let path = crate::config::ProfileStore::new(dir).resolve_config_path(None)?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    ConfigLoader::load_or_default(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Report every managed INI key whose on-disk value differs from (or is
/// missing versus) what the app would write — i.e. drift caused by ARK itself
/// or another tool rewriting `Game.ini` / `GameUserSettings.ini`.
#[tauri::command]
pub async fn detect_config_drift(
    app: tauri::AppHandle,
) -> std::result::Result<Vec<crate::config::DriftEntry>, String> {
    let config = active_profile_config(&app).await?;
    Ok(crate::config::drift::detect(&config))
}

// ── Config presets (shareable templates) ─────────────────────────────────────

#[tauri::command]
pub fn list_config_presets(app: tauri::AppHandle) -> std::result::Result<Vec<PresetInfo>, String> {
    Ok(PresetStore::new(get_config_dir(&app)?).list())
}

#[tauri::command]
pub fn save_config_preset(
    app: tauri::AppHandle,
    name: String,
    config: ServerConfig,
) -> std::result::Result<PresetInfo, String> {
    PresetStore::new(get_config_dir(&app)?).save(&name, &config)
}

#[tauri::command]
pub fn load_config_preset(
    app: tauri::AppHandle,
    name: String,
) -> std::result::Result<ServerConfig, String> {
    PresetStore::new(get_config_dir(&app)?).load(&name)
}

#[tauri::command]
pub fn delete_config_preset(app: tauri::AppHandle, name: String) -> std::result::Result<(), String> {
    PresetStore::new(get_config_dir(&app)?).delete(&name)
}

#[tauri::command]
pub fn config_presets_dir(app: tauri::AppHandle) -> std::result::Result<String, String> {
    Ok(PresetStore::new(get_config_dir(&app)?)
        .dir()
        .to_string_lossy()
        .to_string())
}

/// Persist `config` to `toml_path` and regenerate the INIs at the paths the
/// config itself declares, backing up the previous INI files first.
async fn persist_config(config: &ServerConfig, toml_path: &PathBuf) -> Result<(), String> {
    use tokio::fs;

    if let Some(parent) = toml_path.parent() {
        fs::create_dir_all(parent).await
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    log::info!("Saving config to {:?}", toml_path);
    ConfigPersister::save_toml(config, toml_path).await.map_err(|e| e.to_string())?;

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

    if let Err(e) = ConfigPersister::generate_game_ini(config, &game_ini).await {
        log::warn!("Could not write Game.ini (path may not exist yet): {}", e);
    }
    if let Err(e) = ConfigPersister::generate_gamesettings_ini(config, &gus_ini).await {
        log::warn!("Could not write GameUserSettings.ini (path may not exist yet): {}", e);
    }
    Ok(())
}

/// Save to the **active** profile.
#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, config: ServerConfig) -> std::result::Result<(), String> {
    let config_dir = get_config_dir(&app)?;
    let store = crate::config::ProfileStore::new(config_dir);
    let toml_path = store.resolve_config_path(None)?;
    persist_config(&config, &toml_path).await
}

/// Save to a **specific** profile.
#[tauri::command]
pub async fn save_config_for_profile(
    app: tauri::AppHandle,
    id: String,
    config: ServerConfig,
) -> std::result::Result<(), String> {
    let config_dir = get_config_dir(&app)?;
    let store = crate::config::ProfileStore::new(config_dir);
    let toml_path = store.resolve_config_path(Some(&id))?;
    persist_config(&config, &toml_path).await
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
