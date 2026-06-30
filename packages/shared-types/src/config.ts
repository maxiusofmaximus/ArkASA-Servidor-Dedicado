/**
 * Server configuration types shared between the Tauri desktop app and the
 * Vercel-hosted admin web. Mirrors `src-tauri/src/config/schema.rs`.
 *
 * IMPORTANT: when adding a new field, add it to BOTH places to avoid drift.
 */

// Primary and sub-tab type unions
export type PrimaryTab = 'arks' | 'mod_settings' | 'game_rules' | 'advanced' | 'engrams'
export type GameRulesSubTab = 'player' | 'creature' | 'structure' | 'world' | 'rules'
export type AdvancedSubTab = 'pve' | 'pvp' | 'world' | 'wild_dino' | 'tamed_dino' | 'player' | 'xp_multipliers' | 'misc'
export type ModSettingsSubTab = 'active_mods' | 'available_mods'

// Legacy alias for compatibility
export type Tab = PrimaryTab

// Stat multipliers for dinos (10 stats: health, stamina, oxygen, food, water, weight, melee_damage, speed, fortitude, torpidity)
export interface DinoStatsPerLevel {
  health: number
  stamina: number
  oxygen: number
  food: number
  water: number
  weight: number
  melee_damage: number
  speed: number
  fortitude: number
  torpidity: number
}

// Player stats (11 stats: same as dino + crafting_speed at index 9)
export interface PlayerStatsPerLevel {
  health: number
  stamina: number
  oxygen: number
  food: number
  water: number
  weight: number
  melee_damage: number
  speed: number
  fortitude: number
  crafting_speed: number
  torpidity: number
}

// Tamed dino stats: three tables (per_level, add_per_level, affinity)
export interface DinoTamedStatsConfig {
  per_level: DinoStatsPerLevel
  add_per_level: DinoStatsPerLevel
  affinity: DinoStatsPerLevel
}

// XP multipliers (11 types)
export interface XpMultipliersConfig {
  generic_xp_multiplier: number
  kill_xp_multiplier: number
  harvest_xp_multiplier: number
  craft_xp_multiplier: number
  special_xp_multiplier: number
  explorer_note_xp_multiplier: number
  boss_kill_xp_multiplier: number
  alpha_kill_xp_multiplier: number
  wild_kill_xp_multiplier: number
  cave_kill_xp_multiplier: number
  tamed_kill_xp_multiplier: number
}

// Identification settings
export interface IdentificationConfig {
  session_name: string
  server_password: string
  admin_password: string
  server_message_of_the_day: string
}

// Connection type for dynamic connection entries
export type ConnectionType = 'tailscale' | 'public_ip' | 'duckdns' | 'local_ip' | 'manual' | 'playit_tunnel' | 'custom'

// Connection method for the server IP selector (legacy)
export type ConnectionMethod = 'tailscale' | 'public' | 'duckdns' | 'local' | 'manual'

// A connection entry in the dynamic connection list
export interface ConnectionEntry {
  id: string
  conn_type: ConnectionType
  label: string
  address: string
  is_primary: boolean
  tunnel_port?: number | null
}

// Network settings
export interface NetworkConfig {
  port: number
  query_port: number
  rcon_port: number
  server_platform: string
  // Network / launch behavior flags (v2.1.0)
  /** When true, ARK launches with `-NoBattlEye` (BattleEye **disabled**). */
  no_battleye: boolean
  /** When true, each cluster map gets a stable port triplet derived from its
   *  map id hash. When false, ports are computed by order-of-arrival. */
  fixed_port_assignment_per_map: boolean
  /** When true, the frontend internet-gate is bypassed at start time. */
  allow_start_without_internet: boolean
  // Connection Manager (v2 dynamic list)
  connection_entries: ConnectionEntry[]
  // Legacy — read from old TOML; effective_ip() falls back to them
  connection_method: ConnectionMethod
  tailscale_ip: string
  public_ip: string
  duckdns_host: string
  local_ip: string
  manual_ip: string
  server_ip: string
}

// A saved friend IP contact
export interface FriendContact {
  id: string
  name: string
  ip: string
}

// Result of the detect_ips Tauri command
export interface DetectedIps {
  public_ip:    string | null
  tailscale_ip: string | null
  local_ip:     string | null
}

// Gameplay settings
export interface GameplayConfig {
  server_pve: boolean
  server_hardcore: boolean
  max_players: number
  difficulty_offset: number
  override_official_difficulty: number
  dino_count_multiplier: number
  enable_pvp_gamma_bypass: boolean
  disable_pvp_gamma: boolean
  allow_third_person_player: boolean
  allow_cryopod_nerf_removal: boolean
  allow_speed_leveling: boolean
  allow_flyer_speed_leveling: boolean
  allow_unlimited_respecs: boolean
  show_floating_damage_text: boolean
  allow_hit_markers: boolean
  server_crosshair: boolean
  force_no_hud: boolean
  proximity_chat: boolean
  global_voice_chat: boolean
  admin_logging: boolean
  always_notify_player_left: boolean
  dont_always_notify_player_joined: boolean
  kick_idle_players_period: number
}

// Multipliers
export interface MultipliersConfig {
  xp_multiplier: number
  taming_speed_multiplier: number
  harvest_amount_multiplier: number
  harvest_health_multiplier: number
  player_damage_multiplier: number
  player_resistance_multiplier: number
  player_character_water_drain_multiplier: number
  player_character_food_drain_multiplier: number
  player_character_stamina_drain_multiplier: number
  player_character_health_recovery_multiplier: number
  dino_damage_multiplier: number
  dino_resistance_multiplier: number
  dino_character_health_multiplier: number
  dino_character_food_drain_multiplier: number
  dino_character_stamina_drain_multiplier: number
  structure_damage_multiplier: number
  structure_resistance_multiplier: number
  baby_mature_speed_multiplier: number
  baby_food_consumption_multiplier: number
  baby_cuddle_loss_multiplier: number
  baby_cuddle_interval_multiplier: number
  baby_cuddle_grace_period_multiplier: number
  baby_imprint_stat_scale_multiplier: number
  egg_hatch_speed_multiplier: number
  poops_interval_multiplier: number
  lay_egg_interval_multiplier: number
  mating_interval_multiplier: number
  crafting_skill_bonus_multiplier: number
  crafting_speed_multiplier: number
}

// Mods configuration
export interface ModsConfig {
  active_mods: string[]
  mod_config: Record<string, string>
}

// Paths configuration
export interface PathsConfig {
  steam_cmd_dir: string
  server_dir: string
  backup_dir: string
  game_ini_path: string
  gamesettings_ini_path: string
}

// Performance configuration
export interface PerformanceConfig {
  max_structure_in_range: number
  structure_prevention_radius: number
  use_optimization: boolean
  enable_debug_logging: boolean
}

// World configuration
export interface WorldConfig {
  day_cycle_speed_scale: number
  night_time_speed_scale: number
  day_time_speed_scale: number
  overall_damage_multiplier: number
  player_character_health_multiplier: number
  dino_character_health_multiplier: number
  global_spoiling_time_multiplier: number
  global_item_decomposition_time_multiplier: number
  global_corpse_decomposition_time_multiplier: number
  resource_no_replenish_radius_players: number
  resource_no_replenish_radius_structures: number
  resource_respawn_period_multiplier: number
  crop_growth_speed_multiplier: number
  crop_decay_speed_multiplier: number
  fuel_consumption_interval_multiplier: number
  force_reset_wild_dinos: boolean
}

// PvE configuration
export interface PveConfig {
  allow_cave_building: boolean
  disable_structure_decay_pve: boolean
  structure_decay_period_multiplier: number
  disable_dino_decay_pve: boolean
  dino_decay_period_multiplier: number
  force_allow_cave_flyers: boolean
  allow_flyer_carry: boolean
  extra_structure_prevention_volumes: boolean
  prevent_diseases: boolean
  non_permanent_diseases: boolean
  prevent_tribe_alliances: boolean
  pve_allow_tribe_war: boolean
  pve_allow_tribe_war_cancel: boolean
}

// PvP configuration
export interface PvpConfig {
  pvp_dino_decay: boolean
  override_structure_platform_prevention: boolean
  increase_pvp_respawn_interval: boolean
  increase_pvp_respawn_interval_check_period: number
  increase_pvp_respawn_interval_multiplier: number
  increase_pvp_respawn_interval_base_amount: number
  pvp_zone_structure_damage_multiplier: number
  structure_damage_repair_cooldown: number
}

// Advanced configuration
export interface AdvancedConfig {
  allow_unlimited_respecs: boolean
  allow_flyer_carry: boolean
  allow_cryo_sick_pve: boolean
  disable_structure_decay: boolean
  enable_cave_flyers: boolean
  no_survivor_downloads: boolean
  no_dino_downloads: boolean
  no_item_downloads: boolean
  disable_dino_riding: boolean
  disable_dino_taming: boolean
  disable_default_dino_taming: boolean
  max_tamed_dinos: number
  allow_raid_dino_feeding: boolean
  passive_defenses_damage_riderless_dinos: boolean
  allow_platform_saddle_multi_floors: boolean
  disable_photo_mode: boolean
  photo_mode_range_limit: number
  allow_custom_recipes: boolean
  custom_recipe_effectiveness_multiplier: number
  custom_recipe_skill_multiplier: number
  supply_crate_loot_quality_multiplier: number
  fishing_loot_quality_multiplier: number
  platform_structure_limit: number
  limit_generators_num: number
  limit_generators_range: number
  force_gacha_unhappy_in_caves: boolean
  disable_friendly_fire: boolean
  disable_structure_placement_collision: boolean
  only_allow_specific_engrams: boolean
  auto_unlock_engrams: number[]
  custom_config: Record<string, string>
}

export interface MapInstanceStatus {
  map_index: number
  map_id: string
  map_label: string
  running: boolean
}

// Main server configuration
export interface ServerConfig {
  cluster_maps: string[]
  identification: IdentificationConfig
  network: NetworkConfig
  gameplay: GameplayConfig
  multipliers: MultipliersConfig
  xp_multipliers: XpMultipliersConfig
  dino_wild_stats: DinoStatsPerLevel
  dino_tamed_stats: DinoTamedStatsConfig
  player_stats: PlayerStatsPerLevel
  mods: ModsConfig
  paths: PathsConfig
  performance: PerformanceConfig
  world: WorldConfig
  pve: PveConfig
  pvp: PvpConfig
  advanced: AdvancedConfig
}

// Validation types
export interface ValidationError {
  field: string
  message: string
  code: string
}

export interface ValidationResult {
  valid: boolean
  errors: ValidationError[]
}

export interface ServerStatus {
  running: boolean
  lastUpdate: number
  configValid: boolean
}
