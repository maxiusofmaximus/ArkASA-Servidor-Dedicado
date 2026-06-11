# Tauri API Documentation

This document describes all available Tauri commands for managing ARK server configuration.

## Configuration Commands

### `load_config`

Load configuration from disk.

**Parameters:**
- `config_path: string` - Path to config.toml file

**Returns:** `ServerConfig`

**Example:**
```typescript
const config = await invoke('load_config', { configPath: 'config.toml' })
```

---

### `validate_config`

Validate server configuration against all rules.

**Parameters:**
- `config: ServerConfig` - Configuration object to validate

**Returns:** 
```json
{
  "valid": boolean,
  "errors": [
    {
      "validator": string,
      "message": string
    }
  ]
}
```

**Example:**
```typescript
const result = await invoke('validate_config', { config })
if (!result.valid) {
  result.errors.forEach(error => {
    console.error(`${error.validator}: ${error.message}`)
  })
}
```

---

### `save_config`

Validate and save configuration to disk. Also generates INI files.

**Parameters:**
- `config: ServerConfig` - Configuration to save
- `config_path: string` - Path to save to

**Returns:** `void` (throws on error)

**Side Effects:**
- Saves to `config.toml`
- Generates `Game.ini`
- Generates `GameUserSettings.ini`
- Logs to SQLite audit table

**Example:**
```typescript
await invoke('save_config', { 
  config: myConfig, 
  configPath: 'config.toml' 
})
```

---

### `get_default_config`

Get a fresh ServerConfig with all default values.

**Parameters:** None

**Returns:** `ServerConfig`

**Example:**
```typescript
const defaults = await invoke('get_default_config')
```

---

### `get_config_schema`

Get the JSON schema for configuration validation.

**Parameters:** None

**Returns:** JSON schema object

**Example:**
```typescript
const schema = await invoke('get_config_schema')
```

---

## Server Management Commands

### `server_status`

Get current server status (running, uptime, etc).

**Parameters:** None

**Returns:**
```json
{
  "running": boolean,
  "process_id": number | null,
  "uptime_seconds": number
}
```

**Example:**
```typescript
const status = await invoke('server_status')
if (status.running) {
  console.log(`Server uptime: ${status.uptime_seconds}s`)
}
```

---

### `start_server`

Start ARK server with given configuration.

**Parameters:**
- `config: ServerConfig` - Server configuration

**Returns:** `string` - Success message with PID

**Throws:**
- If configuration is invalid
- If server executable not found
- If process fails to start

**Example:**
```typescript
try {
  const msg = await invoke('start_server', { config })
  console.log(msg) // "Server started with PID: 12345"
} catch (err) {
  console.error(`Failed to start server: ${err}`)
}
```

---

### `stop_server`

Stop the running ARK server.

**Parameters:** None

**Returns:** `string` - Success message

**Throws:**
- If no server is running
- If process termination fails

**Example:**
```typescript
try {
  await invoke('stop_server')
  console.log('Server stopped')
} catch (err) {
  console.error(`Failed to stop server: ${err}`)
}
```

---

### `restart_server`

Restart the server gracefully.

**Parameters:**
- `config: ServerConfig` - Current server configuration

**Returns:** `string` - Success message

**Process:**
1. Stop current server (with 3s grace period)
2. Start new server with config

**Example:**
```typescript
await invoke('restart_server', { config })
```

---

### `check_installation`

Check if ARK server is installed.

**Parameters:**
- `steam_cmd_dir: string` - Path to SteamCMD installation
- `server_dir: string` - Path where server should be installed

**Returns:** `boolean` - True if installed

**Example:**
```typescript
const installed = await invoke('check_installation', {
  steamCmdDir: 'C:\\steamcmd',
  serverDir: 'C:\\asa_server'
})
```

---

### `install_server`

Install ARK server using SteamCMD.

**Parameters:**
- `steam_cmd_dir: string` - Path to SteamCMD
- `server_dir: string` - Where to install

**Returns:** `string` - Success message

**Throws:**
- If SteamCMD not found
- If download fails
- If installation fails

**Note:** This is a long-running operation and may take 30+ minutes.

**Example:**
```typescript
try {
  const msg = await invoke('install_server', {
    steamCmdDir: 'C:\\steamcmd',
    serverDir: 'C:\\asa_server'
  })
  console.log(msg)
} catch (err) {
  console.error(`Installation failed: ${err}`)
}
```

---

## Type Definitions

### ServerConfig

```typescript
interface ServerConfig {
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
```

### IdentificationConfig

```typescript
interface IdentificationConfig {
  session_name: string              // Server name shown in browser
  server_password: string           // Password to join (optional)
  admin_password: string            // Admin/RCON password (REQUIRED)
  server_message_of_the_day: string // MOTD shown on join
}
```

### NetworkConfig

```typescript
interface NetworkConfig {
  port: number              // Main game port (default: 7777)
  query_port: number        // Query port (default: 27015)
  rcon_port: number         // RCON port (default: 27020)
  server_platform: string   // "ALL", "WIN", or "LINUX"
}
```

### GameplayConfig

```typescript
interface GameplayConfig {
  server_pve: boolean              // PvE vs PvP
  max_players: number              // 1-1000
  difficulty_offset: number        // Creature difficulty (0-∞)
  dino_count_multiplier: number    // Spawn rate
  enable_pvp_gamma_bypass: boolean
  allow_third_person_player: boolean
  allow_cryopod_nerf_removal: boolean
}
```

### MultipliersConfig

```typescript
interface MultipliersConfig {
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
```

### ModsConfig

```typescript
interface ModsConfig {
  active_mods: string[]              // Array of mod IDs
  mod_config: Record<string, string> // Custom mod settings
}
```

### PathsConfig

```typescript
interface PathsConfig {
  steam_cmd_dir: string        // C:\steamcmd
  server_dir: string           // C:\ASA\server
  backup_dir: string           // C:\ASA\backups
  game_ini_path: string        // Path to Game.ini
  gamesettings_ini_path: string // Path to GameUserSettings.ini
}
```

### PerformanceConfig

```typescript
interface PerformanceConfig {
  max_structure_in_range: number        // Max structures per area
  structure_prevention_radius: number   // Radius in UE units
  use_optimization: boolean
  enable_debug_logging: boolean
}
```

### WorldConfig

```typescript
interface WorldConfig {
  day_cycle_speed_scale: number          // Time scale for day
  night_time_speed_scale: number         // Time scale for night
  day_time_speed_scale: number           // Unused, included for completeness
  overall_damage_multiplier: number
  player_character_health_multiplier: number
  dino_character_health_multiplier: number
}
```

### AdvancedConfig

```typescript
interface AdvancedConfig {
  allow_unlimited_respecs: boolean
  allow_flyer_carry: boolean
  allow_cryo_sick_pve: boolean
  disable_structure_decay: boolean
  enable_cave_flyers: boolean
  no_survivor_downloads: boolean
  no_dino_downloads: boolean
  no_item_downloads: boolean
  custom_config: Record<string, string> // For extensibility
}
```

---

## Error Handling

All commands throw typed errors with the following structure:

```typescript
interface ErrorResponse {
  field?: string          // Which field has the error (if validation error)
  message: string         // Human-readable error message
  code: string           // Machine-readable error code
}
```

**Common Error Codes:**
- `INVALID_PORT` - Port number out of range
- `INVALID_MOD_ID_ZERO` - Mod ID cannot be 0
- `EMPTY_MOD_ID` - Empty mod IDs are not allowed
- `DEFAULT_PASSWORD` - Using default password
- `SERVER_NOT_FOUND` - Server executable not found
- `VALIDATION_ERROR` - Generic validation failure

---

## Best Practices

### 1. Always Validate Before Saving

```typescript
const result = await invoke('validate_config', { config })
if (!result.valid) {
  // Show errors to user
  return
}
await invoke('save_config', { config, configPath })
```

### 2. Check Server Status Before Starting

```typescript
const status = await invoke('server_status')
if (status.running) {
  console.warn('Server already running')
  return
}
await invoke('start_server', { config })
```

### 3. Poll Status with Debouncing

```typescript
let lastCheck = 0
async function checkStatus() {
  if (Date.now() - lastCheck < 1000) return // Debounce to 1s
  lastCheck = Date.now()
  const status = await invoke('server_status')
  // Update UI
}
```

### 4. Handle Long Operations

```typescript
// Installation can take 30+ minutes
invoke('install_server', { steamCmdDir, serverDir })
  .then(() => showSuccessModal())
  .catch(err => showErrorModal(err.message))

// Don't await the result - show loading spinner instead
```

---

**Last Updated:** 2026-06-10  
**API Version:** 1.0
