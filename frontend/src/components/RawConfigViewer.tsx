import { useState, useMemo, useCallback, useEffect } from 'react'
import type { ServerConfig } from '../types'
import { invoke } from '../services/tauri'
import { useBackupStore } from '../stores/backupStore'
import { useI18n } from '../i18n/useI18n'
import { useTextHistory } from '../hooks/useTextHistory'
import ConfigFormEditor from './ConfigFormEditor'

interface RawConfigViewerProps {
  config: ServerConfig
  onConfigSaved?: (config: ServerConfig) => void
}

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

type BuiltinTab = 'gameusersettings' | 'game' | 'toml'
type EditMode = 'idle' | 'form' | 'raw'

interface RawConfigViewerProps {
  config: ServerConfig
  onConfigSaved?: (config: ServerConfig) => void
}

const BUILTIN_TABS: { id: BuiltinTab; label: string }[] = [
  { id: 'gameusersettings', label: 'GameUserSettings.ini' },
  { id: 'game', label: 'Game.ini' },
  { id: 'toml', label: 'config.toml' },
]

function pathForBuiltinTab(tab: BuiltinTab, config: ServerConfig): string | null {
  if (tab === 'game') return config.paths.game_ini_path
  if (tab === 'gameusersettings') return config.paths.gamesettings_ini_path
  return null
}

function generatedForTab(tab: BuiltinTab, config: ServerConfig): string {
  if (tab === 'gameusersettings') return generateGameUserSettings(config)
  if (tab === 'game') return generateGameIni(config)
  return generateToml(config)
}

export default function RawConfigViewer({ config, onConfigSaved }: RawConfigViewerProps) {
  const { customConfigTabs, addCustomConfigTab, removeCustomConfigTab } = useBackupStore()
  const { tk } = useI18n()

  const [activeTab, setActiveTab] = useState<string>('gameusersettings')
  const [editMode, setEditMode] = useState<EditMode>('idle')
  const [search, setSearch] = useState('')
  const [copied, setCopied] = useState(false)
  const [saveStatus, setSaveStatus] = useState<string | null>(null)
  const [saveError, setSaveError] = useState<string | null>(null)
  const [isSaving, setIsSaving] = useState(false)

  const { text, commit, reset, undo, redo, canUndo, canRedo } = useTextHistory('')

  const allTabs = useMemo(
    () => [
      ...BUILTIN_TABS,
      ...customConfigTabs.map((t) => ({ id: t.id, label: t.label })),
    ],
    [customConfigTabs]
  )

  const activeBuiltin = BUILTIN_TABS.find((t) => t.id === activeTab)?.id as BuiltinTab | undefined
  const activeCustom = customConfigTabs.find((t) => t.id === activeTab)

  const generatedContent = useMemo(() => {
    if (activeBuiltin) return generatedForTab(activeBuiltin, config)
    return ''
  }, [activeBuiltin, config])

  const isEditing = editMode !== 'idle'
  const displayContent = isEditing ? text : (activeBuiltin ? generatedContent : text)

  const loadTabContent = useCallback(async (tabId: string) => {
    const builtin = BUILTIN_TABS.find((t) => t.id === tabId)?.id as BuiltinTab | undefined
    const custom = customConfigTabs.find((t) => t.id === tabId)

    if (builtin) {
      const path = pathForBuiltinTab(builtin, config)
      const generated = generatedForTab(builtin, config)
      if (path) {
        try {
          const disk = await invoke<string>('read_text_file', { path })
          reset(disk.trim() ? disk : generated)
        } catch {
          reset(generated)
        }
      } else {
        reset(generated)
      }
    } else if (custom) {
      try {
        const disk = await invoke<string>('read_text_file', { path: custom.path })
        reset(disk)
      } catch {
        reset('')
      }
    }
  }, [config, customConfigTabs, reset])

  useEffect(() => {
    loadTabContent(activeTab)
    setEditMode('idle')
    setSearch('')
  }, [activeTab]) // eslint-disable-line react-hooks/exhaustive-deps

  const handleStartEdit = async (mode: EditMode) => {
    await loadTabContent(activeTab)
    setEditMode(mode)
  }

  const handleCancelEdit = () => {
    setEditMode('idle')
    loadTabContent(activeTab)
  }

  const saveContent = async (content: string) => {
    setIsSaving(true)
    setSaveStatus(null)
    setSaveError(null)
    try {
      if (activeBuiltin === 'toml') {
        const parsed = await invoke<ServerConfig>('parse_config_from_toml', { tomlStr: content })
        await invoke('save_config', { config: parsed })
        onConfigSaved?.(parsed)
        setSaveStatus(tk('config_saved', 'Configuration saved'))
      } else if (activeBuiltin === 'game' || activeBuiltin === 'gameusersettings') {
        const path = pathForBuiltinTab(activeBuiltin, config)!
        await invoke('write_text_file', { path, content })
        const merged = await invoke<ServerConfig>('merge_config_from_ini', { config, iniContent: content })
        await invoke('save_config', { config: merged })
        onConfigSaved?.(merged)
        setSaveStatus(tk('config_saved', 'Configuration saved'))
      } else if (activeCustom) {
        await invoke('write_text_file', { path: activeCustom.path, content })
        setSaveStatus(tk('file_saved', 'File saved'))
      }
      setEditMode('idle')
    } catch (e) {
      setSaveError(String(e))
    } finally {
      setIsSaving(false)
    }
  }

  const handleSave = () => saveContent(text)

  const handleFormSave = async (newContent: string) => {
    await saveContent(newContent)
  }

  const handleAddTab = () => {
    const label = prompt(tk('custom_tab_label_prompt', 'Tab name (e.g. Custom.ini):'))
    if (!label?.trim()) return
    const path = prompt(tk('custom_tab_path_prompt', 'Full file path:'))
    if (!path?.trim()) return
    const id = `custom-${Date.now()}`
    addCustomConfigTab({ id, label: label.trim(), path: path.trim() })
    setActiveTab(id)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isEditing) return
    if (e.ctrlKey && e.key === 'z' && !e.shiftKey) {
      e.preventDefault()
      undo()
    }
    if ((e.ctrlKey && e.key === 'y') || (e.ctrlKey && e.shiftKey && e.key === 'z')) {
      e.preventDefault()
      redo()
    }
  }

  const q = search.trim().toLowerCase()
  const lines = useMemo(() => displayContent.split('\n'), [displayContent])
  const matchCount = useMemo(
    () => (q ? lines.filter((l) => l.toLowerCase().includes(q)).length : 0),
    [lines, q]
  )

  const handleCopy = () => {
    navigator.clipboard.writeText(displayContent)
    setCopied(true)
    setTimeout(() => setCopied(false), 1800)
  }

  const activeLabel = allTabs.find((t) => t.id === activeTab)?.label ?? activeTab

  return (
    <div className="flex flex-col gap-3 h-full" onKeyDown={handleKeyDown}>
      {/* Sub-tab bar */}
      <div className="flex gap-1 flex-wrap items-center">
        {allTabs.map((t) => (
          <button
            key={t.id}
            onClick={() => setActiveTab(t.id)}
            className="px-3 py-1.5 text-[11px] font-bold tracking-wider rounded-md transition-all font-mono"
            style={{
              background: activeTab === t.id ? 'rgba(0,200,255,0.12)' : 'rgba(255,255,255,0.03)',
              border: `1px solid ${activeTab === t.id ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.08)'}`,
              color: activeTab === t.id ? 'rgba(0,200,255,0.9)' : 'rgba(255,255,255,0.35)',
            }}
          >
            {t.label}
          </button>
        ))}
        <button
          onClick={handleAddTab}
          className="px-2 py-1.5 text-[11px] rounded-md"
          style={{ border: '1px dashed rgba(0,200,255,0.35)', color: 'rgba(0,200,255,0.5)' }}
          title={tk('add_config_tab', 'Add config file')}
        >
          + {tk('add', 'Add')}
        </button>
        {activeCustom && (
          <button
            onClick={() => {
              if (confirm(tk('remove_tab_confirm', 'Remove this tab?'))) {
                removeCustomConfigTab(activeCustom.id)
                setActiveTab('gameusersettings')
              }
            }}
            className="px-2 py-1 text-[10px] text-red-400/60 hover:text-red-400"
          >
            ✕
          </button>
        )}
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 flex-wrap">
        {editMode === 'idle' ? (
          <>
            <button onClick={() => handleStartEdit('form')} className="ark-action-btn text-[10px] px-3 py-1.5">
              {tk('form_edit', 'Form Edit')}
            </button>
            <button onClick={() => handleStartEdit('raw')} className="ark-action-btn text-[10px] px-3 py-1.5">
              {tk('raw_edit', 'Raw Edit')}
            </button>
          </>
        ) : (
          <>
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="ark-action-btn text-[10px] px-3 py-1.5 disabled:opacity-40"
              style={{ display: editMode === 'raw' ? undefined : 'none' }}
            >
              {isSaving ? tk('saving', 'Saving...') : tk('save', 'Save')}
            </button>
            <button onClick={handleCancelEdit} className="ark-action-btn text-[10px] px-3 py-1.5">
              {tk('cancel', 'Cancel')}
            </button>
            {editMode === 'raw' && (
              <>
                <button
                  onClick={undo}
                  disabled={!canUndo}
                  className="ark-action-btn text-[10px] px-2 py-1.5 disabled:opacity-25"
                >
                  ↩
                </button>
                <button
                  onClick={redo}
                  disabled={!canRedo}
                  className="ark-action-btn text-[10px] px-2 py-1.5 disabled:opacity-25"
                >
                  ↪
                </button>
                <span className="text-ark-cyan/30 text-[10px]">{tk('undo_redo_hint', 'Ctrl+Z / Ctrl+Y')}</span>
              </>
            )}
          </>
        )}

        <div className="relative flex-1 min-w-[140px]">
          <span className="absolute left-3 top-1/2 -translate-y-1/2 text-ark-cyan/40 text-xs pointer-events-none">🔍</span>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={tk('config_search_placeholder', 'Search keys…')}
            disabled={isEditing}
            className="w-full bg-transparent border border-ark-cyan/25 text-ark-cyan/90 text-xs pl-8 pr-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/25 font-mono disabled:opacity-40"
          />
        </div>

        <button onClick={handleCopy} className="ark-action-btn text-[10px] px-3 py-1.5 flex-shrink-0">
          {copied ? '✓ Copiado' : '⎘ Copiar'}
        </button>
      </div>

      {saveStatus && <p className="text-green-400/80 text-xs">{saveStatus}</p>}
      {saveError && <p className="text-red-400/80 text-xs">{saveError}</p>}

      {/* Content */}
      {editMode === 'form' ? (
        <ConfigFormEditor
          content={text}
          onSave={handleFormSave}
          onCancel={handleCancelEdit}
        />
      ) : editMode === 'raw' ? (
        <textarea
          value={text}
          onChange={(e) => commit(e.target.value)}
          className="flex-1 w-full font-mono text-[11px] leading-relaxed rounded-lg p-3 resize-none focus:outline-none"
          style={{
            background: 'rgba(0,0,0,0.35)',
            border: '1px solid rgba(0,200,255,0.25)',
            color: 'rgba(180,220,255,0.85)',
            minHeight: 'calc(90vh - 320px)',
            maxHeight: 'calc(90vh - 320px)',
          }}
          spellCheck={false}
        />
      ) : (
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
                  <span
                    className="select-none flex-shrink-0 text-right pr-3 w-9"
                    style={{ color: 'rgba(100,130,150,0.4)', fontSize: '10px', lineHeight: '1.6' }}
                  >
                    {isEmpty ? '' : i + 1}
                  </span>
                  <span style={{ color, whiteSpace: 'pre-wrap', flex: 1 }}>
                    {isMatch && q ? highlightMatch(line, q) : line || ' '}
                  </span>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {q && !isEditing && matchCount === 0 && (
        <p className="text-ark-cyan/30 text-xs text-center py-1">
          Sin resultados para "{search}" en {activeLabel}
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
