import { useState, useEffect } from 'react'

interface ServerStatusData {
  running: boolean
  configValid: boolean
  processId?: number
  uptime?: string
  lastUpdate: Date
  playerCount?: number
  fps?: number
}

export default function ServerStatus() {
  const [status, setStatus] = useState<ServerStatusData>({
    running: false,
    configValid: true,
    lastUpdate: new Date(),
    playerCount: 0,
    fps: 60,
  })

  const [loading, setLoading] = useState(false)

  useEffect(() => {
    // Simulate status updates
    const interval = setInterval(() => {
      setStatus((prev) => ({
        ...prev,
        lastUpdate: new Date(),
        uptime: prev.running ? `${Math.floor(Math.random() * 1440)} min` : undefined,
      }))
    }, 5000)

    return () => clearInterval(interval)
  }, [])

  const handleStart = async () => {
    setLoading(true)
    try {
      // TODO: Invoke Tauri command: start_server
      setStatus((prev) => ({
        ...prev,
        running: true,
        processId: Math.floor(Math.random() * 100000),
        lastUpdate: new Date(),
      }))
    } catch (err) {
      console.error('Failed to start server:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleStop = async () => {
    setLoading(true)
    try {
      // TODO: Invoke Tauri command: stop_server
      setStatus((prev) => ({
        ...prev,
        running: false,
        processId: undefined,
        lastUpdate: new Date(),
      }))
    } catch (err) {
      console.error('Failed to stop server:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleRestart = async () => {
    setLoading(true)
    try {
      // TODO: Invoke Tauri command: restart_server
      setStatus((prev) => ({
        ...prev,
        running: true,
        processId: Math.floor(Math.random() * 100000),
        lastUpdate: new Date(),
      }))
    } catch (err) {
      console.error('Failed to restart server:', err)
    } finally {
      setLoading(false)
    }
  }

  const getStatusColor = (running: boolean) => {
    return running ? 'from-green-600 to-green-700' : 'from-gray-600 to-gray-700'
  }

  return (
    <aside className="w-96 bg-ark-secondary border-r border-ark-cyan/30 p-6 overflow-y-auto max-h-screen">
      <h2 className="text-2xl font-bold text-ark-cyan mb-6">Server Status</h2>

      {/* Status Card */}
      <div className={`bg-gradient-to-br ${getStatusColor(status.running)} rounded-lg p-6 mb-6 text-white`}>
        <div className="flex items-center justify-between mb-4">
          <div>
            <p className="text-sm opacity-90">Server Status</p>
            <p className="text-2xl font-bold">{status.running ? 'RUNNING' : 'OFFLINE'}</p>
          </div>
          <div className={`w-16 h-16 rounded-full flex items-center justify-center ${status.running ? 'bg-green-500/30 animate-pulse' : 'bg-gray-500/30'}`}>
            <div className={`w-12 h-12 rounded-full ${status.running ? 'bg-green-500' : 'bg-gray-500'}`} />
          </div>
        </div>

        {status.running && (
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span>PID:</span>
              <span className="font-mono">{status.processId}</span>
            </div>
            <div className="flex justify-between">
              <span>Uptime:</span>
              <span className="font-mono">{status.uptime || '--'}</span>
            </div>
            <div className="flex justify-between">
              <span>Players:</span>
              <span className="font-mono">{status.playerCount}/70</span>
            </div>
            <div className="flex justify-between">
              <span>FPS:</span>
              <span className="font-mono">{status.fps} Hz</span>
            </div>
          </div>
        )}
      </div>

      {/* Config Status */}
      <div className="space-y-3 mb-6">
        <div className="bg-ark-dark p-4 rounded border border-ark-cyan/20 hover:border-ark-cyan/40 transition">
          <div className="flex items-center gap-3 mb-2">
            <div className={`w-3 h-3 rounded-full ${status.configValid ? 'bg-green-500' : 'bg-ark-accent'}`} />
            <span className="text-sm font-semibold">Configuration</span>
          </div>
          <p className="text-xs text-gray-400">{status.configValid ? 'Config is valid' : 'Config has errors'}</p>
        </div>

        <div className="bg-ark-dark p-4 rounded border border-ark-cyan/20">
          <div className="text-xs text-gray-400 space-y-1">
            <p>Last updated: {status.lastUpdate.toLocaleTimeString()}</p>
            <p>Status auto-refreshes every 5 seconds</p>
          </div>
        </div>
      </div>

      {/* Control Buttons */}
      <div className="space-y-3 mb-6">
        <button
          onClick={handleStart}
          disabled={loading || status.running}
          className="w-full px-4 py-3 bg-gradient-to-r from-green-600 to-green-700 text-white rounded font-semibold hover:from-green-700 hover:to-green-800 disabled:opacity-50 disabled:cursor-not-allowed transition transform hover:scale-105 active:scale-95"
        >
          {loading ? '⏳ Loading...' : status.running ? '✓ Running' : '▶ Start Server'}
        </button>

        <button
          onClick={handleRestart}
          disabled={loading || !status.running}
          className="w-full px-4 py-3 bg-gradient-to-r from-ark-purple to-ark-purple/80 text-white rounded font-semibold hover:from-ark-purple/90 hover:to-ark-purple disabled:opacity-50 disabled:cursor-not-allowed transition transform hover:scale-105 active:scale-95"
        >
          {loading ? '⏳ Loading...' : '🔄 Restart Server'}
        </button>

        <button
          onClick={handleStop}
          disabled={loading || !status.running}
          className="w-full px-4 py-3 bg-gradient-to-r from-red-600 to-red-700 text-white rounded font-semibold hover:from-red-700 hover:to-red-800 disabled:opacity-50 disabled:cursor-not-allowed transition transform hover:scale-105 active:scale-95"
        >
          {loading ? '⏳ Loading...' : '⏹ Stop Server'}
        </button>
      </div>

      {/* Quick Actions */}
      <div className="border-t border-ark-cyan/20 pt-6">
        <p className="text-xs font-semibold text-gray-400 mb-3 uppercase">Quick Actions</p>
        <div className="grid grid-cols-2 gap-3">
          <button className="px-3 py-2 bg-ark-dark border border-ark-cyan/30 text-ark-cyan rounded text-sm hover:bg-ark-cyan/10 transition">
            📊 Metrics
          </button>
          <button className="px-3 py-2 bg-ark-dark border border-ark-cyan/30 text-ark-cyan rounded text-sm hover:bg-ark-cyan/10 transition">
            👥 Players
          </button>
          <button className="px-3 py-2 bg-ark-dark border border-ark-cyan/30 text-ark-cyan rounded text-sm hover:bg-ark-cyan/10 transition">
            📁 Backup
          </button>
          <button className="px-3 py-2 bg-ark-dark border border-ark-cyan/30 text-ark-cyan rounded text-sm hover:bg-ark-cyan/10 transition">
            ⚙️ Settings
          </button>
        </div>
      </div>

      {/* Info Footer */}
      <div className="mt-6 pt-6 border-t border-ark-cyan/20">
        <div className="text-xs text-gray-500 space-y-1">
          <p>💡 Pro tip: Server auto-saves every 15 minutes</p>
          <p>🎯 Players can join on port {7777}</p>
        </div>
      </div>
    </aside>
  )
}
