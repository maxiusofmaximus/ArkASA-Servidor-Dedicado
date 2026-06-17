use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ServerConfig {
    /// List of maps to run. 1 map = single server, 2+ maps = cluster mode.
    /// Old configs with `map = "..."` will silently default to ["TheIsland_WP"].
    pub cluster_maps: Vec<String>,
    pub identification: IdentificationConfig,
    pub network: NetworkConfig,
    pub gameplay: GameplayConfig,
    pub multipliers: MultipliersConfig,
    pub xp_multipliers: XpMultipliersConfig,
    pub dino_wild_stats: DinoStatsPerLevel,
    pub dino_tamed_stats: DinoTamedStatsConfig,
    pub player_stats: PlayerStatsPerLevel,
    pub mods: ModsConfig,
    pub paths: PathsConfig,
    pub performance: PerformanceConfig,
    pub world: WorldConfig,
    pub pve: PveConfig,
    pub pvp: PvpConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionMethod {
    Tailscale,
    Public,
    DuckDns,
    Local,
    #[default]
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NetworkConfig {
    pub port: u16,
    pub query_port: u16,
    pub rcon_port: u16,
    pub server_platform: String,
    // ── Connection Manager ────────────────────────────────────────────────────
    pub connection_method: ConnectionMethod,
    pub tailscale_ip: String,
    pub public_ip: String,
    pub duckdns_host: String,
    pub local_ip: String,
    /// Generic / migration target for old server_ip values.
    pub manual_ip: String,
    // ── Legacy: kept to read old TOML files; effective_ip() falls back to it ─
    #[serde(default)]
    pub server_ip: String,
}

impl NetworkConfig {
    /// Returns the IP string ARK receives as -ip=VALUE.
    /// Empty string → omit -ip flag (ARK binds all interfaces).
    pub fn effective_ip(&self) -> String {
        let resolved = match self.connection_method {
            ConnectionMethod::Tailscale => self.tailscale_ip.trim().to_string(),
            ConnectionMethod::Public    => self.public_ip.trim().to_string(),
            ConnectionMethod::DuckDns   => self.duckdns_host.trim().to_string(),
            ConnectionMethod::Local     => self.local_ip.trim().to_string(),
            ConnectionMethod::Manual    => self.manual_ip.trim().to_string(),
        };
        if resolved.is_empty() { self.server_ip.trim().to_string() } else { resolved }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GameplayConfig {
    pub server_pve: bool,
    pub server_hardcore: bool,
    pub max_players: u16,
    pub difficulty_offset: f32,
    pub override_official_difficulty: f32,
    pub dino_count_multiplier: f32,
    pub enable_pvp_gamma_bypass: bool,
    pub disable_pvp_gamma: bool,
    pub allow_third_person_player: bool,
    pub allow_cryopod_nerf_removal: bool,
    pub allow_speed_leveling: bool,
    pub allow_flyer_speed_leveling: bool,
    pub allow_unlimited_respecs: bool,
    pub show_floating_damage_text: bool,
    pub allow_hit_markers: bool,
    pub server_crosshair: bool,
    pub force_no_hud: bool,
    pub proximity_chat: bool,
    pub global_voice_chat: bool,
    pub admin_logging: bool,
    pub always_notify_player_left: bool,
    pub dont_always_notify_player_joined: bool,
    pub kick_idle_players_period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MultipliersConfig {
    pub xp_multiplier: f32,
    pub taming_speed_multiplier: f32,
    pub harvest_amount_multiplier: f32,
    pub harvest_health_multiplier: f32,
    pub player_damage_multiplier: f32,
    pub player_resistance_multiplier: f32,
    pub player_character_water_drain_multiplier: f32,
    pub player_character_food_drain_multiplier: f32,
    pub player_character_stamina_drain_multiplier: f32,
    pub player_character_health_recovery_multiplier: f32,
    pub dino_damage_multiplier: f32,
    pub dino_resistance_multiplier: f32,
    pub dino_character_health_multiplier: f32,
    pub dino_character_food_drain_multiplier: f32,
    pub dino_character_stamina_drain_multiplier: f32,
    pub structure_damage_multiplier: f32,
    pub structure_resistance_multiplier: f32,
    pub baby_mature_speed_multiplier: f32,
    pub baby_food_consumption_multiplier: f32,
    pub baby_cuddle_loss_multiplier: f32,
    pub baby_cuddle_interval_multiplier: f32,
    pub baby_cuddle_grace_period_multiplier: f32,
    pub baby_imprint_stat_scale_multiplier: f32,
    pub egg_hatch_speed_multiplier: f32,
    pub poops_interval_multiplier: f32,
    pub lay_egg_interval_multiplier: f32,
    pub mating_interval_multiplier: f32,
    pub crafting_skill_bonus_multiplier: f32,
    pub crafting_speed_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct XpMultipliersConfig {
    pub generic_xp_multiplier: f32,
    pub kill_xp_multiplier: f32,
    pub harvest_xp_multiplier: f32,
    pub craft_xp_multiplier: f32,
    pub special_xp_multiplier: f32,
    pub explorer_note_xp_multiplier: f32,
    pub boss_kill_xp_multiplier: f32,
    pub alpha_kill_xp_multiplier: f32,
    pub wild_kill_xp_multiplier: f32,
    pub cave_kill_xp_multiplier: f32,
    pub tamed_kill_xp_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DinoStatsPerLevel {
    pub health: f32,
    pub stamina: f32,
    pub oxygen: f32,
    pub food: f32,
    pub water: f32,
    pub weight: f32,
    pub melee_damage: f32,
    pub speed: f32,
    pub fortitude: f32,
    pub torpidity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PlayerStatsPerLevel {
    pub health: f32,
    pub stamina: f32,
    pub oxygen: f32,
    pub food: f32,
    pub water: f32,
    pub weight: f32,
    pub melee_damage: f32,
    pub speed: f32,
    pub fortitude: f32,
    pub crafting_speed: f32,
    pub torpidity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DinoTamedStatsConfig {
    pub per_level: DinoStatsPerLevel,
    pub add_per_level: DinoStatsPerLevel,
    pub affinity: DinoStatsPerLevel,
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
    pub global_spoiling_time_multiplier: f32,
    pub global_item_decomposition_time_multiplier: f32,
    pub global_corpse_decomposition_time_multiplier: f32,
    pub resource_no_replenish_radius_players: f32,
    pub resource_no_replenish_radius_structures: f32,
    pub resource_respawn_period_multiplier: f32,
    pub crop_growth_speed_multiplier: f32,
    pub crop_decay_speed_multiplier: f32,
    pub fuel_consumption_interval_multiplier: f32,
    pub force_reset_wild_dinos: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PveConfig {
    pub allow_cave_building: bool,
    pub disable_structure_decay_pve: bool,
    pub structure_decay_period_multiplier: f32,
    pub disable_dino_decay_pve: bool,
    pub dino_decay_period_multiplier: f32,
    pub force_allow_cave_flyers: bool,
    pub allow_flyer_carry: bool,
    pub extra_structure_prevention_volumes: bool,
    pub prevent_diseases: bool,
    pub non_permanent_diseases: bool,
    pub prevent_tribe_alliances: bool,
    pub pve_allow_tribe_war: bool,
    pub pve_allow_tribe_war_cancel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PvpConfig {
    pub pvp_dino_decay: bool,
    pub override_structure_platform_prevention: bool,
    pub increase_pvp_respawn_interval: bool,
    pub increase_pvp_respawn_interval_check_period: u32,
    pub increase_pvp_respawn_interval_multiplier: f32,
    pub increase_pvp_respawn_interval_base_amount: u32,
    pub pvp_zone_structure_damage_multiplier: f32,
    pub structure_damage_repair_cooldown: u32,
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
    pub disable_dino_riding: bool,
    pub disable_dino_taming: bool,
    pub disable_default_dino_taming: bool,
    pub max_tamed_dinos: u32,
    pub allow_raid_dino_feeding: bool,
    pub passive_defenses_damage_riderless_dinos: bool,
    pub allow_platform_saddle_multi_floors: bool,
    pub disable_photo_mode: bool,
    pub photo_mode_range_limit: f32,
    pub allow_custom_recipes: bool,
    pub custom_recipe_effectiveness_multiplier: f32,
    pub custom_recipe_skill_multiplier: f32,
    pub supply_crate_loot_quality_multiplier: f32,
    pub fishing_loot_quality_multiplier: f32,
    pub platform_structure_limit: u32,
    pub force_gacha_unhappy_in_caves: bool,
    pub disable_friendly_fire: bool,
    pub disable_structure_placement_collision: bool,
    pub only_allow_specific_engrams: bool,
    pub auto_unlock_engrams: Vec<u32>,
    pub custom_config: HashMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain helpers — used by both start_server and on-demand stubs
// ─────────────────────────────────────────────────────────────────────────────

impl ServerConfig {
    /// Effective map list — defaults to TheIsland_WP for single-server setups.
    pub fn effective_maps(&self) -> Vec<String> {
        if self.cluster_maps.is_empty() {
            vec!["TheIsland_WP".to_string()]
        } else {
            self.cluster_maps.clone()
        }
    }

    /// Safe alphanumeric cluster ID derived from the session name.
    pub fn cluster_id(&self) -> String {
        let id: String = self.identification.session_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .to_lowercase();
        if id.is_empty() { "ark_cluster".to_string() } else { id }
    }
}

impl NetworkConfig {
    /// (game_port, query_port, rcon_port) for cluster instance at index `i`.
    /// ARK uses two consecutive UDP ports per game port, so game_port steps by 2.
    pub fn ports_for_index(&self, i: usize) -> (u16, u16, u16) {
        (
            self.port       + (i as u16) * 2,
            self.query_port + i as u16,
            self.rcon_port  + i as u16,
        )
    }
}

impl PathsConfig {
    /// Absolute path to ArkAscendedServer.exe derived from server_dir.
    pub fn ark_exe(&self) -> String {
        let dir = self.server_dir.trim_end_matches('\\');
        format!("{}\\ShooterGame\\Binaries\\Win64\\ArkAscendedServer.exe", dir)
    }

    /// Cluster save directory (sibling of ShooterGame).
    pub fn cluster_dir(&self) -> String {
        let dir = self.server_dir.trim_end_matches('\\');
        format!("{}\\clusters", dir)
    }
}

// Default implementations
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            cluster_maps: vec!["TheIsland_WP".to_string()],
            identification: IdentificationConfig::default(),
            network: NetworkConfig::default(),
            gameplay: GameplayConfig::default(),
            multipliers: MultipliersConfig::default(),
            xp_multipliers: XpMultipliersConfig::default(),
            dino_wild_stats: DinoStatsPerLevel::default(),
            dino_tamed_stats: DinoTamedStatsConfig::default(),
            player_stats: PlayerStatsPerLevel::default(),
            mods: ModsConfig::default(),
            paths: PathsConfig::default(),
            performance: PerformanceConfig::default(),
            world: WorldConfig::default(),
            pve: PveConfig::default(),
            pvp: PvpConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

impl Default for IdentificationConfig {
    fn default() -> Self {
        Self {
            session_name: "ServidorMax".to_string(),
            server_password: "bhahyvdhavd9954485".to_string(),
            // Note: the original GUS had this line corrupted with extra command-line args appended.
            // The real admin password is the part before the first '?'.
            admin_password: "Bafbv/aHdvhZ*w956545*".to_string(),
            server_message_of_the_day: String::new(),
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
            connection_method: ConnectionMethod::Manual,
            tailscale_ip: String::new(),
            public_ip: String::new(),
            duckdns_host: String::new(),
            local_ip: String::new(),
            manual_ip: String::new(),
            server_ip: String::new(),
        }
    }
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            server_pve: true,
            server_hardcore: false,
            max_players: 70,
            difficulty_offset: 2.0,
            override_official_difficulty: 6.0, // dino max level 180
            dino_count_multiplier: 1.0,
            enable_pvp_gamma_bypass: true,  // EnablePvPGamma=True
            disable_pvp_gamma: false,
            allow_third_person_player: true,
            allow_cryopod_nerf_removal: false,
            allow_speed_leveling: false,
            allow_flyer_speed_leveling: false,
            allow_unlimited_respecs: true,
            show_floating_damage_text: true,
            allow_hit_markers: true,
            server_crosshair: true,
            force_no_hud: false,
            proximity_chat: false,
            global_voice_chat: false,
            admin_logging: false,
            always_notify_player_left: false,
            dont_always_notify_player_joined: false,
            kick_idle_players_period: 3600,
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
            player_damage_multiplier: 3.0,
            player_resistance_multiplier: 1.0,
            player_character_water_drain_multiplier: 0.252,
            player_character_food_drain_multiplier: 0.246,
            player_character_stamina_drain_multiplier: 1.254,
            player_character_health_recovery_multiplier: 3.0,
            dino_damage_multiplier: 1.0,
            dino_resistance_multiplier: 1.0,
            dino_character_health_multiplier: 3.0, // DinoCharacterHealthRecoveryMultiplier
            dino_character_food_drain_multiplier: 5.0,
            dino_character_stamina_drain_multiplier: 0.248,
            structure_damage_multiplier: 1.0,
            structure_resistance_multiplier: 0.264,
            baby_mature_speed_multiplier: 40.0,
            baby_food_consumption_multiplier: 4.0,
            baby_cuddle_loss_multiplier: 1.0,
            baby_cuddle_interval_multiplier: 0.074,
            baby_cuddle_grace_period_multiplier: 1.0,
            baby_imprint_stat_scale_multiplier: 2.033,
            egg_hatch_speed_multiplier: 20.0,
            poops_interval_multiplier: 1.0,
            lay_egg_interval_multiplier: 5.039,
            mating_interval_multiplier: 2.975,
            crafting_skill_bonus_multiplier: 3.0,
            crafting_speed_multiplier: 3.0,
        }
    }
}

impl Default for XpMultipliersConfig {
    fn default() -> Self {
        Self {
            generic_xp_multiplier: 3.0,
            kill_xp_multiplier: 3.0,
            harvest_xp_multiplier: 3.0,
            craft_xp_multiplier: 3.0,
            special_xp_multiplier: 3.0,
            explorer_note_xp_multiplier: 2.0,
            boss_kill_xp_multiplier: 1.0,
            alpha_kill_xp_multiplier: 1.0,
            wild_kill_xp_multiplier: 1.0,
            cave_kill_xp_multiplier: 1.0,
            tamed_kill_xp_multiplier: 1.0,
        }
    }
}

impl Default for DinoStatsPerLevel {
    fn default() -> Self {
        Self {
            health: 1.0,
            stamina: 1.0,
            oxygen: 1.0,
            food: 1.0,
            water: 1.0,
            weight: 1.0,
            melee_damage: 1.0,
            speed: 1.0,
            fortitude: 1.0,
            torpidity: 1.0,
        }
    }
}

impl Default for PlayerStatsPerLevel {
    fn default() -> Self {
        // PerLevelStatsMultiplier_Player[i]: all 3x except crafting_speed (index 10) = 5x
        Self {
            health: 3.0,
            stamina: 3.0,
            oxygen: 3.0,
            food: 3.0,
            water: 3.0,
            weight: 3.0,
            melee_damage: 3.0,
            speed: 3.0,
            fortitude: 3.0,
            crafting_speed: 5.0,
            torpidity: 1.0,
        }
    }
}

impl Default for DinoTamedStatsConfig {
    fn default() -> Self {
        Self {
            // PerLevelStatsMultiplier_DinoTamed[i] — index mapping: 0=Health,1=Stamina,2=Torpidity,3=Oxygen,4=Food,5=Water,7=Weight,8=Melee,9=Speed
            per_level: DinoStatsPerLevel {
                health: 3.0,        // [0]
                stamina: 6.0,       // [1]
                torpidity: 3.0,     // [2]
                oxygen: 5.0,        // [3]
                food: 5.0,          // [4]
                water: 3.0,         // [5]
                weight: 6.0,        // [7]
                melee_damage: 1.283, // [8]
                speed: 2.0,         // [9]
                fortitude: 1.0,
            },
            // PerLevelStatsMultiplier_DinoTamed_Add[i]
            add_per_level: DinoStatsPerLevel {
                health: 3.0,        // [0]
                stamina: 5.0,       // [1]
                torpidity: 3.0,     // [2]
                oxygen: 5.0,        // [3]
                food: 5.0,          // [4]
                water: 3.0,         // [5]
                weight: 5.0,        // [7]
                melee_damage: 2.016, // [8]
                speed: 3.0,         // [9]
                fortitude: 1.0,
            },
            // PerLevelStatsMultiplier_DinoTamed_Affinity[i]
            affinity: DinoStatsPerLevel {
                health: 5.0,        // [0]
                stamina: 6.0,       // [1]
                torpidity: 6.0,     // [2]
                oxygen: 6.0,        // [3]
                food: 6.0,          // [4]
                water: 6.0,         // [5]
                weight: 6.0,        // [7]
                melee_damage: 1.0,  // [8]
                speed: 1.0,         // [9]
                fortitude: 1.0,
            },
        }
    }
}

impl Default for ModsConfig {
    fn default() -> Self {
        Self {
            // User's original active mods (CurseForge IDs)
            active_mods: vec![
                "955131".to_string(),
                "1102729".to_string(),
                "1306435".to_string(),
                "958001".to_string(),
                "1182795".to_string(),
                "932756".to_string(),
                "930494".to_string(),
                "1262693".to_string(),
                "928650".to_string(),
                "953154".to_string(),
                "947033".to_string(),
            ],
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
            day_cycle_speed_scale: 0.504,
            night_time_speed_scale: 1.459,
            day_time_speed_scale: 1.571,
            overall_damage_multiplier: 1.0,
            player_character_health_multiplier: 1.0,
            dino_character_health_multiplier: 1.0,
            global_spoiling_time_multiplier: 0.0,     // items never spoil
            global_item_decomposition_time_multiplier: 0.0, // items never decompose
            global_corpse_decomposition_time_multiplier: 6.0,
            resource_no_replenish_radius_players: 0.502,
            resource_no_replenish_radius_structures: 1.0,
            resource_respawn_period_multiplier: 1.001,
            crop_growth_speed_multiplier: 20.0,
            crop_decay_speed_multiplier: 6.0,
            fuel_consumption_interval_multiplier: 4.0,
            force_reset_wild_dinos: false,
        }
    }
}

impl Default for PveConfig {
    fn default() -> Self {
        Self {
            allow_cave_building: false,
            disable_structure_decay_pve: false,
            structure_decay_period_multiplier: 1.0,
            disable_dino_decay_pve: false,
            dino_decay_period_multiplier: 1.0,
            force_allow_cave_flyers: false,
            allow_flyer_carry: false, // AllowFlyerCarryPvE=False
            extra_structure_prevention_volumes: false,
            prevent_diseases: false,
            non_permanent_diseases: false,
            prevent_tribe_alliances: false,
            pve_allow_tribe_war: true,        // bPvEAllowTribeWar=True
            pve_allow_tribe_war_cancel: false,
        }
    }
}

impl Default for PvpConfig {
    fn default() -> Self {
        Self {
            pvp_dino_decay: false,
            override_structure_platform_prevention: false,
            increase_pvp_respawn_interval: true,
            increase_pvp_respawn_interval_check_period: 300,
            increase_pvp_respawn_interval_multiplier: 2.0,
            increase_pvp_respawn_interval_base_amount: 60,
            pvp_zone_structure_damage_multiplier: 6.0,
            structure_damage_repair_cooldown: 1,
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        let mut custom = HashMap::new();
        // Keys present in user's Game.ini that don't have dedicated schema fields:
        custom.insert("DinoHarvestingDamageMultiplier".to_string(), "5".to_string());
        custom.insert("PlayerHarvestingDamageMultiplier".to_string(), "7".to_string());
        custom.insert("DinoTurretDamageMultiplier".to_string(), "1".to_string());
        custom.insert("bHardLimitTurretsInRange".to_string(), "True".to_string());
        custom.insert("bUseCorpseLocator".to_string(), "True".to_string());
        custom.insert("MaxNumberOfPlayersInTribe".to_string(), "0".to_string());
        custom.insert("UnclaimedKillXPMultiplier".to_string(), "1".to_string());
        custom.insert("OverrideMaxExperiencePointsPlayer".to_string(), "0".to_string());
        custom.insert("OverrideMaxExperiencePointsDino".to_string(), "0".to_string());
        custom.insert("bUseSingleplayerSettings".to_string(), "False".to_string());
        custom.insert("MaxDifficulty".to_string(), "False".to_string());
        Self {
            allow_unlimited_respecs: true,
            allow_flyer_carry: false,
            allow_cryo_sick_pve: false,
            disable_structure_decay: false,
            enable_cave_flyers: false,
            no_survivor_downloads: false,
            no_dino_downloads: false,
            no_item_downloads: false,
            disable_dino_riding: false,
            disable_dino_taming: false,
            disable_default_dino_taming: false,
            max_tamed_dinos: 5000,
            allow_raid_dino_feeding: true,
            passive_defenses_damage_riderless_dinos: true,
            allow_platform_saddle_multi_floors: false,
            disable_photo_mode: false,
            photo_mode_range_limit: 3000.0,
            allow_custom_recipes: true,
            custom_recipe_effectiveness_multiplier: 3.0,
            custom_recipe_skill_multiplier: 3.0,
            supply_crate_loot_quality_multiplier: 2.0,
            fishing_loot_quality_multiplier: 20.0,
            platform_structure_limit: 128,
            force_gacha_unhappy_in_caves: false,
            disable_friendly_fire: true,
            disable_structure_placement_collision: true,
            only_allow_specific_engrams: false,
            auto_unlock_engrams: vec![],
            custom_config: custom,
        }
    }
}
