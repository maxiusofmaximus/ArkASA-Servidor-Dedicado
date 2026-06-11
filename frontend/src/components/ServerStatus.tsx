import { useState, useEffect } from 'react'

export default function ServerStatus() {
  const [status, setStatus] = useState({
    running: false,
    configValid: false,
    lastUpdate: new Date(),
  })

  useEffect(() => {
    // TODO: Implement actual server status checking
    setStatus({
      running: false,
      configValid: true,
      lastUpdate: new Date(),
    })
  }, [])

  return (
    <aside className="w-80 bg-ark-secondary border-r border-ark-cyan/30 p-6">
      <h2 className="text-xl font-bold text-ark-cyan mb-6">Server Status</h2>

      <div className="space-y-4">
        <div className="bg-ark-dark p-4 rounded border border-ark-cyan/20">
          <div className="flex items-center gap-3">
            <div
              className={`w-4 h-4 rounded-full ${
                status.running ? 'bg-green-500' : 'bg-red-500'
              }`}
            />
            <span className="text-sm">
              Server: {status.running ? 'Running' : 'Offline'}
            </span>
          </div>
        </div>

        <div className="bg-ark-dark p-4 rounded border border-ark-cyan/20">
          <div className="flex items-center gap-3">
            <div className={`w-4 h-4 rounded-full bg-${status.configValid ? 'green' : 'red'}-500`} />
            <span className="text-sm">
              Config: {status.configValid ? 'Valid' : 'Invalid'}
            </span>
          </div>
        </div>

        <div className="bg-ark-dark p-4 rounded border border-ark-cyan/20">
          <div className="text-xs text-gray-400">
            Last Update: {status.lastUpdate.toLocaleTimeString()}
          </div>
        </div>
      </div>

      <div className="mt-8 space-y-3">
        <button className="w-full px-4 py-2 bg-ark-cyan text-ark-dark rounded font-semibold hover:bg-ark-cyan/80 transition">
          Start Server
        </button>
        <button className="w-full px-4 py-2 bg-ark-purple text-white rounded font-semibold hover:bg-ark-purple/80 transition">
          Stop Server
        </button>
        <button className="w-full px-4 py-2 border border-ark-cyan text-ark-cyan rounded hover:bg-ark-cyan/10 transition">
          Restart
        </button>
      </div>
    </aside>
  )
}
