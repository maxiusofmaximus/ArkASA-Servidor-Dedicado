/**
 * Safe Tauri service that handles both dev and production modes
 */

import { logger } from './logger'

let invokeFunction: ((cmd: string, args?: any) => Promise<any>) | null = null
let isTauriAvailable = false

/**
 * Initialize Tauri invoke function
 * Handles both dev mode (mock) and production mode (real Tauri)
 */
export async function initializeTauri() {
  logger.info('Initializing Tauri connection...')

  // Check if we're in Tauri context by checking for __TAURI__ global
  const isTauriContext = typeof window !== 'undefined' && '__TAURI__' in window

  if (!isTauriContext) {
    logger.info('Not in Tauri context - using mock implementation')
    invokeFunction = createMockInvoke()
    isTauriAvailable = false
    return false
  }

  try {
    // We're in Tauri context, try to import the real invoke
    const { invoke } = await import('@tauri-apps/api/core')

    if (!invoke || typeof invoke !== 'function') {
      throw new Error('invoke is not a function')
    }

    logger.info('Tauri invoke loaded successfully')
    invokeFunction = invoke
    isTauriAvailable = true
    return true
  } catch (error) {
    logger.warn('Failed to load Tauri invoke, falling back to mock', error)
    logger.info('Running in dev mode - will use mock implementation')

    // Fallback for dev mode
    invokeFunction = createMockInvoke()
    isTauriAvailable = false
    return false
  }
}

/**
 * Create mock implementation for dev mode
 */
function createMockInvoke() {
  logger.info('Setting up mock Tauri invoke for development')

  return async (command: string, args?: any) => {
    logger.debug(`Mock invoke called: ${command}`, args)

    // Mock responses for development
    const mocks: Record<string, any> = {
      get_default_config: {
        identification: {
          session_name: 'ARK Server',
          server_password: 'changeme',
          admin_password: 'changeme',
          server_message_of_the_day: 'Welcome to ARK!',
        },
        network: {
          port: 7777,
          query_port: 27015,
          rcon_port: 27020,
          server_platform: 'ALL',
        },
        gameplay: {
          server_pve: true,
          server_hardcore: false,
          max_players: 70,
          difficulty_offset: 2.0,
          override_official_difficulty: 4.6,
          dino_count_multiplier: 2.0,
          enable_pvp_gamma_bypass: false,
          disable_pvp_gamma: false,
          allow_third_person_player: true,
          allow_cryopod_nerf_removal: false,
          allow_speed_leveling: true,
          allow_flyer_speed_leveling: true,
          allow_unlimited_respecs: true,
          show_floating_damage_text: true,
          allow_hit_markers: true,
          server_crosshair: false,
          force_no_hud: false,
          proximity_chat: true,
          global_voice_chat: true,
          admin_logging: true,
          always_notify_player_left: false,
          dont_always_notify_player_joined: false,
          kick_idle_players_period: 3600,
        },
        multipliers: {
          xp_multiplier: 3.0,
          taming_speed_multiplier: 15.0,
          harvest_amount_multiplier: 8.0,
          harvest_health_multiplier: 3.0,
          player_damage_multiplier: 1.0,
          player_resistance_multiplier: 1.0,
          player_character_water_drain_multiplier: 1.0,
          player_character_food_drain_multiplier: 1.0,
          player_character_stamina_drain_multiplier: 1.0,
          player_character_health_recovery_multiplier: 1.0,
          dino_damage_multiplier: 1.0,
          dino_resistance_multiplier: 1.0,
          dino_character_health_multiplier: 1.0,
          dino_character_food_drain_multiplier: 1.0,
          dino_character_stamina_drain_multiplier: 1.0,
          structure_damage_multiplier: 1.0,
          structure_resistance_multiplier: 1.0,
          baby_mature_speed_multiplier: 40.0,
          baby_food_consumption_multiplier: 4.0,
          baby_cuddle_loss_multiplier: 0.07,
          baby_cuddle_interval_multiplier: 0.5,
          baby_cuddle_grace_period_multiplier: 1.0,
          baby_imprint_stat_scale_multiplier: 1.0,
          egg_hatch_speed_multiplier: 20.0,
          poops_interval_multiplier: 1.0,
          lay_egg_interval_multiplier: 5.04,
          mating_interval_multiplier: 2.98,
          crafting_skill_bonus_multiplier: 1.0,
          crafting_speed_multiplier: 3.0,
        },
        xp_multipliers: {
          generic_xp_multiplier: 1.0,
          kill_xp_multiplier: 1.0,
          harvest_xp_multiplier: 1.0,
          craft_xp_multiplier: 1.0,
          special_xp_multiplier: 1.0,
          explorer_note_xp_multiplier: 1.0,
          boss_kill_xp_multiplier: 1.0,
          alpha_kill_xp_multiplier: 1.0,
          wild_kill_xp_multiplier: 1.0,
          cave_kill_xp_multiplier: 1.0,
          tamed_kill_xp_multiplier: 1.0,
        },
        dino_wild_stats: {
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
        },
        dino_tamed_stats: {
          per_level: {
            health: 0.2,
            stamina: 0.1,
            oxygen: 0.1,
            food: 0.1,
            water: 0.1,
            weight: 0.1,
            melee_damage: 0.1,
            speed: 0.1,
            fortitude: 0.1,
            torpidity: 0.1,
          },
          add_per_level: {
            health: 0.1,
            stamina: 0.05,
            oxygen: 0.05,
            food: 0.05,
            water: 0.05,
            weight: 0.05,
            melee_damage: 0.05,
            speed: 0.05,
            fortitude: 0.05,
            torpidity: 0.05,
          },
          affinity: {
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
          },
        },
        player_stats: {
          health: 1.0,
          stamina: 1.0,
          oxygen: 1.0,
          food: 1.0,
          water: 1.0,
          weight: 1.0,
          melee_damage: 1.0,
          speed: 1.0,
          fortitude: 1.0,
          crafting_speed: 1.0,
          torpidity: 1.0,
        },
        mods: {
          active_mods: [],
          mod_config: {},
        },
        paths: {
          steam_cmd_dir: 'C:\\ASA\\steamcmd',
          server_dir: 'C:\\ASA\\server',
          backup_dir: 'C:\\ASA\\backups',
          game_ini_path: 'C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\Game.ini',
          gamesettings_ini_path: 'C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\GameUserSettings.ini',
        },
        performance: {
          max_structure_in_range: 10500,
          structure_prevention_radius: 1000.0,
          use_optimization: true,
          enable_debug_logging: false,
        },
        world: {
          day_cycle_speed_scale: 0.5,
          night_time_speed_scale: 1.46,
          day_time_speed_scale: 0.5,
          overall_damage_multiplier: 1.0,
          player_character_health_multiplier: 3.0,
          dino_character_health_multiplier: 1.0,
          global_spoiling_time_multiplier: 1.0,
          global_item_decomposition_time_multiplier: 1.0,
          global_corpse_decomposition_time_multiplier: 1.0,
          resource_no_replenish_radius_players: 1.0,
          resource_no_replenish_radius_structures: 1.0,
          resource_respawn_period_multiplier: 1.0,
          crop_growth_speed_multiplier: 1.0,
          crop_decay_speed_multiplier: 1.0,
          fuel_consumption_interval_multiplier: 1.0,
          force_reset_wild_dinos: false,
        },
        pve: {
          allow_cave_building: false,
          disable_structure_decay_pve: false,
          structure_decay_period_multiplier: 1.0,
          disable_dino_decay_pve: false,
          dino_decay_period_multiplier: 1.0,
          force_allow_cave_flyers: false,
          allow_flyer_carry: true,
          extra_structure_prevention_volumes: false,
          prevent_diseases: false,
          non_permanent_diseases: false,
          prevent_tribe_alliances: false,
          pve_allow_tribe_war: false,
          pve_allow_tribe_war_cancel: false,
        },
        pvp: {
          pvp_dino_decay: false,
          override_structure_platform_prevention: false,
          increase_pvp_respawn_interval: false,
          increase_pvp_respawn_interval_check_period: 0,
          increase_pvp_respawn_interval_multiplier: 1.0,
          increase_pvp_respawn_interval_base_amount: 0,
          pvp_zone_structure_damage_multiplier: 1.0,
          structure_damage_repair_cooldown: 0,
        },
        advanced: {
          allow_unlimited_respecs: true,
          allow_flyer_carry: true,
          allow_cryo_sick_pve: false,
          disable_structure_decay: false,
          enable_cave_flyers: true,
          no_survivor_downloads: false,
          no_dino_downloads: false,
          no_item_downloads: false,
          disable_dino_riding: false,
          disable_dino_taming: false,
          disable_default_dino_taming: false,
          max_tamed_dinos: 5000,
          allow_raid_dino_feeding: false,
          passive_defenses_damage_riderless_dinos: false,
          allow_platform_saddle_multi_floors: false,
          disable_photo_mode: false,
          photo_mode_range_limit: 16000,
          allow_custom_recipes: false,
          custom_recipe_effectiveness_multiplier: 1.0,
          custom_recipe_skill_multiplier: 1.0,
          supply_crate_loot_quality_multiplier: 1.0,
          fishing_loot_quality_multiplier: 1.0,
          platform_structure_limit: 128,
          force_gacha_unhappy_in_caves: false,
          disable_friendly_fire: false,
          disable_structure_placement_collision: false,
          only_allow_specific_engrams: false,
          auto_unlock_engrams: [],
          custom_config: {},
        },
      },
      server_status: {
        running: false,
        process_id: null,
        uptime_seconds: 0,
      },
      get_server_logs: ['[INFO] Mock server started', '[WARN] Mock warning'],
      get_server_metrics: { cpu: 5, memory: 256, fps: 60 },
    }

    const response = mocks[command]
    if (response) {
      logger.info(`Mock response for ${command}`, response)
      return response
    }

    logger.warn(`No mock implementation for command: ${command}`)
    return { mock: true, command }
  }
}

/**
 * Invoke a Tauri command
 */
export async function invoke(command: string, args?: any): Promise<any> {
  if (!invokeFunction) {
    throw new Error('Tauri not initialized. Call initializeTauri() first.')
  }

  logger.debug(`Invoking command: ${command}`, args)

  try {
    const result = await invokeFunction(command, args)
    logger.info(`Command ${command} succeeded`, result)
    return result
  } catch (error) {
    logger.error(`Command ${command} failed`, error)
    throw error
  }
}

/**
 * Check if running in Tauri
 */
export function isTauri(): boolean {
  return isTauriAvailable
}

/**
 * Get Tauri status for debugging
 */
export function getTauriStatus() {
  return {
    available: isTauriAvailable,
    initialized: invokeFunction !== null,
    mode: isTauriAvailable ? 'production' : 'development',
  }
}
