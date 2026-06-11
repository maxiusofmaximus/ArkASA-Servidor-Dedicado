use crate::config::schema::ServerConfig;
use crate::error::{Error, Result};
use std::path::Path;
use tokio::fs;

pub struct ConfigPersister;

impl ConfigPersister {
    pub async fn save_toml(config: &ServerConfig, path: &Path) -> Result<()> {
        let serialized = toml::to_string_pretty(config)
            .map_err(|e| Error::ConfigError(format!("Failed to serialize config: {}", e)))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::IoError(e))?;
        }

        fs::write(path, serialized)
            .await
            .map_err(|e| Error::IoError(e))?;

        log::info!("Config saved to {:?}", path);
        Ok(())
    }

    pub async fn generate_game_ini(config: &ServerConfig, path: &Path) -> Result<()> {
        let content = Self::generate_game_ini_content(config);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::IoError(e))?;
        }

        fs::write(path, content)
            .await
            .map_err(|e| Error::IoError(e))?;

        log::info!("Game.ini generated at {:?}", path);
        Ok(())
    }

    pub async fn generate_gamesettings_ini(config: &ServerConfig, path: &Path) -> Result<()> {
        let content = Self::generate_gamesettings_ini_content(config);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::IoError(e))?;
        }

        fs::write(path, content)
            .await
            .map_err(|e| Error::IoError(e))?;

        log::info!("GameUserSettings.ini generated at {:?}", path);
        Ok(())
    }

    fn generate_game_ini_content(config: &ServerConfig) -> String {
        let mut content = String::new();
        content.push_str("[/script/shootergame.shootergamemode]\n");
        content.push_str(&format!("SessionName={}\n", config.identification.session_name));
        content.push_str(&format!("AdminPassword={}\n", config.identification.admin_password));
        content.push_str(&format!("ServerPassword={}\n", config.identification.server_password));
        content.push_str("\n[/script/shootergame.shootergamestate]\n");
        content.push_str(&format!("MaxPlayers={}\n", config.gameplay.max_players));
        content.push_str(&format!("DifficultyOffset={}\n", config.gameplay.difficulty_offset));
        content.push_str(&format!("DinoCountMultiplier={}\n", config.gameplay.dino_count_multiplier));
        content.push_str(&format!("bServerPVE={}\n", if config.gameplay.server_pve { "true" } else { "false" }));
        content.push_str("\n[/script/shootergame.gri]\n");
        content.push_str(&format!("MessageOfTheDay={}\n", config.identification.server_message_of_the_day));
        content
    }

    fn generate_gamesettings_ini_content(config: &ServerConfig) -> String {
        let mut content = String::new();

        content.push_str("[ServerSettings]\n");
        content.push_str(&format!("Port={}\n", config.network.port));
        content.push_str(&format!("QueryPort={}\n", config.network.query_port));
        content.push_str(&format!("RCONPort={}\n", config.network.rcon_port));
        content.push_str(&format!("ServerPlatform={}\n", config.network.server_platform));

        content.push_str("\n[/script/engine.gamesession]\n");
        content.push_str(&format!("MaxPlayers={}\n", config.gameplay.max_players));

        content.push_str("\n[/script/shootergame.shootercharactermovement]\n");
        content.push_str(&format!("CharacterSpeedMultiplier={}\n", 1.0));

        content.push_str("\n[/script/shootergame.shooterplayercontroller]\n");
        content.push_str(&format!("bAllowThirdPersonPlayer={}\n", if config.gameplay.allow_third_person_player { "true" } else { "false" }));

        content.push_str("\n[Multipliers]\n");
        content.push_str(&format!("XPMultiplier={}\n", config.multipliers.xp_multiplier));
        content.push_str(&format!("TamingSpeedMultiplier={}\n", config.multipliers.taming_speed_multiplier));
        content.push_str(&format!("HarvestAmountMultiplier={}\n", config.multipliers.harvest_amount_multiplier));
        content.push_str(&format!("HarvestHealthMultiplier={}\n", config.multipliers.harvest_health_multiplier));
        content.push_str(&format!("BabyMatureSpeedMultiplier={}\n", config.multipliers.baby_mature_speed_multiplier));
        content.push_str(&format!("BabyFoodConsumptionSpeedMultiplier={}\n", config.multipliers.baby_food_consumption_multiplier));
        content.push_str(&format!("BabyCuddleLossSpeedMultiplier={}\n", config.multipliers.baby_cuddle_loss_multiplier));
        content.push_str(&format!("EggHatchSpeedMultiplier={}\n", config.multipliers.egg_hatch_speed_multiplier));
        content.push_str(&format!("PoopsIntervalMultiplier={}\n", config.multipliers.poops_interval_multiplier));
        content.push_str(&format!("LayEggIntervalMultiplier={}\n", config.multipliers.lay_egg_interval_multiplier));
        content.push_str(&format!("MatingIntervalMultiplier={}\n", config.multipliers.mating_interval_multiplier));
        content.push_str(&format!("CraftingSkillBonusMultiplier={}\n", config.multipliers.crafting_skill_bonus_multiplier));
        content.push_str(&format!("CraftingSpeedMultiplier={}\n", config.multipliers.crafting_speed_multiplier));

        content.push_str("\n[World]\n");
        content.push_str(&format!("DayCycleSpeedScale={}\n", config.world.day_cycle_speed_scale));
        content.push_str(&format!("NightTimeSpeedScale={}\n", config.world.night_time_speed_scale));
        content.push_str(&format!("DayTimeSpeedScale={}\n", config.world.day_time_speed_scale));
        content.push_str(&format!("OverallDamageMultiplier={}\n", config.world.overall_damage_multiplier));

        if !config.mods.active_mods.is_empty() {
            content.push_str("\n[Mods]\n");
            content.push_str(&format!("ActiveMods={}\n", config.mods.active_mods.join(",")));
        }

        if !config.advanced.custom_config.is_empty() {
            content.push_str("\n[Custom]\n");
            for (key, value) in &config.advanced.custom_config {
                content.push_str(&format!("{}={}\n", key, value));
            }
        }

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_game_ini_content() {
        let config = ServerConfig::default();
        let content = ConfigPersister::generate_game_ini_content(&config);

        assert!(content.contains("[/script/shootergame.shootergamemode]"));
        assert!(content.contains("SessionName=ARK Server"));
        assert!(content.contains("MaxPlayers=70"));
        assert!(content.contains("bServerPVE=true"));
    }

    #[test]
    fn test_generate_gamesettings_ini_content() {
        let config = ServerConfig::default();
        let content = ConfigPersister::generate_gamesettings_ini_content(&config);

        assert!(content.contains("[ServerSettings]"));
        assert!(content.contains("Port=7777"));
        assert!(content.contains("QueryPort=27015"));
        assert!(content.contains("[Multipliers]"));
        assert!(content.contains("XPMultiplier=3"));
        assert!(content.contains("TamingSpeedMultiplier=15"));
    }

    #[test]
    fn test_generate_gamesettings_with_mods() {
        let mut config = ServerConfig::default();
        config.mods.active_mods = vec!["955131".to_string(), "1102729".to_string()];

        let content = ConfigPersister::generate_gamesettings_ini_content(&config);
        assert!(content.contains("[Mods]"));
        assert!(content.contains("ActiveMods=955131,1102729"));
    }
}
