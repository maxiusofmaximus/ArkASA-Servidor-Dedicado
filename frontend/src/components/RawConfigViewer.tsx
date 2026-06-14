import { useState, useMemo } from 'react'
import type { ServerConfig } from '../types'

interface RawConfigViewerProps {
  config: ServerConfig
}

type ConfigTab = 'gameusersettings' | 'game' | 'toml'

// ── GameUserSettings.ini generator ─────────────────────────────────────────
function generateGameUserSettings(c: ServerConfig): string {
  const m = c.multipliers
  const w = c.world
  const g = c.gameplay
  const net = c.network
  const id = c.identification

  return [
    '[ServerSettings]',
    `ServerPassword=${id.server_password}`,
    `AdminPassword=${id.admin_password}`,
    `Port=${net.port}`,
    `QueryPort=${net.query_port}`,
    `RCONPort=${net.rcon_port}`,
    `RCONEnabled=True`,
    `ServerPlatform=${net.server_platform}`,
    '',
    '; ─── Multipliers ───────────────────────────────────────',
    `XPMultiplier=${m.xp_multiplier}`,
    `TamingSpeedMultiplier=${m.taming_speed_multiplier}`,
    `HarvestAmountMultiplier=${m.harvest_amount_multiplier}`,
    `HarvestHealthMultiplier=${m.harvest_health_multiplier}`,
    '',
    '; Player',
    `PlayerDamageMultiplier=${m.player_damage_multiplier}`,
    `PlayerResistanceMultiplier=${m.player_resistance_multiplier}`,
    `PlayerCharacterWaterDrainMultiplier=${m.player_character_water_drain_multiplier}`,
    `PlayerCharacterFoodDrainMultiplier=${m.player_character_food_drain_multiplier}`,
    `PlayerCharacterStaminaDrainMultiplier=${m.player_character_stamina_drain_multiplier}`,
    `PlayerCharacterHealthRecoveryMultiplier=${m.player_character_health_recovery_multiplier}`,
    '',
    '; Dinos',
    `DinoDamageMultiplier=${m.dino_damage_multiplier}`,
    `DinoResistanceMultiplier=${m.dino_resistance_multiplier}`,
    `DinoCharacterHealthMultiplier=${m.dino_character_health_multiplier}`,
    `DinoCharacterFoodDrainMultiplier=${m.dino_character_food_drain_multiplier}`,
    `DinoCharacterStaminaDrainMultiplier=${m.dino_character_stamina_drain_multiplier}`,
    '',
    '; Structures',
    `StructureDamageMultiplier=${m.structure_damage_multiplier}`,
    `StructureResistanceMultiplier=${m.structure_resistance_multiplier}`,
    '',
    '; Breeding',
    `BabyMatureSpeedMultiplier=${m.baby_mature_speed_multiplier}`,
    `BabyFoodConsumptionMultiplier=${m.baby_food_consumption_multiplier}`,
    `BabyCuddleLossMultiplier=${m.baby_cuddle_loss_multiplier}`,
    `BabyCuddleIntervalMultiplier=${m.baby_cuddle_interval_multiplier}`,
    `BabyCuddleGracePeriodMultiplier=${m.baby_cuddle_grace_period_multiplier}`,
    `BabyImprintStatScaleMultiplier=${m.baby_imprint_stat_scale_multiplier}`,
    `EggHatchSpeedMultiplier=${m.egg_hatch_speed_multiplier}`,
    `MatingIntervalMultiplier=${m.mating_interval_multiplier}`,
    `LayEggIntervalMultiplier=${m.lay_egg_interval_multiplier}`,
    `PoopsIntervalMultiplier=${m.poops_interval_multiplier}`,
    '',
    '; Crafting',
    `CraftingSpeedMultiplier=${m.crafting_speed_multiplier}`,
    `CraftingSkillBonusMultiplier=${m.crafting_skill_bonus_multiplier}`,
    '',
    '; World / Environment',
    `DayCycleSpeedScale=${w.day_cycle_speed_scale}`,
    `DayTimeSpeedScale=${w.day_time_speed_scale}`,
    `NightTimeSpeedScale=${w.night_time_speed_scale}`,
    `GlobalSpoilingTimeMultiplier=${w.global_spoiling_time_multiplier}`,
    `GlobalItemDecompositionTimeMultiplier=${w.global_item_decomposition_time_multiplier}`,
    `GlobalCorpseDecompositionTimeMultiplier=${w.global_corpse_decomposition_time_multiplier}`,
    `ResourceNoReplenishRadiusPlayers=${w.resource_no_replenish_radius_players}`,
    `ResourceNoReplenishRadiusStructures=${w.resource_no_replenish_radius_structures}`,
    `ResourceRespawnPeriodMultiplier=${w.resource_respawn_period_multiplier}`,
    `CropGrowthSpeedMultiplier=${w.crop_growth_speed_multiplier}`,
    `CropDecaySpeedMultiplier=${w.crop_decay_speed_multiplier}`,
    `FuelConsumptionIntervalMultiplier=${w.fuel_consumption_interval_multiplier}`,
    '',
    '; Gameplay',
    `KickIdlePlayersPeriod=${g.kick_idle_players_period}`,
    '',
    '[SessionSettings]',
    `SessionName=${id.server_message_of_the_day}`,
    '',
    '[/Script/Engine.GameSession]',
    `MaxPlayers=${g.max_players}`,
  ].join('\n')
}

// ── Game.ini generator ──────────────────────────────────────────────────────
function generateGameIni(c: ServerConfig): string {
  const g = c.gameplay
  const id = c.identification
  const w = c.world
  const bool = (v: boolean) => (v ? 'True' : 'False')

  return [
    '[/Script/ShooterGame.ShooterGameMode]',
    `SessionName=${id.session_name}`,
    `MOTD=${id.server_message_of_the_day}`,
    '',
    '; ─── Difficulty ────────────────────────────────────────',
    `DifficultyOffset=${g.difficulty_offset}`,
    `OverrideOfficialDifficulty=${g.override_official_difficulty}`,
    `DinoCountMultiplier=${g.dino_count_multiplier}`,
    '',
    '; ─── Player behaviour ──────────────────────────────────',
    `bServerPVE=${bool(g.server_pve)}`,
    `bServerHardcore=${bool(g.server_hardcore)}`,
    `bAllowThirdPersonPlayer=${bool(g.allow_third_person_player)}`,
    `bAllowSpeedLeveling=${bool(g.allow_speed_leveling)}`,
    `bAllowFlyerSpeedLeveling=${bool(g.allow_flyer_speed_leveling)}`,
    `bAllowUnlimitedRespecs=${bool(g.allow_unlimited_respecs)}`,
    `bShowFloatingDamageText=${bool(g.show_floating_damage_text)}`,
    `bAllowHitMarkers=${bool(g.allow_hit_markers)}`,
    `bServerCrosshair=${bool(g.server_crosshair)}`,
    `bForceNoHud=${bool(g.force_no_hud)}`,
    `bProximityChat=${bool(g.proximity_chat)}`,
    `bGlobalVoiceChat=${bool(g.global_voice_chat)}`,
    `bAdminLogging=${bool(g.admin_logging)}`,
    `bAlwaysNotifyPlayerLeft=${bool(g.always_notify_player_left)}`,
    `bDontAlwaysNotifyPlayerJoined=${bool(g.dont_always_notify_player_joined)}`,
    '',
    '; ─── PvP ───────────────────────────────────────────────',
    `bEnablePVPGamma=${bool(g.enable_pvp_gamma_bypass)}`,
    `bDisablePvEGamma=${bool(g.disable_pvp_gamma)}`,
    `bAllowCryopodNerf=${bool(g.allow_cryopod_nerf_removal)}`,
    '',
    '; ─── World ─────────────────────────────────────────────',
    `ForceResetWildDinos=${bool(w.force_reset_wild_dinos)}`,
    `OverallDamageMultiplier=${w.overall_damage_multiplier}`,
    '',
    '; ─── Mods ──────────────────────────────────────────────',
    `ActiveMods=${c.mods.active_mods.join(',')}`,
  ].join('\n')
}

// ── config.toml generator ───────────────────────────────────────────────────
function generateToml(c: ServerConfig): string {
  const m = c.multipliers
  const w = c.world
  const g = c.gameplay
  const net = c.network
  const id = c.identification

  return [
    '[identification]',
    `session_name = "${id.session_name}"`,
    `admin_password = "${id.admin_password}"`,
    `server_password = "${id.server_password}"`,
    `server_message_of_the_day = "${id.server_message_of_the_day}"`,
    '',
    '[network]',
    `port = ${net.port}`,
    `query_port = ${net.query_port}`,
    `rcon_port = ${net.rcon_port}`,
    `server_platform = "${net.server_platform}"`,
    '',
    '[gameplay]',
    `server_pve = ${g.server_pve}`,
    `server_hardcore = ${g.server_hardcore}`,
    `max_players = ${g.max_players}`,
    `difficulty_offset = ${g.difficulty_offset}`,
    `override_official_difficulty = ${g.override_official_difficulty}`,
    `dino_count_multiplier = ${g.dino_count_multiplier}`,
    `allow_third_person_player = ${g.allow_third_person_player}`,
    `allow_speed_leveling = ${g.allow_speed_leveling}`,
    `allow_flyer_speed_leveling = ${g.allow_flyer_speed_leveling}`,
    `allow_unlimited_respecs = ${g.allow_unlimited_respecs}`,
    `show_floating_damage_text = ${g.show_floating_damage_text}`,
    `allow_hit_markers = ${g.allow_hit_markers}`,
    `server_crosshair = ${g.server_crosshair}`,
    `force_no_hud = ${g.force_no_hud}`,
    `proximity_chat = ${g.proximity_chat}`,
    `global_voice_chat = ${g.global_voice_chat}`,
    `admin_logging = ${g.admin_logging}`,
    `kick_idle_players_period = ${g.kick_idle_players_period}`,
    '',
    '[multipliers]',
    `xp_multiplier = ${m.xp_multiplier}`,
    `taming_speed_multiplier = ${m.taming_speed_multiplier}`,
    `harvest_amount_multiplier = ${m.harvest_amount_multiplier}`,
    `harvest_health_multiplier = ${m.harvest_health_multiplier}`,
    `player_damage_multiplier = ${m.player_damage_multiplier}`,
    `player_resistance_multiplier = ${m.player_resistance_multiplier}`,
    `player_character_water_drain_multiplier = ${m.player_character_water_drain_multiplier}`,
    `player_character_food_drain_multiplier = ${m.player_character_food_drain_multiplier}`,
    `player_character_stamina_drain_multiplier = ${m.player_character_stamina_drain_multiplier}`,
    `player_character_health_recovery_multiplier = ${m.player_character_health_recovery_multiplier}`,
    `dino_damage_multiplier = ${m.dino_damage_multiplier}`,
    `dino_resistance_multiplier = ${m.dino_resistance_multiplier}`,
    `dino_character_health_multiplier = ${m.dino_character_health_multiplier}`,
    `dino_character_food_drain_multiplier = ${m.dino_character_food_drain_multiplier}`,
    `dino_character_stamina_drain_multiplier = ${m.dino_character_stamina_drain_multiplier}`,
    `structure_damage_multiplier = ${m.structure_damage_multiplier}`,
    `structure_resistance_multiplier = ${m.structure_resistance_multiplier}`,
    `baby_mature_speed_multiplier = ${m.baby_mature_speed_multiplier}`,
    `baby_food_consumption_multiplier = ${m.baby_food_consumption_multiplier}`,
    `baby_cuddle_loss_multiplier = ${m.baby_cuddle_loss_multiplier}`,
    `baby_cuddle_interval_multiplier = ${m.baby_cuddle_interval_multiplier}`,
    `baby_cuddle_grace_period_multiplier = ${m.baby_cuddle_grace_period_multiplier}`,
    `baby_imprint_stat_scale_multiplier = ${m.baby_imprint_stat_scale_multiplier}`,
    `egg_hatch_speed_multiplier = ${m.egg_hatch_speed_multiplier}`,
    `mating_interval_multiplier = ${m.mating_interval_multiplier}`,
    `lay_egg_interval_multiplier = ${m.lay_egg_interval_multiplier}`,
    `poops_interval_multiplier = ${m.poops_interval_multiplier}`,
    `crafting_speed_multiplier = ${m.crafting_speed_multiplier}`,
    `crafting_skill_bonus_multiplier = ${m.crafting_skill_bonus_multiplier}`,
    '',
    '[world]',
    `day_cycle_speed_scale = ${w.day_cycle_speed_scale}`,
    `day_time_speed_scale = ${w.day_time_speed_scale}`,
    `night_time_speed_scale = ${w.night_time_speed_scale}`,
    `global_spoiling_time_multiplier = ${w.global_spoiling_time_multiplier}`,
    `global_item_decomposition_time_multiplier = ${w.global_item_decomposition_time_multiplier}`,
    `global_corpse_decomposition_time_multiplier = ${w.global_corpse_decomposition_time_multiplier}`,
    `resource_no_replenish_radius_players = ${w.resource_no_replenish_radius_players}`,
    `resource_no_replenish_radius_structures = ${w.resource_no_replenish_radius_structures}`,
    `resource_respawn_period_multiplier = ${w.resource_respawn_period_multiplier}`,
    `crop_growth_speed_multiplier = ${w.crop_growth_speed_multiplier}`,
    `crop_decay_speed_multiplier = ${w.crop_decay_speed_multiplier}`,
    `fuel_consumption_interval_multiplier = ${w.fuel_consumption_interval_multiplier}`,
    `force_reset_wild_dinos = ${w.force_reset_wild_dinos}`,
    `overall_damage_multiplier = ${w.overall_damage_multiplier}`,
    '',
    '[mods]',
    `active_mods = [${c.mods.active_mods.map((m) => `"${m}"`).join(', ')}]`,
  ].join('\n')
}

// ── Component ───────────────────────────────────────────────────────────────

export default function RawConfigViewer({ config }: RawConfigViewerProps) {
  const [activeTab, setActiveTab] = useState<ConfigTab>('gameusersettings')
  const [search, setSearch] = useState('')
  const [copied, setCopied] = useState(false)

  const fullContent = useMemo(() => {
    if (activeTab === 'gameusersettings') return generateGameUserSettings(config)
    if (activeTab === 'game') return generateGameIni(config)
    return generateToml(config)
  }, [activeTab, config])

  const q = search.trim().toLowerCase()

  // Split content into lines so we can highlight matching ones
  const lines = useMemo(() => fullContent.split('\n'), [fullContent])

  const matchCount = useMemo(
    () => (q ? lines.filter((l) => l.toLowerCase().includes(q)).length : 0),
    [lines, q]
  )

  const handleCopy = () => {
    navigator.clipboard.writeText(fullContent)
    setCopied(true)
    setTimeout(() => setCopied(false), 1800)
  }

  const TAB_LABELS: Record<ConfigTab, string> = {
    gameusersettings: 'GameUserSettings.ini',
    game: 'Game.ini',
    toml: 'config.toml',
  }

  return (
    <div className="flex flex-col gap-3 h-full">
      {/* Sub-tab bar */}
      <div className="flex gap-1">
        {(['gameusersettings', 'game', 'toml'] as ConfigTab[]).map((t) => (
          <button
            key={t}
            onClick={() => setActiveTab(t)}
            className="px-3 py-1.5 text-[11px] font-bold tracking-wider rounded-md transition-all font-mono"
            style={{
              background: activeTab === t ? 'rgba(0,200,255,0.12)' : 'rgba(255,255,255,0.03)',
              border: `1px solid ${activeTab === t ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.08)'}`,
              color: activeTab === t ? 'rgba(0,200,255,0.9)' : 'rgba(255,255,255,0.35)',
            }}
          >
            {TAB_LABELS[t]}
          </button>
        ))}
      </div>

      {/* Search bar */}
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <span className="absolute left-3 top-1/2 -translate-y-1/2 text-ark-cyan/40 text-xs pointer-events-none">🔍</span>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Buscar… (ej: OverrideOfficialDifficulty, WaterDrain, TamingSpeed)"
            className="w-full bg-transparent border border-ark-cyan/25 text-ark-cyan/90 text-xs pl-8 pr-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/25 font-mono"
          />
          {search && (
            <button
              onClick={() => setSearch('')}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-ark-cyan/40 hover:text-ark-cyan/70 text-sm"
            >×</button>
          )}
        </div>
        {q && (
          <span className="text-ark-cyan/40 text-[11px] font-mono flex-shrink-0">
            {matchCount} {matchCount === 1 ? 'línea' : 'líneas'}
          </span>
        )}
        <button
          onClick={handleCopy}
          className="ark-action-btn text-[10px] px-3 py-1.5 flex-shrink-0"
        >
          {copied ? '✓ Copiado' : '⎘ Copiar'}
        </button>
      </div>

      {/* Content */}
      <div
        className="flex-1 overflow-y-auto font-mono text-[11px] leading-relaxed rounded-lg"
        style={{
          background: 'rgba(0,0,0,0.35)',
          border: '1px solid rgba(0,200,255,0.12)',
          maxHeight: 'calc(90vh - 260px)',
        }}
      >
        <div className="p-3 space-y-0">
          {lines.map((line, i) => {
            const isMatch = q && line.toLowerCase().includes(q)
            const isComment = line.trimStart().startsWith(';') || line.trimStart().startsWith('#')
            const isSection = line.startsWith('[')
            const isEmpty = line.trim() === ''

            let color = 'rgba(180,220,255,0.6)'
            if (isComment) color = 'rgba(100,160,100,0.55)'
            else if (isSection) color = 'rgba(0,200,255,0.85)'
            else if (line.includes('=')) color = 'rgba(180,220,255,0.75)'

            return (
              <div
                key={i}
                className="flex"
                style={{
                  background: isMatch ? 'rgba(0,200,255,0.12)' : 'transparent',
                  borderLeft: isMatch ? '2px solid rgba(0,200,255,0.6)' : '2px solid transparent',
                  paddingLeft: isMatch ? '6px' : '8px',
                }}
              >
                {/* Line number */}
                <span
                  className="select-none flex-shrink-0 text-right pr-3 w-9"
                  style={{ color: 'rgba(100,130,150,0.4)', fontSize: '10px', lineHeight: '1.6' }}
                >
                  {isEmpty ? '' : i + 1}
                </span>
                {/* Content */}
                <span style={{ color, whiteSpace: 'pre-wrap', flex: 1 }}>
                  {isMatch && q
                    ? highlightMatch(line, q)
                    : line || ' '}
                </span>
              </div>
            )
          })}
        </div>
      </div>

      {q && matchCount === 0 && (
        <p className="text-ark-cyan/30 text-xs text-center py-1">
          Sin resultados para "{search}" en {TAB_LABELS[activeTab]}
        </p>
      )}
    </div>
  )
}

// Highlight the matching substring in a line
function highlightMatch(line: string, q: string) {
  const idx = line.toLowerCase().indexOf(q)
  if (idx === -1) return line
  const before = line.slice(0, idx)
  const match = line.slice(idx, idx + q.length)
  const after = line.slice(idx + q.length)
  return (
    <>
      {before}
      <span style={{ background: 'rgba(0,200,255,0.35)', color: '#fff', borderRadius: '2px', padding: '0 1px' }}>
        {match}
      </span>
      {after}
    </>
  )
}
