import { invoke } from '../services/tauri'
import { useState } from 'react'
import { useConfigStore } from '../stores/configStore'
import type { ServerConfig, PrimaryTab } from '../types'
import GeneralSettings from '../pages/GeneralSettings'
import GameplaySettings from '../pages/GameplaySettings'
import ServerSettings from '../pages/ServerSettings'
import AdvancedSettings from '../pages/AdvancedSettings'

interface ConfigFormProps {
  config: ServerConfig
  activeTab: PrimaryTab
  onConfigChange: (config: ServerConfig) => void
}

export default function ConfigForm({
  config,
  activeTab,
  onConfigChange,
}: ConfigFormProps) {
  const { setSaving } = useConfigStore()
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const handleSave = async () => {
    try {
      setSaving(true)
      setError(null)

      // Validate
      const validation = await invoke('validate_config', { config })
      console.log('Validation result:', validation)

      // Save
      await invoke('save_config', {
        config,
        configPath: 'config.toml',
      })

      setSuccess(true)
      setTimeout(() => setSuccess(false), 3000)
    } catch (err) {
      setError(`Failed to save config: ${err}`)
    } finally {
      setSaving(false)
    }
  }

  const handleReset = async () => {
    if (confirm('Reset to defaults?')) {
      try {
        const defaults = await invoke('get_default_config')
        onConfigChange(defaults)
      } catch (err) {
        setError(`Failed to load defaults: ${err}`)
      }
    }
  }

  return (
    <div className="space-y-6">
      {error && (
        <div className="bg-ark-accent/20 border-l-4 border-ark-accent p-4 text-ark-accent rounded">
          {error}
        </div>
      )}

      {success && (
        <div className="bg-ark-cyan/20 border-l-4 border-ark-cyan p-4 text-ark-cyan rounded">
          Configuration saved successfully!
        </div>
      )}

      <div className="bg-ark-secondary p-6 rounded-lg border border-ark-cyan/30">
        {activeTab === 'arks' && (
          <GeneralSettings config={config} onConfigChange={onConfigChange} />
        )}
        {activeTab === 'game_rules' && (
          <GameplaySettings config={config} onConfigChange={onConfigChange} />
        )}
        {activeTab === 'mod_settings' && (
          <ServerSettings config={config} onConfigChange={onConfigChange} />
        )}
        {activeTab === 'advanced' && (
          <AdvancedSettings config={config} onConfigChange={onConfigChange} />
        )}
        {activeTab === 'engrams' && (
          <div className="text-ark-cyan text-center py-8">ENGRAMS tab coming soon</div>
        )}
      </div>

      <div className="flex gap-4 justify-end">
        <button
          onClick={handleReset}
          className="px-6 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 transition"
        >
          Reset to Defaults
        </button>
        <button
          onClick={handleSave}
          className="px-6 py-2 bg-ark-cyan text-ark-dark rounded font-semibold hover:bg-ark-cyan/80 transition shadow-ark"
        >
          Save Configuration
        </button>
      </div>
    </div>
  )
}
