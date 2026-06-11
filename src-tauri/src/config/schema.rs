use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ServerConfig {
    pub identification: IdentificationConfig,
    pub network: NetworkConfig,
    pub gameplay: GameplayConfig,
    pub multipliers: MultipliersConfig,
    pub mods: ModsConfig,
    pub paths: PathsConfig,
    pub performance: PerformanceConfig,
    pub world: WorldConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct IdentificationConfig {
    pub session_name: String,
    pub server_password: String,
    pub admin_password: String,
    pub server_message_of_the_day: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NetworkConfig {
    pub port: u16,
    pub query_port: u16,
    pub rcon_port: u16,
    pub server_platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GameplayConfig {
    pub server_pve: bool,
    pub max_players: u16,
    pub difficulty_offset: f32,
    pub dino_count_multiplier: f32,
    pub enable_pvp_gamma_bypass: bool,
    pub allow_third_person_player: bool,
    pub allow_cryopod_nerf_removal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MultipliersConfig {
    pub xp_multiplier: f32,
    pub taming_speed_multiplier: f32,
    pub harvest_amount_multiplier: f32,
    pub harvest_health_multiplier: f32,
    pub baby_mature_speed_multiplier: f32,
    pub baby_food_consumption_multiplier: f32,
    pub baby_cuddle_loss_multiplier: f32,
    pub egg_hatch_speed_multiplier: f32,
    pub poops_interval_multiplier: f32,
    pub lay_egg_interval_multiplier: f32,
    pub mating_interval_multiplier: f32,
    pub crafting_skill_bonus_multiplier: f32,
    pub crafting_speed_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ModsConfig {
    pub active_mods: Vec<String>,
    pub mod_config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PathsConfig {
    pub steam_cmd_dir: String,
    pub server_dir: String,
    pub backup_dir: String,
    pub game_ini_path: String,
    pub gamesettings_ini_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PerformanceConfig {
    pub max_structure_in_range: u32,
    pub structure_prevention_radius: f32,
    pub use_optimization: bool,
    pub enable_debug_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WorldConfig {
    pub day_cycle_speed_scale: f32,
    pub night_time_speed_scale: f32,
    pub day_time_speed_scale: f32,
    pub overall_damage_multiplier: f32,
    pub player_character_health_multiplier: f32,
    pub dino_character_health_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AdvancedConfig {
    pub allow_unlimited_respecs: bool,
    pub allow_flyer_carry: bool,
    pub allow_cryo_sick_pve: bool,
    pub disable_structure_decay: bool,
    pub enable_cave_flyers: bool,
    pub no_survivor_downloads: bool,
    pub no_dino_downloads: bool,
    pub no_item_downloads: bool,
    pub custom_config: HashMap<String, String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            identification: IdentificationConfig::default(),
            network: NetworkConfig::default(),
            gameplay: GameplayConfig::default(),
            multipliers: MultipliersConfig::default(),
            mods: ModsConfig::default(),
            paths: PathsConfig::default(),
            performance: PerformanceConfig::default(),
            world: WorldConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

impl Default for IdentificationConfig {
    fn default() -> Self {
        Self {
            session_name: "ARK Server".to_string(),
            server_password: "changeme".to_string(),
            admin_password: "changeme".to_string(),
            server_message_of_the_day: "Welcome to ARK!".to_string(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: 7777,
            query_port: 27015,
            rcon_port: 27020,
            server_platform: "ALL".to_string(),
        }
    }
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            server_pve: true,
            max_players: 70,
            difficulty_offset: 2.0,
            dino_count_multiplier: 2.0,
            enable_pvp_gamma_bypass: false,
            allow_third_person_player: true,
            allow_cryopod_nerf_removal: false,
        }
    }
}

impl Default for MultipliersConfig {
    fn default() -> Self {
        Self {
            xp_multiplier: 3.0,
            taming_speed_multiplier: 15.0,
            harvest_amount_multiplier: 8.0,
            harvest_health_multiplier: 3.0,
            baby_mature_speed_multiplier: 40.0,
            baby_food_consumption_multiplier: 4.0,
            baby_cuddle_loss_multiplier: 0.07,
            egg_hatch_speed_multiplier: 20.0,
            poops_interval_multiplier: 1.0,
            lay_egg_interval_multiplier: 5.04,
            mating_interval_multiplier: 2.98,
            crafting_skill_bonus_multiplier: 3.0,
            crafting_speed_multiplier: 3.0,
        }
    }
}

impl Default for ModsConfig {
    fn default() -> Self {
        Self {
            active_mods: vec![],
            mod_config: HashMap::new(),
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            steam_cmd_dir: "C:\\ASA\\steamcmd".to_string(),
            server_dir: "C:\\ASA\\server".to_string(),
            backup_dir: "C:\\ASA\\backups".to_string(),
            game_ini_path: "C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\Game.ini".to_string(),
            gamesettings_ini_path: "C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\GameUserSettings.ini".to_string(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_structure_in_range: 10500,
            structure_prevention_radius: 1000.0,
            use_optimization: true,
            enable_debug_logging: false,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            day_cycle_speed_scale: 0.5,
            night_time_speed_scale: 1.46,
            day_time_speed_scale: 0.5,
            overall_damage_multiplier: 1.0,
            player_character_health_multiplier: 3.0,
            dino_character_health_multiplier: 1.0,
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            allow_unlimited_respecs: true,
            allow_flyer_carry: true,
            allow_cryo_sick_pve: false,
            disable_structure_decay: false,
            enable_cave_flyers: true,
            no_survivor_downloads: false,
            no_dino_downloads: false,
            no_item_downloads: false,
            custom_config: HashMap::new(),
        }
    }
}
