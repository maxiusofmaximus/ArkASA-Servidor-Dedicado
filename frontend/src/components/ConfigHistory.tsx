import { useState } from 'react'
import type { ServerConfig } from '../types'

interface ConfigSnapshot {
  version: number
  timestamp: string
  changes: string
  description?: string
}

interface ConfigHistoryProps {
  currentConfig: ServerConfig
}

export default function ConfigHistory({ currentConfig }: ConfigHistoryProps) {
  // Mock history data - en producción vendría de la base de datos
  const [history] = useState<ConfigSnapshot[]>([
    {
      version: 3,
      timestamp: '2026-06-11 10:30',
      changes: 'Max players: 70 → 100, XP multiplier: 3 → 5',
      description: 'Increased population and XP for faster progression',
    },
    {
      version: 2,
      timestamp: '2026-06-10 15:45',
      changes: 'Added 3 mods, set PvE mode, adjusted dino spawn',
      description: 'Initial setup with quality mods',
    },
    {
      version: 1,
      timestamp: '2026-06-10 09:00',
      changes: 'Initial configuration with defaults',
      description: 'Fresh installation',
    },
  ])

  const [selectedVersion, setSelectedVersion] = useState<number | null>(null)

  return (
    <div className="bg-ark-secondary border border-ark-cyan/30 rounded-lg p-6">
      <h3 className="text-xl font-bold text-ark-cyan mb-6">Configuration History</h3>

      <div className="space-y-3 max-h-96 overflow-y-auto">
        {history.map((snapshot) => (
          <div
            key={snapshot.version}
            onClick={() => setSelectedVersion(selectedVersion === snapshot.version ? null : snapshot.version)}
            className={`p-4 rounded cursor-pointer transition ${
              selectedVersion === snapshot.version
                ? 'bg-ark-cyan/20 border border-ark-cyan'
                : 'bg-ark-dark border border-ark-cyan/20 hover:border-ark-cyan/40'
            }`}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 bg-ark-cyan/20 rounded flex items-center justify-center text-ark-cyan font-semibold text-sm">
                  v{snapshot.version}
                </div>
                <div>
                  <p className="font-semibold text-white">{snapshot.description}</p>
                  <p className="text-xs text-gray-400">{snapshot.timestamp}</p>
                </div>
              </div>
              <span className="text-ark-cyan text-sm">→</span>
            </div>

            {selectedVersion === snapshot.version && (
              <div className="mt-4 pt-4 border-t border-ark-cyan/20">
                <p className="text-sm text-gray-300 mb-3">
                  <strong>Changes:</strong>
                </p>
                <p className="text-sm text-gray-400">{snapshot.changes}</p>

                <div className="mt-4 flex gap-2">
                  <button className="px-3 py-1 bg-ark-cyan/20 text-ark-cyan rounded text-sm hover:bg-ark-cyan/30 transition">
                    Restore
                  </button>
                  <button className="px-3 py-1 bg-ark-purple/20 text-ark-purple rounded text-sm hover:bg-ark-purple/30 transition">
                    Compare
                  </button>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      <div className="mt-6 pt-6 border-t border-ark-cyan/20">
        <div className="bg-ark-dark p-4 rounded">
          <p className="text-sm text-gray-400 mb-2">💾 Current version: <strong>v{history.length}</strong></p>
          <p className="text-xs text-gray-500">All changes are automatically backed up to database</p>
        </div>
      </div>
    </div>
  )
}
