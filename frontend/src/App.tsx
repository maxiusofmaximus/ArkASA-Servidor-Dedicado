import { useEffect, useState } from 'react'
import { initializeTauri, invoke, getTauriStatus } from './services/tauri'
import { logger } from './services/logger'
import ArkLayout from './components/ArkLayout'
import PrimaryNav from './components/PrimaryNav'
import SubNav from './components/SubNav'
import ActionBar from './components/ActionBar'
import LogsViewer from './components/LogsViewer'
import { useConfigStore } from './stores/configStore'
import { useUiStore } from './stores/uiStore'
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
  const { config, setConfig, isSaving, setSaving } = useConfigStore()
  const { primaryTab, setPrimaryTab, gameRulesSubTab, advancedSubTab, modSettingsSubTab } = useUiStore()

  useEffect(() => {
    logger.info('App component mounted')
    initAppAndConfig()
  }, [])

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
      setError(`Failed to initialize: ${errorMsg}`)
      setLoading(false)
    }
  }

  const loadConfig = async () => {
    try {
      logger.info('Loading default config...')
      setLoading(true)
      const defaultConfig: ServerConfig = await invoke('get_default_config')
      logger.info('Config loaded successfully', defaultConfig)
      setConfig(defaultConfig)
      setError(null)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to load config', err)
      setError(`Failed to load config: ${errorMsg}`)
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
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to save config', err)
      setError(`Failed to save: ${errorMsg}`)
    } finally {
      setSaving(false)
    }
  }

  const handleReset = async () => {
    if (confirm('Reset to defaults?')) {
      try {
        const defaults = await invoke('get_default_config')
        setConfig(defaults)
      } catch (err) {
        setError(`Failed to load defaults: ${err}`)
      }
    }
  }

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

      <ActionBar onSave={handleSave} onReset={handleReset} isSaving={isSaving} />
      <LogsViewer />
    </ArkLayout>
  )
}

export default App
