import { useState } from 'react'

interface LogEntry {
  timestamp: string
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG'
  message: string
}

export default function ServerLogs() {
  const [logs] = useState<LogEntry[]>([
    {
      timestamp: '2026-06-11 10:45:23',
      level: 'INFO',
      message: 'ARK Server initialized',
    },
    {
      timestamp: '2026-06-11 10:45:25',
      level: 'INFO',
      message: 'Loading 11 mods from ActiveMods list',
    },
    {
      timestamp: '2026-06-11 10:45:30',
      level: 'INFO',
      message: 'Mod 955131 loaded successfully',
    },
    {
      timestamp: '2026-06-11 10:45:31',
      level: 'INFO',
      message: 'Mod 1102729 loaded successfully',
    },
    {
      timestamp: '2026-06-11 10:45:35',
      level: 'INFO',
      message: 'All mods loaded (LoadGameMods with 11 mods)',
    },
    {
      timestamp: '2026-06-11 10:45:40',
      level: 'INFO',
      message: 'Server ready on port 7777',
    },
    {
      timestamp: '2026-06-11 10:46:12',
      level: 'INFO',
      message: 'Player joined: MaxiusOfMaximus',
    },
    {
      timestamp: '2026-06-11 10:47:15',
      level: 'WARN',
      message: 'High CPU usage detected (85%)',
    },
  ])
  const [filter, setFilter] = useState<'ALL' | 'INFO' | 'WARN' | 'ERROR' | 'DEBUG'>('ALL')
  const [autoScroll, setAutoScroll] = useState(true)

  const filteredLogs = filter === 'ALL' ? logs : logs.filter((log) => log.level === filter)

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'INFO':
        return 'text-ark-cyan'
      case 'WARN':
        return 'text-yellow-400'
      case 'ERROR':
        return 'text-ark-accent'
      case 'DEBUG':
        return 'text-gray-400'
      default:
        return 'text-white'
    }
  }

  const getLevelBg = (level: string) => {
    switch (level) {
      case 'INFO':
        return 'bg-ark-cyan/10'
      case 'WARN':
        return 'bg-yellow-500/10'
      case 'ERROR':
        return 'bg-ark-accent/10'
      case 'DEBUG':
        return 'bg-gray-500/10'
      default:
        return 'bg-ark-dark'
    }
  }

  return (
    <div className="bg-ark-secondary border border-ark-cyan/30 rounded-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-bold text-ark-cyan">Server Logs</h3>
        <div className="flex gap-2">
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="w-4 h-4"
            />
            <span className="text-gray-300">Auto-scroll</span>
          </label>
        </div>
      </div>

      <div className="flex gap-2 mb-4 flex-wrap">
        {(['ALL', 'INFO', 'WARN', 'ERROR', 'DEBUG'] as const).map((level) => (
          <button
            key={level}
            onClick={() => setFilter(level)}
            className={`px-3 py-1 rounded text-sm transition ${
              filter === level
                ? 'bg-ark-cyan text-ark-dark font-semibold'
                : 'bg-ark-dark border border-ark-cyan/30 text-gray-300 hover:bg-ark-cyan/10'
            }`}
          >
            {level}
          </button>
        ))}
      </div>

      <div className="bg-ark-dark rounded p-4 font-mono text-xs max-h-96 overflow-y-auto space-y-2">
        {filteredLogs.length > 0 ? (
          filteredLogs.map((log, idx) => (
            <div key={idx} className={`flex gap-3 p-2 rounded ${getLevelBg(log.level)}`}>
              <span className="text-gray-500 min-w-fit">{log.timestamp}</span>
              <span className={`font-semibold min-w-fit ${getLevelColor(log.level)}`}>[{log.level}]</span>
              <span className="text-gray-300">{log.message}</span>
            </div>
          ))
        ) : (
          <div className="text-gray-500 text-center py-8">No logs for this filter</div>
        )}
      </div>

      <div className="mt-4 flex gap-2">
        <button className="flex-1 px-4 py-2 bg-ark-dark border border-ark-cyan/30 text-ark-cyan rounded hover:bg-ark-cyan/10 transition text-sm">
          📁 Open Log File
        </button>
        <button className="flex-1 px-4 py-2 bg-ark-dark border border-ark-cyan/30 text-ark-cyan rounded hover:bg-ark-cyan/10 transition text-sm">
          🔄 Refresh
        </button>
        <button className="flex-1 px-4 py-2 bg-ark-dark border border-ark-cyan/30 text-ark-cyan rounded hover:bg-ark-cyan/10 transition text-sm">
          📋 Export
        </button>
      </div>
    </div>
  )
}
