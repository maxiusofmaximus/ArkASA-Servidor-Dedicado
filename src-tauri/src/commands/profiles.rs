//! Tauri commands for the multi-server profile registry (`config::profiles`).

use crate::commands::get_config_dir;
use crate::config::profiles::{ProfileStore, ServerProfile};

fn store(app: &tauri::AppHandle) -> Result<ProfileStore, String> {
    Ok(ProfileStore::new(get_config_dir(app)?))
}

/// Keep the multi-channel router pointed at the active profile. The router
/// resolves `ARK_ASA_CONFIG_PATH` lazily at dispatch time, so updating the
/// process env here is enough to make chat/loopback/Convex follow the switch.
fn pin_router_config(store: &ProfileStore) {
    if let Ok(path) = store.resolve_config_path(None) {
        std::env::set_var("ARK_ASA_CONFIG_PATH", &path);
    }
}

#[tauri::command]
pub fn list_server_profiles(app: tauri::AppHandle) -> Result<Vec<ServerProfile>, String> {
    Ok(store(&app)?.list())
}

#[tauri::command]
pub fn active_server_profile(app: tauri::AppHandle) -> Result<String, String> {
    Ok(store(&app)?.active_id())
}

#[tauri::command]
pub fn set_active_server_profile(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let s = store(&app)?;
    s.set_active(&id)?;
    pin_router_config(&s);
    Ok(())
}

#[tauri::command]
pub fn create_server_profile(app: tauri::AppHandle, name: String) -> Result<ServerProfile, String> {
    store(&app)?.create(&name)
}

#[tauri::command]
pub fn rename_server_profile(app: tauri::AppHandle, id: String, name: String) -> Result<(), String> {
    store(&app)?.rename(&id, &name)
}

#[tauri::command]
pub fn delete_server_profile(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let s = store(&app)?;
    s.delete(&id)?;
    pin_router_config(&s);
    Ok(())
}

/// Absolute `server-config.toml` path for a profile (active when `id` is None).
#[tauri::command]
pub fn server_profile_config_path(app: tauri::AppHandle, id: Option<String>) -> Result<String, String> {
    let s = store(&app)?;
    Ok(s.resolve_config_path(id.as_deref())?.to_string_lossy().to_string())
}
