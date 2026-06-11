import { useEffect, useState } from 'react'
import { logger } from '../services/logger'

export default function LogsViewer() {
  const [logs, setLogs] = useState<string>('')
  const [isOpen, setIsOpen] = useState(false)

  useEffect(() => {
    // Update logs every 500ms
    const interval = setInterval(() => {
      setLogs(logger.getLogsAsText())
    }, 500)

    return () => clearInterval(interval)
  }, [])

  return (
    <div className="fixed bottom-4 right-4 z-50">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="bg-cyan-600 hover:bg-cyan-700 text-white px-4 py-2 rounded font-mono text-sm"
      >
        {isOpen ? 'Hide Logs' : 'Show Logs'}
      </button>

      {isOpen && (
        <div className="absolute bottom-12 right-0 w-96 bg-slate-900 border border-cyan-500 rounded shadow-xl">
          <div className="bg-cyan-600 text-white px-4 py-2 font-bold flex justify-between items-center">
            <span>Application Logs</span>
            <button
              onClick={() => {
                logger.clearLogs()
                setLogs('')
              }}
              className="text-xs bg-red-600 hover:bg-red-700 px-2 py-1 rounded"
            >
              Clear
            </button>
          </div>
          <div className="bg-slate-950 p-3 h-64 overflow-y-auto font-mono text-xs text-cyan-400">
            {logs ? (
              <pre className="whitespace-pre-wrap break-words">{logs}</pre>
            ) : (
              <div className="text-slate-500">No logs yet...</div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
