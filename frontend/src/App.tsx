import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import Navigation from './components/Navigation'
import ConfigForm from './components/ConfigForm'
import ServerStatus from './components/ServerStatus'
import StatusPage from './pages/Status'
import { useConfigStore } from './stores/configStore'
import type { ServerConfig, Tab } from './types'

function App() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<Tab>('general')
  const { config, setConfig } = useConfigStore()

  useEffect(() => {
    loadConfig()
  }, [])

  const loadConfig = async () => {
    try {
      setLoading(true)
      const defaultConfig: ServerConfig = await invoke('get_default_config')
      setConfig(defaultConfig)
    } catch (err) {
      setError(`Failed to load config: ${err}`)
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
            <>
              {activeTab === 'status' ? (
                <StatusPage config={config} />
              ) : (
                <ConfigForm
                  config={config}
                  activeTab={activeTab}
                  onConfigChange={setConfig}
                />
              )}
            </>
          )}
        </main>
      </div>
    </div>
  )
}

export default App
