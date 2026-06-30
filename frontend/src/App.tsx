import { useEffect, useState, useCallback, lazy, Suspense, type ReactNode } from 'react'
import { initializeTauri, invoke, getTauriStatus } from './services/tauri'
import { logger } from './services/logger'
import { useConfigStore, type ConfigStore } from './stores/configStore'
import { useUiStore, type UiStore } from './stores/uiStore'
import { useBackupStore, type BackupStore } from './stores/backupStore'
import { useShallow } from 'zustand/react/shallow'
import { useServerLifecycle } from './hooks/useServerLifecycle'
import { useAutoSave } from './hooks/useAutoSave'
import { useTauriEvents } from './hooks/useTauriEvents'
import { useInternetStatus } from './hooks/useInternetStatus'
import { useI18n } from './i18n/useI18n'

// Layout & chrome
import ArkLayout from './components/ArkLayout'
import PrimaryNav from './components/PrimaryNav'
import SubNav from './components/SubNav'
import ActionBar from './components/ActionBar'
import DifficultyModal from './components/DifficultyModal'
import OptionsModal from './components/OptionsModal'
import ConfigDiffViewer from './components/ConfigDiffViewer'
import ServerLogsPanel from './components/ServerLogsPanel'
import LogsViewer from './components/LogsViewer'

import { computeDiff } from './utils/configDiff'
import type { DiffEntry } from './utils/configDiff'

// Tab pages (lazy-loaded for code splitting)
const ArksTab = lazy(() => import('./pages/arks/ArksTab'))
const PlayerTab = lazy(() => import('./pages/game-rules/PlayerTab'))
const CreatureTab = lazy(() => import('./pages/game-rules/CreatureTab'))
const StructureTab = lazy(() => import('./pages/game-rules/StructureTab'))
const WorldRulesTab = lazy(() => import('./pages/game-rules/WorldRulesTab'))
const RulesTab = lazy(() => import('./pages/game-rules/RulesTab'))
const PveTab = lazy(() => import('./pages/advanced/PveTab'))
const PvpTab = lazy(() => import('./pages/advanced/PvpTab'))
const WorldAdvancedTab = lazy(() => import('./pages/advanced/WorldAdvancedTab'))
const WildDinoTab = lazy(() => import('./pages/advanced/WildDinoTab'))
const TamedDinoTab = lazy(() => import('./pages/advanced/TamedDinoTab'))
const PlayerStatsTab = lazy(() => import('./pages/advanced/PlayerStatsTab'))
const XpMultipliersTab = lazy(() => import('./pages/advanced/XpMultipliersTab'))
const MiscTab = lazy(() => import('./pages/advanced/MiscTab'))
const ActiveModsTab = lazy(() => import('./pages/mod-settings/ActiveModsTab'))
const AvailableModsTab = lazy(() => import('./pages/mod-settings/AvailableModsTab'))
const EngramsTab = lazy(() => import('./pages/engrams/EngramsTab'))

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

  const { config, savedConfig, setConfig, setSavedConfig, isSaving, setSaving,
          undo, redo, historyIndex, history } = useConfigStore(
            useShallow((s: ConfigStore) => ({
              config: s.config, savedConfig: s.savedConfig,
              setConfig: s.setConfig, setSavedConfig: s.setSavedConfig,
              isSaving: s.isSaving, setSaving: s.setSaving,
              undo: s.undo, redo: s.redo,
              historyIndex: s.historyIndex, history: s.history,
            }))
          )
  const { primaryTab, setPrimaryTab, gameRulesSubTab, advancedSubTab, modSettingsSubTab } = useUiStore(
            useShallow((s: UiStore) => ({
              primaryTab: s.primaryTab, setPrimaryTab: s.setPrimaryTab,
              gameRulesSubTab: s.gameRulesSubTab, advancedSubTab: s.advancedSubTab,
              modSettingsSubTab: s.modSettingsSubTab,
            }))
          )
  const { logsEnabled, minimizeToTray, manualSave } = useBackupStore(
            useShallow((s: BackupStore) => ({
              logsEnabled: s.logsEnabled, minimizeToTray: s.minimizeToTray, manualSave: s.manualSave,
            }))
          )
  const { tk } = useI18n()

  const internet = useInternetStatus()
  const online = internet.online

  // ── Diff viewer state ─────────────────────────────────────────────────────
  const [diffEntries,  setDiffEntries]  = useState<DiffEntry[]>([])
  const [diffTitle,    setDiffTitle]    = useState<string>()
  const [diffApplyLabel, setDiffApplyLabel] = useState<string>(tk('apply_changes', 'Apply Changes'))
  const [pendingApply, setPendingApply] = useState<(() => void) | null>(null)

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
    mapStatuses,
    handleStartServer, handleStopServer,
  } = useServerLifecycle({ config, setSaving, setError, online })

  useAutoSave(config, !manualSave)
  useTauriEvents({
    config,
    minimizeToTray,
    onDemandWaking: (map) => setWakeInfo(`⏳ ${tk('on_demand_waking', '{{name}} is starting… reconnect in ~5 min').replace('{{name}}', map.replace('_WP', ''))}`),
    onDemandReady:  (map) => setWakeInfo(`✅ ${tk('on_demand_ready', '{{name}} is ready — connect now').replace('{{name}}', map.replace('_WP', ''))}`),
  })

  // Escape → toggle Options (unless DifficultyModal is open)
  // Ctrl+Z → undo, Ctrl+Y / Ctrl+Shift+Z → redo
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !showDifficultyModal) {
        setShowOptionsModal((p) => !p)
        return
      }
      const ctrl = e.ctrlKey || e.metaKey
      if (!ctrl) return
      if (e.key === 'z' && !e.shiftKey) { e.preventDefault(); undo(); return }
      if (e.key === 'y' || (e.key === 'z' && e.shiftKey)) { e.preventDefault(); redo(); return }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [showDifficultyModal, undo, redo])

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
      setSavedConfig(saved)
      setError(null)
    } catch (err) {
      logger.warn('Failed to load saved config, keeping localStorage version', err)
      if (!config) setError(`Failed to load config: ${err instanceof Error ? err.message : String(err)}`)
    } finally {
      setLoading(false)
    }
  }

  // ── Config actions ─────────────────────────────────────────────────────────
  const doSave = async () => {
    if (!config) return
    try {
      setSaving(true)
      await invoke('save_config', { config })
      setSavedConfig(config)
      setError(null)
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to save config', err)
      setError(`Failed to save: ${msg}`)
      throw err
    } finally {
      setSaving(false)
    }
  }

  const handleSave = async () => {
    if (!config) return
    // In manual save mode, show diff if 6+ fields changed vs last saved state
    if (manualSave && savedConfig) {
      const diffs = computeDiff(savedConfig, config)
      if (diffs.length >= 6) {
        setDiffEntries(diffs)
        setDiffTitle(`${tk('save_settings', 'Save Settings')} — ${diffs.length} ${diffs.length === 1 ? tk('save_count_singular', 'change') : tk('save_count_plural', 'changes')}`)
        setDiffApplyLabel(tk('save_all', 'SAVE ALL'))
        setPendingApply(() => doSave)
        return
      }
    }
    await doSave()
  }

  const handleReset = async () => {
    if (!confirm('Reset to defaults?')) return
    try {
      setConfig(await invoke('get_default_config'))
    } catch (err) {
      setError(`Failed to load defaults: ${err}`)
    }
  }

  const handleImportConfig = async (tomlText: string) => {
    if (!config) return
    try {
      const imported: ServerConfig = await invoke('parse_config_from_toml', { tomlStr: tomlText })
      const diffs = computeDiff(config, imported)
      if (diffs.length === 0) {
        setError(tk('import_identical', 'The imported file is identical to the current configuration.'))
        return
      }
      setDiffEntries(diffs)
      setDiffTitle(`${tk('import_config', 'Import Config')} — ${diffs.length} ${diffs.length === 1 ? tk('save_count_singular', 'change') : tk('save_count_plural', 'changes')}`)
      setDiffApplyLabel(tk('apply_changes', 'Apply Changes'))
      setPendingApply(() => () => {
        setConfig(imported)
        setSavedConfig(imported)
        invoke('save_config', { config: imported }).catch((e) =>
          setError(`Failed to save: ${e}`)
        )
      })
    } catch (err) {
      setError(`${tk('import_error', 'Import error')}: ${err instanceof Error ? err.message : String(err)}`)
    }
  }

  const handleDiffApply = () => {
    if (pendingApply) pendingApply()
    setPendingApply(null)
    setDiffEntries([])
  }

  const handleDiffCancel = () => {
    setPendingApply(null)
    setDiffEntries([])
  }

  const handleDifficultySelect = (value: number) => {
    if (!config) return
    setConfig({ ...config, gameplay: { ...config.gameplay, override_official_difficulty: value } })
  }

  // ── Page renderer (registry pattern — OCP) ────────────────────────────────
  const renderPage = useCallback((): ReactNode => {
    if (!config) return null
    const page = (() => {
      switch (primaryTab) {
        case 'arks':         return <ArksTab config={config} />
        case 'engrams':      return <EngramsTab />
        case 'mod_settings': return modSettingsSubTab === 'active_mods' ? <ActiveModsTab /> : <AvailableModsTab />
        case 'game_rules':   return GAME_RULES_TABS[gameRulesSubTab]?.(config) ?? null
        case 'advanced':     return ADVANCED_TABS[advancedSubTab]?.(config) ?? null
        default:             return null
      }
    })()
    return <Suspense fallback={<div className="text-ark-cyan text-center py-8">Loading…</div>}>{page}</Suspense>
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

      {!online && mapStatuses.some((s) => !s.running) && (
        <div className="fixed bottom-20 right-8 bg-ark-accent/20 border border-ark-accent text-ark-accent p-4 rounded z-50 max-w-sm text-sm flex items-start gap-3">
          <span className="text-lg leading-none">⚠</span>
          <div>
            <p className="font-bold tracking-widest text-xs uppercase">
              {tk('no_internet_title', 'Sin conexión a internet')}
            </p>
            <p className="text-ark-accent/80 mt-1">
              {tk('no_internet_body', 'No se detecta internet. Los servidores no iniciarán hasta que se restablezca la conexión.')}
            </p>
            <button
              onClick={() => internet.refresh()}
              disabled={internet.loading}
              className="mt-2 text-[10px] uppercase tracking-widest px-2 py-1 rounded border border-ark-accent/40 hover:border-ark-accent transition-colors disabled:opacity-40"
            >
              {internet.loading ? tk('no_internet_checking', '⏳ Comprobando…') : tk('no_internet_retry', '↻ Reintentar')}
            </button>
          </div>
        </div>
      )}

      <ActionBar
        onSave={handleSave}
        onStartServer={handleStartServer}
        onStopServer={handleStopServer}
        mapStatuses={mapStatuses}
        canUndo={historyIndex > 0}
        canRedo={historyIndex < history.length - 1}
        onUndo={undo}
        onRedo={redo}
        isSaving={isSaving}
        autoSave={!manualSave}
        isServerRunning={serverRunning || stubsRunning}
        isServerStarting={isServerStarting}
        isServerStopping={isServerStopping}
        online={online}
        variant={primaryTab === 'mod_settings' ? 'mod_settings' : 'default'}
      />

      {diffEntries.length > 0 && (
        <ConfigDiffViewer
          title={diffTitle}
          entries={diffEntries}
          applyLabel={diffApplyLabel}
          onApply={handleDiffApply}
          onCancel={handleDiffCancel}
        />
      )}

      {showDifficultyModal && config && (
        <DifficultyModal
          currentValue={config.gameplay.override_official_difficulty}
          onSelect={handleDifficultySelect}
          onClose={() => setShowDifficultyModal(false)}
        />
      )}

      {showOptionsModal && (
        <OptionsModal
          onClose={() => setShowOptionsModal(false)}
          onReset={handleReset}
          onChooseDifficulty={() => setShowDifficultyModal(true)}
          onImportConfig={handleImportConfig}
          isSaving={isSaving}
          onToggleLogs={() => setShowLogsPanel((p) => !p)}
          isLogsOpen={showLogsPanel}
        />
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
