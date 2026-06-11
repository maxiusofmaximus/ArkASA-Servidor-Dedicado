import { useEffect, useState } from 'react'
import { initializeTauri, invoke, getTauriStatus } from './services/tauri'
import { logger } from './services/logger'
import Navigation from './components/Navigation'
import ConfigForm from './components/ConfigForm'
import ServerStatus from './components/ServerStatus'
import StatusPage from './pages/Status'
import LogsViewer from './components/LogsViewer'
import { useConfigStore } from './stores/configStore'
import type { ServerConfig, PrimaryTab } from './types'

function App() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<PrimaryTab>('arks')
  const [tauriStatus, setTauriStatus] = useState<string>('')
  const { config, setConfig } = useConfigStore()

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

  if (loading) {
    return (
      <div className="min-h-screen bg-ark-dark flex items-center justify-center">
        <div className="text-ark-cyan text-2xl">Loading configuration...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen bg-ark-dark flex items-center justify-center">
        <div className="text-ark-accent text-xl">{error}</div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-ark-dark text-white">
      <Navigation activeTab={activeTab} onTabChange={setActiveTab} />

      <div className="flex">
        <ServerStatus />

        <main className="flex-1 p-8">
          {config && (
            <ConfigForm
              config={config}
              activeTab={activeTab}
              onConfigChange={setConfig}
            />
          )}
        </main>
      </div>

      <LogsViewer />
    </div>
  )
}

export default App
