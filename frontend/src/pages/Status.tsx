import { useState } from 'react'
import type { ServerConfig } from '../types'
import ConfigPreview from '../components/ConfigPreview'
import ConfigHistory from '../components/ConfigHistory'
import ServerLogs from '../components/ServerLogs'

interface StatusProps {
  config: ServerConfig
}

export default function Status({ config }: StatusProps) {
  const [activePanel, setActivePanel] = useState<'preview' | 'history' | 'logs'>('preview')

  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-bold text-ark-cyan">Server Dashboard</h3>

      {/* Panel Selector */}
      <div className="flex gap-2">
        {(
          [
            { id: 'preview', label: '📋 Config Preview', icon: '📋' },
            { id: 'history', label: '📜 Config History', icon: '📜' },
            { id: 'logs', label: '📝 Server Logs', icon: '📝' },
          ] as const
        ).map((panel) => (
          <button
            key={panel.id}
            onClick={() => setActivePanel(panel.id)}
            className={`px-6 py-3 rounded font-semibold transition ${
              activePanel === panel.id
                ? 'bg-ark-cyan text-ark-dark shadow-ark'
                : 'bg-ark-dark border border-ark-cyan/30 text-ark-cyan hover:border-ark-cyan/60'
            }`}
          >
            {panel.icon} {panel.label.split(' ')[1]}
          </button>
        ))}
      </div>

      {/* Panel Content */}
      <div className="bg-ark-secondary border border-ark-cyan/30 rounded-lg p-6">
        {activePanel === 'preview' && <ConfigPreview config={config} />}
        {activePanel === 'history' && <ConfigHistory currentConfig={config} />}
        {activePanel === 'logs' && <ServerLogs />}
      </div>

      {/* Info Cards */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-ark-secondary border border-ark-cyan/20 rounded p-4">
          <p className="text-xs text-gray-400 mb-1">Configuration Files</p>
          <p className="text-lg font-bold text-ark-cyan">3</p>
          <p className="text-xs text-gray-500 mt-1">Generated automatically</p>
        </div>

        <div className="bg-ark-secondary border border-ark-cyan/20 rounded p-4">
          <p className="text-xs text-gray-400 mb-1">Active Mods</p>
          <p className="text-lg font-bold text-ark-cyan">{config.mods.active_mods.length}</p>
          <p className="text-xs text-gray-500 mt-1">Configured</p>
        </div>

        <div className="bg-ark-secondary border border-ark-cyan/20 rounded p-4">
          <p className="text-xs text-gray-400 mb-1">Max Players</p>
          <p className="text-lg font-bold text-ark-cyan">{config.gameplay.max_players}</p>
          <p className="text-xs text-gray-500 mt-1">Capacity</p>
        </div>
      </div>

      {/* Important Notes */}
      <div className="bg-ark-accent/10 border border-ark-accent/30 rounded p-4">
        <p className="text-sm text-ark-accent">
          ⚠️ <strong>Important:</strong> Some changes require a server restart to take effect. Configuration preview shows what will be generated.
        </p>
      </div>
    </div>
  )
}
