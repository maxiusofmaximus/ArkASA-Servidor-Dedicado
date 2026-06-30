use crate::config::schema::ServerConfig;
use crate::error::{Error, Result};
use std::path::Path;
use tokio::fs;

pub struct ConfigLoader;

impl ConfigLoader {
    pub async fn load_from_toml(path: &Path) -> Result<ServerConfig> {
        if !path.exists() {
            return Err(Error::ConfigNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| Error::IoError(e))?;

        let mut config: ServerConfig = toml::from_str(&content)
            .map_err(|e| Error::TomlError(e))?;

        config.network.migrate_legacy_connections();

        Ok(config)
    }

    pub async fn load_or_default(path: &Path) -> Result<ServerConfig> {
        match Self::load_from_toml(path).await {
            Ok(config) => Ok(config),
            Err(Error::ConfigNotFound { .. }) => {
                log::warn!("Config file not found at {:?}, using defaults", path);
                Ok(ServerConfig::default())
            }
            Err(e) => Err(e),
        }
    }

    pub fn load_from_str(content: &str) -> Result<ServerConfig> {
        toml::from_str(content)
            .map_err(|e| Error::TomlError(e))
    }

    pub async fn load_from_ini(path: &Path) -> Result<ServerConfig> {
        if !path.exists() {
            return Err(Error::ConfigNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| Error::IoError(e))?;

        Self::parse_ini_to_config(&content)
    }

    fn parse_ini_to_config(content: &str) -> Result<ServerConfig> {
        Ok(Self::merge_ini_content(&ServerConfig::default(), content))
    }

    /// Merge INI key=value lines into an existing config (unknown keys → custom_config).
    pub fn merge_ini_content(base: &ServerConfig, content: &str) -> ServerConfig {
        let mut config = base.clone();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with('[') {
                continue;
            }
            if let Some((key, value)) = Self::parse_ini_line(trimmed) {
                Self::apply_ini_value(&mut config, key, value);
            }
        }

        config
    }

    fn parse_ini_line(line: &str) -> Option<(&str, &str)> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            Some((parts[0].trim(), parts[1].trim()))
        } else {
            None
        }
    }

    fn apply_ini_value(config: &mut ServerConfig, key: &str, value: &str) {
        match key {
            // Identification
            "SessionName" => config.identification.session_name = value.to_string(),
            "ServerPassword" => config.identification.server_password = value.to_string(),
            "AdminPassword" => config.identification.admin_password = value.to_string(),
            "MessageOfTheDay" => config.identification.server_message_of_the_day = value.to_string(),

            // Network
            "Port" => config.network.port = value.parse().unwrap_or(7777),
            "QueryPort" => config.network.query_port = value.parse().unwrap_or(27015),
            "RCONPort" => config.network.rcon_port = value.parse().unwrap_or(27020),
            "ServerPlatform" => config.network.server_platform = value.to_string(),

            // Gameplay
            "ServerPVE" => config.gameplay.server_pve = value.parse().unwrap_or(true),
            "MaxPlayers" => config.gameplay.max_players = value.parse().unwrap_or(70),
            "DifficultyOffset" => config.gameplay.difficulty_offset = value.parse().unwrap_or(2.0),
            "DinoCountMultiplier" => {
                config.gameplay.dino_count_multiplier = value.parse().unwrap_or(2.0)
            }

            // Multipliers
            "XPMultiplier" => config.multipliers.xp_multiplier = value.parse().unwrap_or(3.0),
            "TamingSpeedMultiplier" => {
                config.multipliers.taming_speed_multiplier = value.parse().unwrap_or(15.0)
            }
            "HarvestAmountMultiplier" => {
                config.multipliers.harvest_amount_multiplier = value.parse().unwrap_or(8.0)
            }
            "BabyMatureSpeedMultiplier" => {
                config.multipliers.baby_mature_speed_multiplier = value.parse().unwrap_or(40.0)
            }
            "EggHatchSpeedMultiplier" => {
                config.multipliers.egg_hatch_speed_multiplier = value.parse().unwrap_or(20.0)
            }

            // Mods
            "ActiveMods" => {
                config.mods.active_mods = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && s != "0")
                    .collect()
            }

            // Advanced
            "AllowUnlimitedRespecs" => {
                config.advanced.allow_unlimited_respecs = value.parse().unwrap_or(true)
            }
            "AllowFlyerCarry" => {
                config.advanced.allow_flyer_carry = value.parse().unwrap_or(true)
            }
            "LimitGeneratorsNum" => {
                config.advanced.limit_generators_num = value.parse().unwrap_or(0)
            }
            "LimitGeneratorsRange" => {
                config.advanced.limit_generators_range = value.parse().unwrap_or(15000.0)
            }

            _ => {
                // Store in custom_config for extensibility
                config.advanced.custom_config.insert(key.to_string(), value.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini_line() {
        assert_eq!(
            ConfigLoader::parse_ini_line("Port=7777"),
            Some(("Port", "7777"))
        );
        assert_eq!(
            ConfigLoader::parse_ini_line("ServerName=My Server"),
            Some(("ServerName", "My Server"))
        );
        assert_eq!(ConfigLoader::parse_ini_line(";Comment"), None);
    }

    #[tokio::test]
    async fn test_parse_ini_to_config() {
        let ini_content = r#"
            Port=7777
            MaxPlayers=100
            ServerPVE=true
            ActiveMods=955131,1102729,1306435
        "#;

        let config = ConfigLoader::parse_ini_to_config(ini_content).unwrap();
        assert_eq!(config.network.port, 7777);
        assert_eq!(config.gameplay.max_players, 100);
        assert!(config.gameplay.server_pve);
        assert_eq!(config.mods.active_mods.len(), 3);
    }

    #[test]
    fn test_load_from_str() {
        let toml_content = r#"
            [identification]
            session_name = "Test Server"
            admin_password = "admin123"

            [network]
            port = 7777
        "#;

        let config = ConfigLoader::load_from_str(toml_content).unwrap();
        assert_eq!(config.identification.session_name, "Test Server");
        assert_eq!(config.network.port, 7777);
    }
}
