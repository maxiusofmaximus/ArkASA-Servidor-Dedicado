// Default configuration values and paths
use std::path::PathBuf;

pub struct AppConfig {
    pub config_dir: PathBuf,
    pub database_url: String,
    pub log_level: String,
    pub log_file: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let config_dir = AppConfig::get_config_dir();
        let database_url = AppConfig::get_database_url(&config_dir);
        let log_file = config_dir.join("app.log");

        Self {
            config_dir,
            database_url,
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            log_file,
        }
    }

    fn get_config_dir() -> PathBuf {
        // Windows: %APPDATA%/ARK ASA Config Manager
        // macOS: ~/Library/Application Support/ARK ASA Config Manager
        // Linux: ~/.config/ark-asa-config

        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(appdata).join("ARK ASA Config Manager")
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join("Library/Application Support/ARK ASA Config Manager")
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config/ark-asa-config")
        }
    }

    fn get_database_url(config_dir: &PathBuf) -> String {
        let db_path = config_dir.join("ark-config.db");
        format!("sqlite://{}?mode=rwc", db_path.display())
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(self.config_dir.join("backups"))?;
        std::fs::create_dir_all(self.config_dir.join("logs"))?;
        Ok(())
    }
}

// Server configuration defaults
pub mod server_defaults {
    pub const STEAMCMD_DIR: &str = "C:\\steamcmd";
    pub const SERVER_DIR: &str = "C:\\ASA\\server";
    pub const BACKUP_DIR: &str = "C:\\ASA\\backups";
    pub const GAME_INI_PATH: &str = "C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\Game.ini";
    pub const GAMESETTINGS_INI_PATH: &str = "C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\GameUserSettings.ini";

    // Game defaults
    pub const DEFAULT_PORT: u16 = 7777;
    pub const DEFAULT_QUERY_PORT: u16 = 27015;
    pub const DEFAULT_RCON_PORT: u16 = 27020;
    pub const DEFAULT_MAX_PLAYERS: u16 = 70;
    pub const DEFAULT_SESSION_NAME: &str = "ARK Server";

    // Multipliers
    pub const DEFAULT_XP_MULTIPLIER: f32 = 3.0;
    pub const DEFAULT_TAMING_SPEED: f32 = 15.0;
    pub const DEFAULT_HARVEST_AMOUNT: f32 = 8.0;
    pub const DEFAULT_BABY_MATURE_SPEED: f32 = 40.0;

    // Validation constraints
    pub const MIN_PORT: u16 = 1024;
    pub const MAX_PORT: u16 = 65535;
    pub const MIN_PLAYERS: u16 = 1;
    pub const MAX_PLAYERS: u16 = 1000;
    pub const MIN_PASSWORD_LENGTH: usize = 4;

    // Database
    pub const BACKUP_RETENTION_DAYS: i32 = 30;
    pub const LOG_RETENTION_DAYS: i32 = 7;
    pub const METRICS_RETENTION_DAYS: i32 = 1;

    // Monitoring
    pub const STATUS_CHECK_INTERVAL_SECS: u64 = 5;
    pub const METRICS_SAVE_INTERVAL_SECS: u64 = 30;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_creation() {
        let config = AppConfig::from_env();
        assert!(!config.database_url.is_empty());
        assert!(!config.log_level.is_empty());
    }

    #[test]
    fn test_config_dir_paths() {
        let config_dir = AppConfig::get_config_dir();
        assert!(config_dir.to_string_lossy().contains("ARK"));
    }
}
