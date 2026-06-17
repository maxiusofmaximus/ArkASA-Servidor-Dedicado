import type { ServerConfig } from '../types'

export interface DiffEntry {
  path: string
  label: string
  category: string
  oldValue: unknown
  newValue: unknown
  sensitive: boolean
}

// ─── Field metadata ───────────────────────────────────────────────────────────

const SENSITIVE_PATHS = new Set([
  'identification.server_password',
  'identification.admin_password',
])

// Paths excluded from diff (machine-specific or internal)
const EXCLUDED_PREFIXES = ['paths.', 'advanced.custom_config.', 'mods.mod_config.']

const FIELD_META: Record<string, { label: string; category: string }> = {
  // Identification
  'identification.session_name':              { label: 'Nombre del servidor',          category: 'Identificación' },
  'identification.server_password':           { label: 'Contraseña del servidor',      category: 'Identificación' },
  'identification.admin_password':            { label: 'Contraseña admin',             category: 'Identificación' },
  'identification.server_message_of_the_day':{ label: 'Mensaje MOTD',                  category: 'Identificación' },
  // Network
  'network.port':                             { label: 'Puerto de juego',              category: 'Red' },
  'network.query_port':                       { label: 'Puerto de consulta',           category: 'Red' },
  'network.rcon_port':                        { label: 'Puerto RCON',                  category: 'Red' },
  'network.server_platform':                  { label: 'Plataforma',                   category: 'Red' },
  'network.connection_method':                { label: 'Método de conexión',           category: 'Red' },
  'network.tailscale_ip':                     { label: 'IP Tailscale',                 category: 'Red' },
  'network.public_ip':                        { label: 'IP pública',                   category: 'Red' },
  'network.duckdns_host':                     { label: 'Host DuckDNS',                 category: 'Red' },
  'network.local_ip':                         { label: 'IP local',                     category: 'Red' },
  'network.manual_ip':                        { label: 'IP manual',                    category: 'Red' },
  // Gameplay
  'gameplay.max_players':                     { label: 'Jugadores máximos',            category: 'Gameplay' },
  'gameplay.server_pve':                      { label: 'Modo PVE',                     category: 'Gameplay' },
  'gameplay.server_hardcore':                 { label: 'Modo hardcore',                category: 'Gameplay' },
  'gameplay.difficulty_offset':               { label: 'Dificultad',                   category: 'Gameplay' },
  'gameplay.override_official_difficulty':    { label: 'Dificultad oficial',           category: 'Gameplay' },
  'gameplay.dino_count_multiplier':           { label: 'Multiplicador de dinos',       category: 'Gameplay' },
  'gameplay.kick_idle_players_period':        { label: 'Kick inactivos (min)',          category: 'Gameplay' },
  'gameplay.allow_third_person_player':       { label: 'Tercera persona',              category: 'Gameplay' },
  'gameplay.allow_speed_leveling':            { label: 'Nivel de velocidad',           category: 'Gameplay' },
  'gameplay.allow_flyer_speed_leveling':      { label: 'Velocidad de voladores',       category: 'Gameplay' },
  'gameplay.allow_unlimited_respecs':         { label: 'Respecs ilimitados',           category: 'Gameplay' },
  'gameplay.show_floating_damage_text':       { label: 'Daño flotante',                category: 'Gameplay' },
  'gameplay.server_crosshair':                { label: 'Punto de mira',                category: 'Gameplay' },
  'gameplay.global_voice_chat':               { label: 'Chat de voz global',           category: 'Gameplay' },
  'gameplay.admin_logging':                   { label: 'Log de admin',                 category: 'Gameplay' },
  'gameplay.proximity_chat':                  { label: 'Chat de proximidad',           category: 'Gameplay' },
  'gameplay.enable_pvp_gamma_bypass':         { label: 'Gamma en PVP',                 category: 'Gameplay' },
  'gameplay.disable_pvp_gamma':               { label: 'Sin gamma PVP',                category: 'Gameplay' },
  'gameplay.allow_cryopod_nerf_removal':      { label: 'Sin penalti cryopod',          category: 'Gameplay' },
  'gameplay.allow_hit_markers':               { label: 'Marcadores de golpe',          category: 'Gameplay' },
  'gameplay.force_no_hud':                    { label: 'Sin HUD',                      category: 'Gameplay' },
  // Multipliers
  'multipliers.xp_multiplier':                          { label: 'XP',                         category: 'Multiplicadores' },
  'multipliers.taming_speed_multiplier':                { label: 'Velocidad de doma',           category: 'Multiplicadores' },
  'multipliers.harvest_amount_multiplier':              { label: 'Cantidad de cosecha',         category: 'Multiplicadores' },
  'multipliers.harvest_health_multiplier':              { label: 'Vida de recursos',            category: 'Multiplicadores' },
  'multipliers.player_damage_multiplier':               { label: 'Daño del jugador',            category: 'Multiplicadores' },
  'multipliers.player_resistance_multiplier':           { label: 'Resistencia del jugador',     category: 'Multiplicadores' },
  'multipliers.player_character_food_drain_multiplier': { label: 'Consumo de comida',           category: 'Multiplicadores' },
  'multipliers.player_character_water_drain_multiplier':{ label: 'Consumo de agua',             category: 'Multiplicadores' },
  'multipliers.player_character_stamina_drain_multiplier':{ label: 'Consumo de stamina',        category: 'Multiplicadores' },
  'multipliers.player_character_health_recovery_multiplier':{ label: 'Recuperación de vida',    category: 'Multiplicadores' },
  'multipliers.dino_damage_multiplier':                 { label: 'Daño de dinos',               category: 'Multiplicadores' },
  'multipliers.dino_resistance_multiplier':             { label: 'Resistencia de dinos',        category: 'Multiplicadores' },
  'multipliers.dino_character_health_multiplier':       { label: 'Vida de dinos',               category: 'Multiplicadores' },
  'multipliers.dino_character_food_drain_multiplier':   { label: 'Comida de dinos',             category: 'Multiplicadores' },
  'multipliers.dino_character_stamina_drain_multiplier':{ label: 'Stamina de dinos',            category: 'Multiplicadores' },
  'multipliers.structure_damage_multiplier':            { label: 'Daño a estructuras',          category: 'Multiplicadores' },
  'multipliers.structure_resistance_multiplier':        { label: 'Resistencia de estructuras',  category: 'Multiplicadores' },
  'multipliers.baby_mature_speed_multiplier':           { label: 'Velocidad de maduración',     category: 'Multiplicadores' },
  'multipliers.baby_food_consumption_multiplier':       { label: 'Comida de cría',              category: 'Multiplicadores' },
  'multipliers.baby_cuddle_loss_multiplier':            { label: 'Pérdida de mimitos',          category: 'Multiplicadores' },
  'multipliers.baby_cuddle_interval_multiplier':        { label: 'Intervalo de mimitos',        category: 'Multiplicadores' },
  'multipliers.baby_imprint_stat_scale_multiplier':     { label: 'Imprinting (stats)',          category: 'Multiplicadores' },
  'multipliers.egg_hatch_speed_multiplier':             { label: 'Velocidad de eclosión',       category: 'Multiplicadores' },
  'multipliers.mating_interval_multiplier':             { label: 'Intervalo de cría',           category: 'Multiplicadores' },
  'multipliers.crafting_speed_multiplier':              { label: 'Velocidad de crafteo',        category: 'Multiplicadores' },
  'multipliers.poops_interval_multiplier':              { label: 'Intervalo de excrementos',    category: 'Multiplicadores' },
  'multipliers.lay_egg_interval_multiplier':            { label: 'Intervalo de huevos',         category: 'Multiplicadores' },
  'multipliers.crafting_skill_bonus_multiplier':        { label: 'Bonus de crafteo por skill',  category: 'Multiplicadores' },
  // XP multipliers
  'xp_multipliers.generic_xp_multiplier':   { label: 'XP genérico',            category: 'Multiplicadores XP' },
  'xp_multipliers.kill_xp_multiplier':      { label: 'XP por matar',            category: 'Multiplicadores XP' },
  'xp_multipliers.harvest_xp_multiplier':   { label: 'XP por cosecha',          category: 'Multiplicadores XP' },
  'xp_multipliers.craft_xp_multiplier':     { label: 'XP por crafteo',          category: 'Multiplicadores XP' },
  'xp_multipliers.special_xp_multiplier':   { label: 'XP especial',             category: 'Multiplicadores XP' },
  'xp_multipliers.explorer_note_xp_multiplier': { label: 'XP notas explorador', category: 'Multiplicadores XP' },
  'xp_multipliers.boss_kill_xp_multiplier': { label: 'XP por jefes',            category: 'Multiplicadores XP' },
  'xp_multipliers.alpha_kill_xp_multiplier':{ label: 'XP por alfa',             category: 'Multiplicadores XP' },
  'xp_multipliers.wild_kill_xp_multiplier': { label: 'XP por salvajes',         category: 'Multiplicadores XP' },
  'xp_multipliers.cave_kill_xp_multiplier': { label: 'XP en cuevas',            category: 'Multiplicadores XP' },
  'xp_multipliers.tamed_kill_xp_multiplier':{ label: 'XP por domados',          category: 'Multiplicadores XP' },
  // World
  'world.day_cycle_speed_scale':                      { label: 'Velocidad del ciclo día',      category: 'Mundo' },
  'world.night_time_speed_scale':                     { label: 'Velocidad de la noche',        category: 'Mundo' },
  'world.day_time_speed_scale':                       { label: 'Velocidad del día',            category: 'Mundo' },
  'world.overall_damage_multiplier':                  { label: 'Daño general',                 category: 'Mundo' },
  'world.player_character_health_multiplier':         { label: 'Vida del jugador',             category: 'Mundo' },
  'world.dino_character_health_multiplier':           { label: 'Vida de dinos (mundo)',        category: 'Mundo' },
  'world.global_spoiling_time_multiplier':            { label: 'Tiempo de descomposición',     category: 'Mundo' },
  'world.global_item_decomposition_time_multiplier':  { label: 'Descomposición de items',      category: 'Mundo' },
  'world.global_corpse_decomposition_time_multiplier':{ label: 'Descomposición de cadáveres',  category: 'Mundo' },
  'world.resource_no_replenish_radius_players':       { label: 'Radio sin recursos (jugador)', category: 'Mundo' },
  'world.resource_no_replenish_radius_structures':    { label: 'Radio sin recursos (estructura)', category: 'Mundo' },
  'world.resource_respawn_period_multiplier':         { label: 'Reaparición de recursos',      category: 'Mundo' },
  'world.crop_growth_speed_multiplier':               { label: 'Crecimiento de cultivos',      category: 'Mundo' },
  'world.crop_decay_speed_multiplier':                { label: 'Deterioro de cultivos',        category: 'Mundo' },
  'world.fuel_consumption_interval_multiplier':       { label: 'Intervalo de consumo de combustible', category: 'Mundo' },
  'world.force_reset_wild_dinos':                     { label: 'Resetear dinos salvajes',      category: 'Mundo' },
  // Performance
  'performance.max_structure_in_range':       { label: 'Estructuras máximas en rango', category: 'Rendimiento' },
  'performance.structure_prevention_radius':  { label: 'Radio de prevención',          category: 'Rendimiento' },
  'performance.use_optimization':             { label: 'Optimización activa',          category: 'Rendimiento' },
  'performance.enable_debug_logging':         { label: 'Log de depuración',            category: 'Rendimiento' },
  // Mods
  'mods.active_mods': { label: 'Mods activos', category: 'Mods' },
  // Cluster
  'cluster_maps': { label: 'Mapas del clúster', category: 'Clúster' },
  // PVE
  'pve.allow_cave_building':                 { label: 'Construir en cuevas',         category: 'PVE' },
  'pve.disable_structure_decay_pve':         { label: 'Sin deterioro de estructuras', category: 'PVE' },
  'pve.structure_decay_period_multiplier':   { label: 'Multiplicador deterioro',      category: 'PVE' },
  'pve.disable_dino_decay_pve':              { label: 'Sin deterioro de dinos',       category: 'PVE' },
  'pve.dino_decay_period_multiplier':        { label: 'Deterioro de dinos',           category: 'PVE' },
  'pve.force_allow_cave_flyers':             { label: 'Voladores en cuevas',          category: 'PVE' },
  'pve.allow_flyer_carry':                   { label: 'Llevar jugadores (voladores)', category: 'PVE' },
  'pve.extra_structure_prevention_volumes':  { label: 'Volúmenes extra de prevención', category: 'PVE' },
  // PVP
  'pvp.pvp_dino_decay':                                    { label: 'Deterioro de dinos en PVP', category: 'PVP' },
  'pvp.override_structure_platform_prevention':            { label: 'Sin prevención en plataformas', category: 'PVP' },
  'pvp.increase_pvp_respawn_interval':                     { label: 'Penalización de respawn', category: 'PVP' },
  'pvp.increase_pvp_respawn_interval_multiplier':          { label: 'Mult. penalización respawn', category: 'PVP' },
  'pvp.pvp_zone_structure_damage_multiplier':              { label: 'Daño en zona PVP', category: 'PVP' },
  'pvp.structure_damage_repair_cooldown':                  { label: 'Cooldown de reparación', category: 'PVP' },
  // Advanced
  'advanced.max_tamed_dinos':                       { label: 'Máx. dinos domados',        category: 'Avanzado' },
  'advanced.allow_custom_recipes':                  { label: 'Recetas personalizadas',     category: 'Avanzado' },
  'advanced.custom_recipe_effectiveness_multiplier':{ label: 'Efect. de recetas',          category: 'Avanzado' },
  'advanced.supply_crate_loot_quality_multiplier':  { label: 'Calidad de botín',           category: 'Avanzado' },
  'advanced.fishing_loot_quality_multiplier':       { label: 'Calidad de pesca',           category: 'Avanzado' },
  'advanced.disable_friendly_fire':                 { label: 'Sin fuego amigo',            category: 'Avanzado' },
  'advanced.only_allow_specific_engrams':           { label: 'Solo engrams específicos',   category: 'Avanzado' },
  'advanced.disable_dino_riding':                   { label: 'Sin montura de dinos',       category: 'Avanzado' },
  'advanced.disable_dino_taming':                   { label: 'Sin domado de dinos',        category: 'Avanzado' },
  'advanced.platform_structure_limit':              { label: 'Límite de estructuras en plataforma', category: 'Avanzado' },
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function isExcluded(path: string): boolean {
  return EXCLUDED_PREFIXES.some((pfx) => path.startsWith(pfx))
}

function labelOf(path: string): { label: string; category: string } {
  if (FIELD_META[path]) return FIELD_META[path]
  const parts = path.split('.')
  const section = parts[0]
  const field = parts[parts.length - 1]
  const label = field.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
  const category = section.charAt(0).toUpperCase() + section.slice(1)
  return { label, category }
}

type FlatMap = Record<string, unknown>

function flatten(obj: unknown, prefix = ''): FlatMap {
  if (obj === null || obj === undefined) return {}
  if (typeof obj !== 'object' || Array.isArray(obj)) {
    return prefix ? { [prefix]: obj } : {}
  }
  const result: FlatMap = {}
  for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
    const key = prefix ? `${prefix}.${k}` : k
    if (Array.isArray(v)) {
      result[key] = v
    } else if (v !== null && typeof v === 'object') {
      Object.assign(result, flatten(v, key))
    } else {
      result[key] = v
    }
  }
  return result
}

// ─── Public API ───────────────────────────────────────────────────────────────

export function computeDiff(before: ServerConfig, after: ServerConfig): DiffEntry[] {
  const a = flatten(before)
  const b = flatten(after)

  const allKeys = new Set([...Object.keys(a), ...Object.keys(b)])
  const diffs: DiffEntry[] = []

  for (const path of allKeys) {
    if (isExcluded(path)) continue
    const aVal = a[path]
    const bVal = b[path]
    if (JSON.stringify(aVal) === JSON.stringify(bVal)) continue
    const meta = labelOf(path)
    diffs.push({
      path,
      label: meta.label,
      category: meta.category,
      oldValue: aVal,
      newValue: bVal,
      sensitive: SENSITIVE_PATHS.has(path),
    })
  }

  // Sort by category order, then label
  const CATEGORY_ORDER = [
    'Identificación', 'Red', 'Gameplay', 'Multiplicadores', 'Multiplicadores XP',
    'Mundo', 'Rendimiento', 'Mods', 'Clúster', 'PVE', 'PVP', 'Avanzado',
  ]
  diffs.sort((a, b) => {
    const ai = CATEGORY_ORDER.indexOf(a.category)
    const bi = CATEGORY_ORDER.indexOf(b.category)
    if (ai !== bi) return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi)
    return a.label.localeCompare(b.label)
  })

  return diffs
}

export function formatValue(val: unknown, sensitive = false): string {
  if (sensitive) return '•••'
  if (val === null || val === undefined) return '(vacío)'
  if (typeof val === 'boolean') return val ? 'Sí' : 'No'
  if (Array.isArray(val)) {
    if (val.length === 0) return '(ninguno)'
    if (val.length <= 4) return val.join(', ')
    return `${val.length} elementos`
  }
  if (typeof val === 'number') return String(val)
  if (typeof val === 'string') return val === '' ? '(vacío)' : val
  return JSON.stringify(val)
}
