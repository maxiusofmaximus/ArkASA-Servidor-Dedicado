import { useEffect, useState } from 'react'
import { logger } from '../services/logger'

// Only renders in dev mode (Vite exposes import.meta.env.DEV)
const IS_DEV = import.meta.env.DEV

export default function LogsViewer() {
  const [logs, setLogs] = useState<string>('')
  const [isOpen, setIsOpen] = useState(false)

  useEffect(() => {
    if (!IS_DEV) return
    const interval = setInterval(() => {
      setLogs(logger.getLogsAsText())
    }, 500)
    return () => clearInterval(interval)
  }, [])

  // Hidden entirely in production builds
  if (!IS_DEV) return null

  return (
    <div className="fixed bottom-20 right-4 z-50">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="opacity-30 hover:opacity-80 transition-opacity ark-action-btn text-[9px] px-2 py-0.5"
      >
        {isOpen ? 'HIDE LOGS' : 'LOGS'}
      </button>

      {isOpen && (
        <div className="absolute bottom-10 right-0 w-96 bg-slate-900/95 border border-ark-cyan/40 rounded shadow-xl">
          <div className="bg-ark-secondary px-4 py-2 font-bold flex justify-between items-center text-ark-cyan text-xs tracking-widest">
            <span>APP LOGS</span>
            <button
              onClick={() => { logger.clearLogs(); setLogs('') }}
              className="text-[10px] ark-action-btn px-2 py-0.5"
            >
              CLEAR
            </button>
          </div>
          <div className="bg-slate-950 p-3 h-64 overflow-y-auto font-mono text-xs text-ark-cyan/70">
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
