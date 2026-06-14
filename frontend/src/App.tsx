import { useEffect, useRef, useState } from 'react'
import { initializeTauri, invoke, getTauriStatus } from './services/tauri'
import { logger } from './services/logger'
import ArkLayout from './components/ArkLayout'
import PrimaryNav from './components/PrimaryNav'
import SubNav from './components/SubNav'
import ActionBar from './components/ActionBar'
import DifficultyModal from './components/DifficultyModal'
import OptionsModal from './components/OptionsModal'
import ServerLogsPanel from './components/ServerLogsPanel'
import LogsViewer from './components/LogsViewer'
import { useConfigStore } from './stores/configStore'
import { useUiStore } from './stores/uiStore'
import { useBackupStore } from './stores/backupStore'
import type { ServerConfig, PrimaryTab } from './types'

// Tab page imports
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

function App() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [tauriStatus, setTauriStatus] = useState<string>('')
  const [serverRunning, setServerRunning] = useState(false)
  const [stubsRunning, setStubsRunning] = useState(false)  // on-demand stubs active (no ARK process yet)
  const [isServerStarting, setIsServerStarting] = useState(false)
  const [isServerStopping, setIsServerStopping] = useState(false)
  const [showDifficultyModal, setShowDifficultyModal] = useState(false)
  const [showOptionsModal, setShowOptionsModal] = useState(false)
  const [showLogsPanel, setShowLogsPanel] = useState(false)
  const { config, setConfig, isSaving, setSaving } = useConfigStore()
  const { primaryTab, setPrimaryTab, gameRulesSubTab, advancedSubTab, modSettingsSubTab, goBack } = useUiStore()
  const { logsEnabled, minimizeToTray, manualSave, onDemandEnabled, onDemandMaps, autoShutdownMin } = useBackupStore()
  const autoSaveTimer = useRef<ReturnType<typeof setTimeout>>()
  const errorDismissTimer = useRef<ReturnType<typeof setTimeout>>()
  // Keep a ref to the latest config so the close handler always sees current state
  const configRef = useRef(config)
  // Keep a ref to minimizeToTray so the close handler always sees current value
  const minimizeToTrayRef = useRef(minimizeToTray)

  const setErrorAutoDismiss = (msg: string | null) => {
    setError(msg)
    clearTimeout(errorDismissTimer.current)
    if (msg !== null) {
      errorDismissTimer.current = setTimeout(() => setError(null), 3000)
    }
  }

  useEffect(() => {
    logger.info('App component mounted')
    initAppAndConfig()
  }, [])

  // Escape key → toggle Options modal (unless another modal is open)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      if (showDifficultyModal) return // let DifficultyModal handle it
      setShowOptionsModal((prev) => !prev)
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [showDifficultyModal])

  // Keep refs in sync
  useEffect(() => { configRef.current = config }, [config])
  useEffect(() => { minimizeToTrayRef.current = minimizeToTray }, [minimizeToTray])

  // Sync minimizeToTray to Rust whenever it changes
  useEffect(() => {
    invoke('set_minimize_to_tray', { enabled: minimizeToTray }).catch(() => {})
  }, [minimizeToTray])

  // Listen for tray "Salir" — save config before the process exits
  useEffect(() => {
    let unlisten: (() => void) | undefined
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen('tray-quit', async () => {
        const cfg = configRef.current
        if (cfg) {
          try { await invoke('save_config', { config: cfg }) } catch {}
        }
        await invoke('quit_app')
      }).then((fn) => { unlisten = fn })
    }).catch(() => {})
    return () => { unlisten?.() }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // Auto-save config to TOML on every change (debounced 1.5s) — disabled in manual save mode
  useEffect(() => {
    if (!config || manualSave) return
    clearTimeout(autoSaveTimer.current)
    autoSaveTimer.current = setTimeout(() => {
      invoke('save_config', { config }).catch(err =>
        logger.warn('Auto-save failed', err)
      )
    }, 1500)
    return () => clearTimeout(autoSaveTimer.current)
  }, [config]) // eslint-disable-line react-hooks/exhaustive-deps

  // Save immediately when the window is closed (beats the 1.5s debounce race)
  useEffect(() => {
    let unlisten: (() => void) | undefined
    import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
      const win = getCurrentWindow()
      win.onCloseRequested(async (event) => {
        event.preventDefault()
        const cfg = configRef.current
        if (cfg) {
          try {
            await invoke('save_config', { config: cfg })
          } catch (err) {
            logger.warn('Close-save failed', err)
          }
        }
        if (minimizeToTrayRef.current) {
          await win.hide()
        } else {
          await win.destroy()
        }
      }).then(fn => { unlisten = fn })
    }).catch(() => { /* not in Tauri */ })
    return () => { unlisten?.() }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const initAppAndConfig = async () => {
    try {
      logger.info('Initializing application...')
      const tauriReady = await initializeTauri()
      logger.info(`Tauri initialization result: ${tauriReady}`, getTauriStatus())
      setTauriStatus(JSON.stringify(getTauriStatus()))
      await loadConfig()
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      logger.error('App initialization failed', err)
      setErrorAutoDismiss(`Failed to initialize: ${errorMsg}`)
      setLoading(false)
    }
  }

  const loadConfig = async () => {
    try {
      logger.info('Loading saved config...')
      setLoading(true)

      // If we already have a config in localStorage (from persist middleware), keep it
      // but still try to load the latest from disk to stay in sync
      const savedConfig: ServerConfig = await invoke('load_config_or_default')
      logger.info('Config loaded successfully', savedConfig)
      setConfig(savedConfig)
      setError(null)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      logger.warn('Failed to load saved config, keeping localStorage version', err)
      // If Tauri load fails (e.g. web mode), the persist middleware already
      // restored config from localStorage — so only set error if we have nothing
      if (!config) {
        setErrorAutoDismiss(`Failed to load config: ${errorMsg}`)
      }
    } finally {
      setLoading(false)
    }
  }

  const handleSave = async () => {
    if (!config) return
    try {
      setSaving(true)
      await invoke('save_config', { config })
      logger.info('Config saved successfully')
      setError(null)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to save config', err)
      setErrorAutoDismiss(`Failed to save: ${errorMsg}`)
      throw err  // rethrow so ActionBar doesn't show false "GUARDADO ✓"
    } finally {
      setSaving(false)
    }
  }

  const handleChooseDifficulty = () => setShowDifficultyModal(true)

  const handleDifficultySelect = (value: number) => {
    if (!config) return
    setConfig({ ...config, gameplay: { ...config.gameplay, override_official_difficulty: value } })
  }

  const handleReset = async () => {
    if (confirm('Reset to defaults?')) {
      try {
        const defaults = await invoke('get_default_config')
        setConfig(defaults)
      } catch (err) {
        setErrorAutoDismiss(`Failed to load defaults: ${err}`)
      }
    }
  }

  const handleStartServer = async () => {
    if (!config || isServerStarting || serverRunning) return  // prevent double-click
    try {
      setIsServerStarting(true)
      setError(null)

      // Validate active mods before launching
      const activeMods = config.mods?.active_mods ?? []
      if (activeMods.length > 0) {
        try {
          const unavailable = await invoke<string[]>('check_mods_available', { modIds: activeMods })
          if (unavailable.length > 0) {
            const proceed = confirm(
              `⚠️ Los siguientes mods no están disponibles en CurseForge y pueden crashear el servidor:\n\n${unavailable.join(', ')}\n\nSe recomienda eliminarlos de Mods Activos antes de lanzar.\n\n¿Continuar de todos modos?`
            )
            if (!proceed) return
          }
        } catch {
          // Ignore validation errors — don't block launch
        }
      }

      // Auto-save before launching so the INI files are up to date
      setSaving(true)
      await invoke('save_config', { config })
      setSaving(false)

      const maps: string[] = config.cluster_maps?.length ? config.cluster_maps : ['TheIsland_WP']

      // Split maps into always-on vs on-demand (dormant stubs)
      const normalIndices: number[] = []
      const dormantIndices: number[] = []
      maps.forEach((mapId, i) => {
        if (onDemandEnabled && onDemandMaps.includes(mapId)) dormantIndices.push(i)
        else normalIndices.push(i)
      })

      // Start on-demand stubs (non-blocking — they just bind ports)
      for (const idx of dormantIndices) {
        try {
          await invoke('enable_on_demand', {
            config,
            mapIndex: idx,
            autoShutdownMin,
          })
          logger.info(`On-demand stub started for map index ${idx}`)
        } catch (err) {
          logger.warn(`Failed to start stub for map index ${idx}`, err)
        }
      }

      // Launch always-on maps via normal start_server (only if there are any)
      if (normalIndices.length > 0) {
        // Build a config slice with only the always-on maps
        const normalConfig = {
          ...config,
          cluster_maps: normalIndices.map((i) => maps[i]),
        }
        const msg = await invoke<string>('start_server', { config: normalConfig })
        logger.info('Server start result', msg)
        setServerRunning(true)
      }

      // If only stubs (no always-on ARK), track via stubsRunning instead
      if (dormantIndices.length > 0 && normalIndices.length === 0) {
        setStubsRunning(true)
      }

      setError(null)
    } catch (err) {
      setSaving(false)
      const errorMsg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to start server', err)
      setErrorAutoDismiss(`Server start failed: ${errorMsg}`)
    } finally {
      setIsServerStarting(false)
    }
  }

  const handleStopServer = async () => {
    if (isServerStopping) return  // prevent double-click
    try {
      setIsServerStopping(true)
      // Stop all on-demand stubs first
      invoke('disable_all_on_demand').catch(() => {})
      const msg = await invoke<string>('stop_server')
      logger.info('Server stop result', msg)
      setError(null)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      logger.warn('stop_server error (treating as stopped)', err)
      setErrorAutoDismiss(`Server stop failed: ${errorMsg}`)
    } finally {
      setServerRunning(false)
      setStubsRunning(false)
      setIsServerStopping(false)
    }
  }

  // Poll server process every 5s while serverRunning is true — detect crashes
  useEffect(() => {
    if (!serverRunning) return
    const interval = setInterval(async () => {
      try {
        const running = await invoke<boolean>('is_server_running')
        if (!running) {
          logger.info('Server process no longer detected — updating UI state')
          setServerRunning(false)
          setErrorAutoDismiss('El servidor se detuvo inesperadamente')
        }
      } catch {
        // ignore transient poll errors
      }
    }, 5000)
    return () => clearInterval(interval)
  }, [serverRunning]) // eslint-disable-line react-hooks/exhaustive-deps

  const renderPage = () => {
    if (!config) return null

    switch (primaryTab) {
      case 'arks':
        return <ArksTab config={config} />
      case 'mod_settings':
        return modSettingsSubTab === 'active_mods' ? <ActiveModsTab /> : <AvailableModsTab />
      case 'game_rules':
        switch (gameRulesSubTab) {
          case 'player':
            return <PlayerTab config={config} />
          case 'creature':
            return <CreatureTab config={config} />
          case 'structure':
            return <StructureTab config={config} />
          case 'world':
            return <WorldRulesTab config={config} />
          case 'rules':
            return <RulesTab config={config} />
        }
        break
      case 'advanced':
        switch (advancedSubTab) {
          case 'pve':
            return <PveTab config={config} />
          case 'pvp':
            return <PvpTab config={config} />
          case 'world':
            return <WorldAdvancedTab config={config} />
          case 'wild_dino':
            return <WildDinoTab config={config} />
          case 'tamed_dino':
            return <TamedDinoTab config={config} />
          case 'player':
            return <PlayerStatsTab config={config} />
          case 'xp_multipliers':
            return <XpMultipliersTab config={config} />
          case 'misc':
            return <MiscTab config={config} />
        }
        break
      case 'engrams':
        return <EngramsTab />
    }
  }

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

      <ActionBar
        onSave={handleSave}
        onReset={handleReset}
        onBack={goBack}
        onChooseDifficulty={handleChooseDifficulty}
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
          onClose={() => setShowLogsPanel(false)}
        />
      )}

      <LogsViewer />
    </ArkLayout>
  )
}

export default App
