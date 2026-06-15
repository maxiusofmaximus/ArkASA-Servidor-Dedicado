import { useEffect, useState, useCallback, type ReactNode } from 'react'
import { initializeTauri, invoke, getTauriStatus } from './services/tauri'
import { logger } from './services/logger'
import { useConfigStore } from './stores/configStore'
import { useUiStore } from './stores/uiStore'
import { useBackupStore } from './stores/backupStore'
import { useServerLifecycle } from './hooks/useServerLifecycle'
import { useAutoSave } from './hooks/useAutoSave'
import { useTauriEvents } from './hooks/useTauriEvents'

// Layout & chrome
import ArkLayout from './components/ArkLayout'
import PrimaryNav from './components/PrimaryNav'
import SubNav from './components/SubNav'
import ActionBar from './components/ActionBar'
import DifficultyModal from './components/DifficultyModal'
import OptionsModal from './components/OptionsModal'
import ServerLogsPanel from './components/ServerLogsPanel'
import LogsViewer from './components/LogsViewer'

// Tab pages
import ArksTab from './pages/arks/ArksTab'
import PlayerTab from './pages/game-rules/PlayerTab'
import CreatureTab from './pages/game-rules/CreatureTab'
import StructureTab from './pages/game-rules/StructureTab'
import WorldRulesTab from './pages/game-rules/WorldRulesTab'
import RulesTab from './pages/game-rules/RulesTab'
import PveTab from './pages/advanced/PveTab'
import PvpTab from './pages/advanced/PvpTab'
import WorldAdvancedTab from './pages/advanced/WorldAdvancedTab'
import WildDinoTab from './pages/advanced/WildDinoTab'
import TamedDinoTab from './pages/advanced/TamedDinoTab'
import PlayerStatsTab from './pages/advanced/PlayerStatsTab'
import XpMultipliersTab from './pages/advanced/XpMultipliersTab'
import MiscTab from './pages/advanced/MiscTab'
import ActiveModsTab from './pages/mod-settings/ActiveModsTab'
import AvailableModsTab from './pages/mod-settings/AvailableModsTab'
import EngramsTab from './pages/engrams/EngramsTab'

import type { ServerConfig, GameRulesSubTab, AdvancedSubTab } from './types'

// ─────────────────────────────────────────────────────────────────────────────
// Tab registries (OCP: add a new sub-tab here without touching the renderer)
// ─────────────────────────────────────────────────────────────────────────────

const GAME_RULES_TABS: Record<GameRulesSubTab, (c: ServerConfig) => ReactNode> = {
  player:    (c) => <PlayerTab config={c} />,
  creature:  (c) => <CreatureTab config={c} />,
  structure: (c) => <StructureTab config={c} />,
  world:     (c) => <WorldRulesTab config={c} />,
  rules:     (c) => <RulesTab config={c} />,
}

const ADVANCED_TABS: Record<AdvancedSubTab, (c: ServerConfig) => ReactNode> = {
  pve:            (c) => <PveTab config={c} />,
  pvp:            (c) => <PvpTab config={c} />,
  world:          (c) => <WorldAdvancedTab config={c} />,
  wild_dino:      (c) => <WildDinoTab config={c} />,
  tamed_dino:     (c) => <TamedDinoTab config={c} />,
  player:         (c) => <PlayerStatsTab config={c} />,
  xp_multipliers: (c) => <XpMultipliersTab config={c} />,
  misc:           (c) => <MiscTab config={c} />,
}

// ─────────────────────────────────────────────────────────────────────────────
// App
// ─────────────────────────────────────────────────────────────────────────────

function App() {
  const [loading, setLoading] = useState(true)
  const [error,   setErrorRaw] = useState<string | null>(null)
  const [wakeInfo, setWakeInfoRaw] = useState<string | null>(null)
  const [showDifficultyModal, setShowDifficultyModal] = useState(false)
  const [showOptionsModal,    setShowOptionsModal]    = useState(false)
  const [showLogsPanel,       setShowLogsPanel]       = useState(false)

  const { config, setConfig, isSaving, setSaving } = useConfigStore()
  const { primaryTab, setPrimaryTab, gameRulesSubTab, advancedSubTab, modSettingsSubTab, goBack } = useUiStore()
  const { logsEnabled, minimizeToTray, manualSave } = useBackupStore()

  // ── Auto-dismiss error after 3 s ──────────────────────────────────────────
  const errorTimer = useState<ReturnType<typeof setTimeout>>()[0]
  const setError = useCallback((msg: string | null) => {
    setErrorRaw(msg)
    if (msg !== null) {
      clearTimeout(errorTimer)
      setTimeout(() => setErrorRaw(null), 3_000)
    }
  }, [errorTimer])

  // ── Auto-dismiss wake notification after 30 s ─────────────────────────────
  const wakeTimer = useState<ReturnType<typeof setTimeout>>()[0]
  const setWakeInfo = useCallback((msg: string | null) => {
    setWakeInfoRaw(msg)
    if (msg !== null) {
      clearTimeout(wakeTimer)
      setTimeout(() => setWakeInfoRaw(null), 30_000)
    }
  }, [wakeTimer])

  // ── Hooks ──────────────────────────────────────────────────────────────────
  const {
    serverRunning, stubsRunning,
    isServerStarting, isServerStopping,
    handleStartServer, handleStopServer,
  } = useServerLifecycle({ config, setSaving, setError })

  useAutoSave(config, !manualSave)
  useTauriEvents({
    config,
    minimizeToTray,
    onDemandWaking: (map) => setWakeInfo(`⏳ ${map.replace('_WP', '')} está iniciando… reconéctate en ~5 min`),
    onDemandReady:  (map) => setWakeInfo(`✅ ${map.replace('_WP', '')} está listo — conéctate ahora`),
  })

  // Escape → toggle Options (unless DifficultyModal is open)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || showDifficultyModal) return
      setShowOptionsModal((p) => !p)
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [showDifficultyModal])

  // ── Init ───────────────────────────────────────────────────────────────────
  useEffect(() => {
    logger.info('App mounted')
    initAppAndConfig()
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  async function initAppAndConfig() {
    try {
      const tauriReady = await initializeTauri()
      logger.info(`Tauri init: ${tauriReady}`, getTauriStatus())
      await loadConfig()
    } catch (err) {
      logger.error('App init failed', err)
      setError(`Failed to initialize: ${err instanceof Error ? err.message : String(err)}`)
      setLoading(false)
    }
  }

  async function loadConfig() {
    try {
      setLoading(true)
      const saved: ServerConfig = await invoke('load_config_or_default')
      setConfig(saved)
      setError(null)
    } catch (err) {
      logger.warn('Failed to load saved config, keeping localStorage version', err)
      if (!config) setError(`Failed to load config: ${err instanceof Error ? err.message : String(err)}`)
    } finally {
      setLoading(false)
    }
  }

  // ── Config actions ─────────────────────────────────────────────────────────
  const handleSave = async () => {
    if (!config) return
    try {
      setSaving(true)
      await invoke('save_config', { config })
      setError(null)
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to save config', err)
      setError(`Failed to save: ${msg}`)
      throw err // let ActionBar know it failed
    } finally {
      setSaving(false)
    }
  }

  const handleReset = async () => {
    if (!confirm('Reset to defaults?')) return
    try {
      setConfig(await invoke('get_default_config'))
    } catch (err) {
      setError(`Failed to load defaults: ${err}`)
    }
  }

  const handleDifficultySelect = (value: number) => {
    if (!config) return
    setConfig({ ...config, gameplay: { ...config.gameplay, override_official_difficulty: value } })
  }

  // ── Page renderer (registry pattern — OCP) ────────────────────────────────
  const renderPage = useCallback((): ReactNode => {
    if (!config) return null
    switch (primaryTab) {
      case 'arks':         return <ArksTab config={config} />
      case 'engrams':      return <EngramsTab />
      case 'mod_settings': return modSettingsSubTab === 'active_mods' ? <ActiveModsTab /> : <AvailableModsTab />
      case 'game_rules':   return GAME_RULES_TABS[gameRulesSubTab]?.(config) ?? null
      case 'advanced':     return ADVANCED_TABS[advancedSubTab]?.(config) ?? null
      default:             return null
    }
  }, [config, primaryTab, gameRulesSubTab, advancedSubTab, modSettingsSubTab])

  // ── Render ────────────────────────────────────────────────────────────────
  if (loading) {
    return (
      <ArkLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-ark-cyan text-2xl">Loading configuration...</div>
        </div>
      </ArkLayout>
    )
  }

  if (error && !config) {
    return (
      <ArkLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-ark-accent text-xl">{error}</div>
        </div>
      </ArkLayout>
    )
  }

  return (
    <ArkLayout>
      <PrimaryNav activeTab={primaryTab} onTabChange={setPrimaryTab} />
      <SubNav primaryTab={primaryTab} />

      <main className="pb-24">{renderPage()}</main>

      {error && (
        <div className="fixed top-20 right-8 bg-ark-accent/20 border border-ark-accent text-ark-accent p-4 rounded z-50 max-w-sm">
          {error}
        </div>
      )}

      {wakeInfo && (
        <div className="fixed top-36 right-8 bg-ark-cyan/10 border border-ark-cyan text-ark-cyan p-4 rounded z-50 max-w-sm text-sm">
          {wakeInfo}
        </div>
      )}

      <ActionBar
        onSave={handleSave}
        onReset={handleReset}
        onBack={goBack}
        onChooseDifficulty={() => setShowDifficultyModal(true)}
        onStartServer={handleStartServer}
        onStopServer={handleStopServer}
        onOpenOptions={() => setShowOptionsModal(true)}
        onToggleLogs={() => setShowLogsPanel((p) => !p)}
        isSaving={isSaving}
        autoSave={!manualSave}
        isServerRunning={serverRunning || stubsRunning}
        isServerStarting={isServerStarting}
        isServerStopping={isServerStopping}
        showLogsButton={logsEnabled}
        isLogsOpen={showLogsPanel}
        variant={primaryTab === 'mod_settings' ? 'mod_settings' : 'default'}
      />

      {showDifficultyModal && config && (
        <DifficultyModal
          currentValue={config.gameplay.override_official_difficulty}
          onSelect={handleDifficultySelect}
          onClose={() => setShowDifficultyModal(false)}
        />
      )}

      {showOptionsModal && (
        <OptionsModal onClose={() => setShowOptionsModal(false)} />
      )}

      {showLogsPanel && config && (
        <ServerLogsPanel
          serverDir={config.paths.server_dir}
          maps={config.cluster_maps?.length ? config.cluster_maps : ['TheIsland_WP']}
          onClose={() => setShowLogsPanel(false)}
        />
      )}

      <LogsViewer />
    </ArkLayout>
  )
}

export default App
