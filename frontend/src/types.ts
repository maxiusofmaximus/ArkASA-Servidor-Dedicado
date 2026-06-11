export type Tab = 'general' | 'gameplay' | 'network' | 'advanced' | 'status'

export interface IdentificationSettings {
  session_name: string
  admin_password: string
  server_map: string
}

export interface NetworkSettings {
  port: number
  query_port: number
  rcon_port: number
}

export interface GameplaySettings {
  max_players: number
  difficulty: number
}

export interface MultiplierSettings {
  xp_multiplier: number
  harvest_multiplier: number
  taming_speed_multiplier: number
  breeding_speed_multiplier: number
}

export interface PathSettings {
  saved_dir: string
  logs_dir: string
}

export interface ServerConfig {
  identification: IdentificationSettings
  network: NetworkSettings
  gameplay: GameplaySettings
  multipliers: MultiplierSettings
  paths: PathSettings
}
