use crate::config::schema::ServerConfig;
use crate::error::{Error, Result};
use std::path::Path;
use tokio::fs;

// Helper to format booleans the way ARK INI expects: True / False
fn b(v: bool) -> &'static str {
    if v { "True" } else { "False" }
}

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

        fs::write(path, &content)
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

        fs::write(path, &content)
            .await
            .map_err(|e| Error::IoError(e))?;

        log::info!("GameUserSettings.ini generated at {:?}", path);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────
    // Game.ini  →  C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\Game.ini
    // Contains gameplay multipliers, stat arrays, and game-mode flags.
    // ─────────────────────────────────────────────────────────────────
    fn generate_game_ini_content(config: &ServerConfig) -> String {
        let m  = &config.multipliers;
        let xp = &config.xp_multipliers;
        let w  = &config.world;
        let pve = &config.pve;
        let pvp = &config.pvp;
        let adv = &config.advanced;
        let gp  = &config.gameplay;

        let mut s = String::new();

        s.push_str("[/Script/ShooterGame.ShooterGameMode]\n");

        // ── Taming / Breeding ───────────────────────────────────────
        s.push_str(&format!("TamingSpeedMultiplier={}\n",              m.taming_speed_multiplier));
        s.push_str(&format!("BabyMatureSpeedMultiplier={}\n",          m.baby_mature_speed_multiplier));
        s.push_str(&format!("BabyCuddleIntervalMultiplier={}\n",       m.baby_cuddle_interval_multiplier));
        s.push_str(&format!("BabyCuddleLoseImprintQualitySpeedMultiplier={}\n", m.baby_cuddle_loss_multiplier));
        s.push_str(&format!("BabyCuddleGracePeriodMultiplier={}\n",    m.baby_cuddle_grace_period_multiplier));
        s.push_str(&format!("BabyImprintingStatScaleMultiplier={}\n",  m.baby_imprint_stat_scale_multiplier));
        s.push_str(&format!("BabyFoodConsumptionSpeedMultiplier={}\n", m.baby_food_consumption_multiplier));
        s.push_str(&format!("EggHatchSpeedMultiplier={}\n",            m.egg_hatch_speed_multiplier));
        s.push_str(&format!("MatingIntervalMultiplier={}\n",           m.mating_interval_multiplier));
        s.push_str(&format!("LayEggIntervalMultiplier={}\n",           m.lay_egg_interval_multiplier));
        s.push_str(&format!("PoopIntervalMultiplier={}\n",             m.poops_interval_multiplier));

        // ── Harvest / XP ────────────────────────────────────────────
        s.push_str(&format!("HarvestAmountMultiplier={}\n",  m.harvest_amount_multiplier));
        s.push_str(&format!("HarvestHealthMultiplier={}\n",  m.harvest_health_multiplier));
        s.push_str(&format!("XPMultiplier={}\n",             m.xp_multiplier));
        s.push_str(&format!("KillXPMultiplier={}\n",         xp.kill_xp_multiplier));
        s.push_str(&format!("HarvestXPMultiplier={}\n",      xp.harvest_xp_multiplier));
        s.push_str(&format!("CraftXPMultiplier={}\n",        xp.craft_xp_multiplier));
        s.push_str(&format!("SpecialXPMultiplier={}\n",      xp.special_xp_multiplier));
        s.push_str(&format!("ExplorerNoteXPMultiplier={}\n", xp.explorer_note_xp_multiplier));
        s.push_str(&format!("BossKillXPMultiplier={}\n",     xp.boss_kill_xp_multiplier));
        s.push_str(&format!("AlphaKillXPMultiplier={}\n",    xp.alpha_kill_xp_multiplier));
        s.push_str(&format!("WildKillXPMultiplier={}\n",     xp.wild_kill_xp_multiplier));
        s.push_str(&format!("CaveKillXPMultiplier={}\n",     xp.cave_kill_xp_multiplier));
        s.push_str(&format!("TamedKillXPMultiplier={}\n",    xp.tamed_kill_xp_multiplier));
        s.push_str(&format!("GenericXPMultiplier={}\n",      xp.generic_xp_multiplier));

        // ── Player multipliers ───────────────────────────────────────
        s.push_str(&format!("PlayerDamageMultiplier={}\n",                     m.player_damage_multiplier));
        s.push_str(&format!("PlayerResistanceMultiplier={}\n",                 m.player_resistance_multiplier));
        s.push_str(&format!("PlayerCharacterWaterDrainMultiplier={}\n",        m.player_character_water_drain_multiplier));
        s.push_str(&format!("PlayerCharacterFoodDrainMultiplier={}\n",         m.player_character_food_drain_multiplier));
        s.push_str(&format!("PlayerCharacterStaminaDrainMultiplier={}\n",      m.player_character_stamina_drain_multiplier));
        s.push_str(&format!("PlayerCharacterHealthRecoveryMultiplier={}\n",    m.player_character_health_recovery_multiplier));

        // ── Dino multipliers ─────────────────────────────────────────
        s.push_str(&format!("DinoCountMultiplier={}\n",                        gp.dino_count_multiplier));
        s.push_str(&format!("DinoCharacterHealthRecoveryMultiplier={}\n",      m.dino_character_health_multiplier));
        s.push_str(&format!("DinoCharacterFoodDrainMultiplier={}\n",           m.dino_character_food_drain_multiplier));
        s.push_str(&format!("DinoCharacterStaminaDrainMultiplier={}\n",        m.dino_character_stamina_drain_multiplier));
        s.push_str(&format!("DinoDamageMultiplier={}\n",                       m.dino_damage_multiplier));
        s.push_str(&format!("DinoResistanceMultiplier={}\n",                   m.dino_resistance_multiplier));
        s.push_str(&format!("RaidDinoCharacterFoodDrainMultiplier={}\n",       m.dino_character_food_drain_multiplier));

        // ── Structure multipliers ────────────────────────────────────
        s.push_str(&format!("StructureDamageMultiplier={}\n",      m.structure_damage_multiplier));
        s.push_str(&format!("StructureResistanceMultiplier={}\n",  m.structure_resistance_multiplier));

        // ── Crafting ─────────────────────────────────────────────────
        s.push_str(&format!("CraftingSkillBonusMultiplier={}\n",  m.crafting_skill_bonus_multiplier));

        // ── World / Time ─────────────────────────────────────────────
        s.push_str(&format!("DayCycleSpeedScale={}\n",                        w.day_cycle_speed_scale));
        s.push_str(&format!("DayTimeSpeedScale={}\n",                          w.day_time_speed_scale));
        s.push_str(&format!("NightTimeSpeedScale={}\n",                        w.night_time_speed_scale));
        s.push_str(&format!("GlobalSpoilingTimeMultiplier={}\n",               w.global_spoiling_time_multiplier));
        s.push_str(&format!("GlobalItemDecompositionTimeMultiplier={}\n",      w.global_item_decomposition_time_multiplier));
        s.push_str(&format!("GlobalCorpseDecompositionTimeMultiplier={}\n",    w.global_corpse_decomposition_time_multiplier));
        s.push_str(&format!("ResourceNoReplenishRadiusPlayers={}\n",           w.resource_no_replenish_radius_players));
        s.push_str(&format!("ResourceNoReplenishRadiusStructures={}\n",        w.resource_no_replenish_radius_structures));
        s.push_str(&format!("ResourcesRespawnPeriodMultiplier={}\n",           w.resource_respawn_period_multiplier));
        s.push_str(&format!("CropGrowthSpeedMultiplier={}\n",                  w.crop_growth_speed_multiplier));
        s.push_str(&format!("CropDecaySpeedMultiplier={}\n",                   w.crop_decay_speed_multiplier));
        s.push_str(&format!("FuelConsumptionIntervalMultiplier={}\n",          w.fuel_consumption_interval_multiplier));
        s.push_str(&format!("OverallDamageMultiplier={}\n",                    w.overall_damage_multiplier));
        s.push_str(&format!("PlayerCharacterHealthMultiplier={}\n",            w.player_character_health_multiplier));
        s.push_str(&format!("DinoCharacterHealthMultiplier={}\n",              w.dino_character_health_multiplier));
        // force_reset_wild_dinos is a startup flag (-ForceRespawnDinos) not an INI key;
        // append to custom_config if the user enables it via the app.

        // ── PVE settings ─────────────────────────────────────────────
        s.push_str(&format!("bDisableStructureDecayPVE={}\n",           b(pve.disable_structure_decay_pve)));
        s.push_str(&format!("PvEStructureDecayPeriodMultiplier={}\n",   pve.structure_decay_period_multiplier));
        s.push_str(&format!("bDisableDinoDecayPVE={}\n",                b(pve.disable_dino_decay_pve)));
        s.push_str(&format!("PvEDinoDecayPeriodMultiplier={}\n",        pve.dino_decay_period_multiplier));
        s.push_str(&format!("bAllowCaveBuildingPVE={}\n",               b(pve.allow_cave_building)));
        s.push_str(&format!("bFlyerPlatformAllowUnalignedDinoBasing={}\n", b(pve.allow_flyer_carry)));
        s.push_str(&format!("bForceAllowCaveFlyers={}\n",               b(pve.force_allow_cave_flyers)));
        s.push_str(&format!("bExtraStructurePreventionVolumes={}\n",    b(pve.extra_structure_prevention_volumes)));
        s.push_str(&format!("bPreventDiseases={}\n",                    b(pve.prevent_diseases)));
        s.push_str(&format!("bNonPermanentDiseases={}\n",               b(pve.non_permanent_diseases)));
        s.push_str(&format!("bPreventTribeAlliances={}\n",              b(pve.prevent_tribe_alliances)));
        s.push_str(&format!("bPvEAllowTribeWar={}\n",                   b(pve.pve_allow_tribe_war)));
        s.push_str(&format!("bPvEAllowTribeWarCancel={}\n",             b(pve.pve_allow_tribe_war_cancel)));

        // ── PVP settings ─────────────────────────────────────────────
        s.push_str(&format!("bPvPDinoDecay={}\n",                                b(pvp.pvp_dino_decay)));
        s.push_str(&format!("bGenesisUseStructuresPreventionVolumes={}\n",       b(pvp.override_structure_platform_prevention)));
        s.push_str(&format!("bIncreasePvPRespawnInterval={}\n",                  b(pvp.increase_pvp_respawn_interval)));
        s.push_str(&format!("IncreasePvPRespawnIntervalCheckPeriod={}\n",        pvp.increase_pvp_respawn_interval_check_period));
        s.push_str(&format!("IncreasePvPRespawnIntervalMultiplier={}\n",         pvp.increase_pvp_respawn_interval_multiplier));
        s.push_str(&format!("IncreasePvPRespawnIntervalBaseAmount={}\n",         pvp.increase_pvp_respawn_interval_base_amount));
        s.push_str(&format!("PvPZoneStructureDamageMultiplier={}\n",             pvp.pvp_zone_structure_damage_multiplier));
        s.push_str(&format!("StructureDamageRepairCooldown={}\n",                pvp.structure_damage_repair_cooldown));

        // ── Advanced / Misc ───────────────────────────────────────────
        s.push_str(&format!("bAllowUnlimitedRespecs={}\n",                   b(adv.allow_unlimited_respecs)));
        s.push_str(&format!("MaxPersonalTamedDinos={}\n",                    adv.max_tamed_dinos));
        s.push_str(&format!("bPassiveTameDefensesDamageRiderlessDinos={}\n", b(adv.passive_defenses_damage_riderless_dinos)));
        s.push_str(&format!("bAllowRaidDinoFeeding={}\n",                    b(adv.allow_raid_dino_feeding)));
        s.push_str(&format!("bDisableDinoRiding={}\n",                       b(adv.disable_dino_riding)));
        s.push_str(&format!("bDisableDinoTaming={}\n",                       b(adv.disable_dino_taming)));
        s.push_str(&format!("bDisableDefaultDinoTaming={}\n",                b(adv.disable_default_dino_taming)));
        s.push_str(&format!("bAllowCustomRecipes={}\n",                      b(adv.allow_custom_recipes)));
        s.push_str(&format!("CustomRecipeEffectivenessMultiplier={}\n",      adv.custom_recipe_effectiveness_multiplier));
        s.push_str(&format!("CustomRecipeSkillMultiplier={}\n",              adv.custom_recipe_skill_multiplier));
        s.push_str(&format!("SupplyCrateLootQualityMultiplier={}\n",         adv.supply_crate_loot_quality_multiplier));
        s.push_str(&format!("FishingLootQualityMultiplier={}\n",             adv.fishing_loot_quality_multiplier));
        s.push_str(&format!("PlatformSaddleBuildAreaBoundsMultiplier={}\n",  adv.platform_structure_limit));
        s.push_str(&format!("bDisableFriendlyFire={}\n",                     b(adv.disable_friendly_fire)));

        // ── Per-level stat multipliers — Wild Dinos (indices 0-9) ────
        // Index: 0=Health 1=Stamina 2=Torpidity 3=Oxygen 4=Food 5=Water
        //        6=Temperature 7=Weight 8=MeleeDamage 9=Speed
        // (ARK uses a fixed mapping that differs from the display order)
        let wild = &config.dino_wild_stats;
        let wild_vals = [
            wild.health, wild.stamina, wild.torpidity, wild.oxygen,
            wild.food, wild.water, 1.0 /*temp*/, wild.weight,
            wild.melee_damage, wild.speed,
        ];
        for (i, v) in wild_vals.iter().enumerate() {
            s.push_str(&format!("PerLevelStatsMultiplier_DinoWild[{}]={}\n", i, v));
        }

        // ── Per-level stat multipliers — Tamed Dinos ─────────────────
        let tp = &config.dino_tamed_stats.per_level;
        let tp_vals = [
            tp.health, tp.stamina, tp.torpidity, tp.oxygen,
            tp.food, tp.water, 1.0, tp.weight, tp.melee_damage, tp.speed,
        ];
        for (i, v) in tp_vals.iter().enumerate() {
            s.push_str(&format!("PerLevelStatsMultiplier_DinoTamed[{}]={}\n", i, v));
        }

        // ── Add per-level (after tame bonus) ─────────────────────────
        let ta = &config.dino_tamed_stats.add_per_level;
        let ta_vals = [
            ta.health, ta.stamina, ta.torpidity, ta.oxygen,
            ta.food, ta.water, 1.0, ta.weight, ta.melee_damage, ta.speed,
        ];
        for (i, v) in ta_vals.iter().enumerate() {
            s.push_str(&format!("PerLevelStatsMultiplier_DinoTamed_Add[{}]={}\n", i, v));
        }

        // ── Affinity (imprint bonus) ──────────────────────────────────
        let af = &config.dino_tamed_stats.affinity;
        let af_vals = [
            af.health, af.stamina, af.torpidity, af.oxygen,
            af.food, af.water, 1.0, af.weight, af.melee_damage, af.speed,
        ];
        for (i, v) in af_vals.iter().enumerate() {
            s.push_str(&format!("PerLevelStatsMultiplier_DinoTamed_Affinity[{}]={}\n", i, v));
        }

        // ── Per-level stat multipliers — Players ─────────────────────
        // Index: 0=Health,1=Stamina,2=Torpidity,3=Oxygen,4=Food,5=Water,6=Temp,7=Weight,8=Melee,9=Speed,10=CraftingSpeed
        let ps = &config.player_stats;
        let ps_vals = [
            ps.health, ps.stamina, 1.0 /*torpidity*/, ps.oxygen,
            ps.food, ps.water, 1.0 /*temp*/, ps.weight, ps.melee_damage, ps.speed,
            ps.crafting_speed,
        ];
        for (i, v) in ps_vals.iter().enumerate() {
            s.push_str(&format!("PerLevelStatsMultiplier_Player[{}]={}\n", i, v));
        }

        // ── Custom config overrides (passthrough) ────────────────────
        for (k, v) in &adv.custom_config {
            s.push_str(&format!("{}={}\n", k, v));
        }

        s
    }

    // ─────────────────────────────────────────────────────────────────
    // GameUserSettings.ini  →  ...WindowsServer\GameUserSettings.ini
    // Contains server identification, network, session settings, mods.
    // ─────────────────────────────────────────────────────────────────
    fn generate_gamesettings_ini_content(config: &ServerConfig) -> String {
        let id  = &config.identification;
        let net = &config.network;
        let gp  = &config.gameplay;
        let adv = &config.advanced;
        let m   = &config.multipliers;

        let mut s = String::new();

        // ── [ServerSettings] ─────────────────────────────────────────
        s.push_str("[ServerSettings]\n");
        s.push_str(&format!("SessionName={}\n",       id.session_name));
        s.push_str(&format!("ServerPassword={}\n",    id.server_password));
        s.push_str(&format!("ServerAdminPassword={}\n", id.admin_password));
        s.push_str(&format!("MessageOfTheDay={}\n",   id.server_message_of_the_day));
        s.push_str(&format!("ServerPVE={}\n",         b(gp.server_pve)));
        s.push_str(&format!("ServerHardcore={}\n",    b(gp.server_hardcore)));
        s.push_str(&format!("DifficultyOffset={}\n",  gp.difficulty_offset));
        s.push_str(&format!("OverrideOfficialDifficulty={}\n", gp.override_official_difficulty));
        s.push_str(&format!("XPMultiplier={}\n",      m.xp_multiplier));
        s.push_str(&format!("TamingSpeedMultiplier={}\n", m.taming_speed_multiplier));
        s.push_str(&format!("HarvestAmountMultiplier={}\n", m.harvest_amount_multiplier));
        s.push_str(&format!("MaxTamedDinos={}\n",     adv.max_tamed_dinos));
        // Network
        s.push_str(&format!("Port={}\n",       net.port));
        s.push_str(&format!("QueryPort={}\n",  net.query_port));
        s.push_str(&format!("RCONPort={}\n",   net.rcon_port));
        s.push_str(&format!("RCONEnabled=True\n"));
        // Gameplay flags visible in ServerSettings
        s.push_str(&format!("ServerCrosshair={}\n",              b(gp.server_crosshair)));
        s.push_str(&format!("ShowFloatingDamageText={}\n",       b(gp.show_floating_damage_text)));
        s.push_str(&format!("AllowHitMarkers={}\n",              b(gp.allow_hit_markers)));
        s.push_str(&format!("AllowThirdPersonPlayer={}\n",       b(gp.allow_third_person_player)));
        s.push_str(&format!("AllowCryoSicknessPVE={}\n",         b(adv.allow_cryo_sick_pve)));
        s.push_str(&format!("EnablePvPGamma={}\n",               b(gp.enable_pvp_gamma_bypass)));
        s.push_str(&format!("DisablePvEGamma={}\n",              b(gp.disable_pvp_gamma)));
        s.push_str(&format!("AllowSpeedLeveling={}\n",           b(gp.allow_speed_leveling)));
        s.push_str(&format!("AllowFlyerSpeedLeveling={}\n",      b(gp.allow_flyer_speed_leveling)));
        s.push_str(&format!("ProximityChat={}\n",                b(gp.proximity_chat)));
        s.push_str(&format!("GlobalVoiceChat={}\n",              b(gp.global_voice_chat)));
        s.push_str(&format!("AdminLogging={}\n",                 b(gp.admin_logging)));
        s.push_str(&format!("AlwaysNotifyPlayerLeft={}\n",       b(gp.always_notify_player_left)));
        s.push_str(&format!("DontAlwaysNotifyPlayerJoined={}\n", b(gp.dont_always_notify_player_joined)));
        s.push_str(&format!("KickIdlePlayersPeriod={}\n",        gp.kick_idle_players_period));
        // Downloads
        s.push_str(&format!("NoTributeDownloads={}\n",  b(adv.no_survivor_downloads)));
        s.push_str(&format!("NoDinoDownloads={}\n",     b(adv.no_dino_downloads)));
        s.push_str(&format!("NoItemDownloads={}\n",     b(adv.no_item_downloads)));
        // Mods — ASA uses ActiveMods (comma-separated CurseForge IDs) in [ServerSettings]
        if !config.mods.active_mods.is_empty() {
            s.push_str(&format!("ActiveMods={}\n", config.mods.active_mods.join(",")));
        }
        // Server platform (Steam/EOS/ALL)
        s.push_str(&format!("ServerPlatform={}\n", net.server_platform));
        // ── ASA-specific settings ────────────────────────────────────
        s.push_str("DisableCryopodFridgeRequirement=True\n");
        s.push_str("DisableCryopodEnemyCheck=True\n");
        s.push_str("AllowDinoAIInsideBunkers=True\n");
        s.push_str("AllowRidingDinosInsideBunkers=True\n");
        s.push_str("AlwaysAllowStructurePickup=True\n");
        s.push_str("StructurePickupHoldDuration=0.5\n");
        s.push_str("StructurePickupTimeAfterPlacement=30\n");
        s.push_str("AutoSavePeriodMinutes=15\n");
        s.push_str("ShowMapPlayerLocation=True\n");
        s.push_str("ItemStackSizeMultiplier=1\n");
        s.push_str("AllowAnyoneBabyImprintCuddle=False\n");
        s.push_str("RandomSupplyCratePoints=False\n");
        s.push_str(&format!("TheMaxStructuresInRange={}\n", config.performance.max_structure_in_range));

        // ── [/Script/Engine.GameSession] ─────────────────────────────
        s.push_str("\n[/Script/Engine.GameSession]\n");
        s.push_str(&format!("MaxPlayers={}\n", gp.max_players));

        // ── [/Script/ShooterGame.ShooterPlayerController] ────────────
        s.push_str("\n[/Script/ShooterGame.ShooterPlayerController]\n");
        s.push_str(&format!("bAllowThirdPersonPlayer={}\n",      b(gp.allow_third_person_player)));
        s.push_str(&format!("bAlwaysNotifyPlayerLeft={}\n",      b(gp.always_notify_player_left)));
        s.push_str(&format!("bDontAlwaysNotifyPlayerJoined={}\n",b(gp.dont_always_notify_player_joined)));

        s
    }
}
