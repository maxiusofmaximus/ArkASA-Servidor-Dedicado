export type Tab = 'general' | 'gameplay' | 'server' | 'advanced' | 'status'

export interface IdentificationConfig {
  session_name: string
  server_password: string
  admin_password: string
  server_message_of_the_day: string
}

export interface NetworkConfig {
  port: number
  query_port: number
  rcon_port: number
  server_platform: string
}

export interface GameplayConfig {
  server_pve: boolean
  max_players: number
  difficulty_offset: number
  dino_count_multiplier: number
  enable_pvp_gamma_bypass: boolean
  allow_third_person_player: boolean
  allow_cryopod_nerf_removal: boolean
}

export interface MultipliersConfig {
  xp_multiplier: number
  taming_speed_multiplier: number
  harvest_amount_multiplier: number
  harvest_health_multiplier: number
  baby_mature_speed_multiplier: number
  baby_food_consumption_multiplier: number
  baby_cuddle_loss_multiplier: number
  egg_hatch_speed_multiplier: number
  poops_interval_multiplier: number
  lay_egg_interval_multiplier: number
  mating_interval_multiplier: number
  crafting_skill_bonus_multiplier: number
  crafting_speed_multiplier: number
}

export interface ModsConfig {
  active_mods: string[]
  mod_config: Record<string, string>
}

export interface PathsConfig {
  steam_cmd_dir: string
  server_dir: string
  backup_dir: string
  game_ini_path: string
  gamesettings_ini_path: string
}

export interface PerformanceConfig {
  max_structure_in_range: number
  structure_prevention_radius: number
  use_optimization: boolean
  enable_debug_logging: boolean
}

export interface WorldConfig {
  day_cycle_speed_scale: number
  night_time_speed_scale: number
  day_time_speed_scale: number
  overall_damage_multiplier: number
  player_character_health_multiplier: number
  dino_character_health_multiplier: number
}

export interface AdvancedConfig {
  allow_unlimited_respecs: boolean
  allow_flyer_carry: boolean
  allow_cryo_sick_pve: boolean
  disable_structure_decay: boolean
  enable_cave_flyers: boolean
  no_survivor_downloads: boolean
  no_dino_downloads: boolean
  no_item_downloads: boolean
  custom_config: Record<string, string>
}

export interface ServerConfig {
  identification: IdentificationConfig
  network: NetworkConfig
  gameplay: GameplayConfig
  multipliers: MultipliersConfig
  mods: ModsConfig
  paths: PathsConfig
  performance: PerformanceConfig
  world: WorldConfig
  advanced: AdvancedConfig
}

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
