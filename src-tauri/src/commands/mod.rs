pub mod app;
pub mod config;
pub mod integrations;
pub mod mods;
pub mod network;
pub mod receipts;
pub mod server;
pub mod utilities;

pub use app::*;
pub use config::*;
pub use integrations::*;
pub use mods::*;
pub use network::*;
pub use receipts::*;
pub use server::*;
pub use utilities::*;

use std::path::PathBuf;
use tauri::Manager;

pub(crate) fn get_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = if cfg!(target_os = "windows") {
        match std::env::var("APPDATA") {
            Ok(p) if !p.is_empty() => PathBuf::from(p),
            _ => app.path()
                    .data_dir()
                    .map_err(|e| format!("Failed to resolve config dir: {}", e))?,
        }
    } else {
        app.path()
            .data_dir()
            .map_err(|e| format!("Failed to resolve config dir: {}", e))?
    };
    Ok(base.join("ARK ASA Config Manager"))
}
