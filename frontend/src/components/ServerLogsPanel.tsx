import { useState, useEffect, useRef } from 'react'
import { invoke } from '../services/tauri'

interface ServerLogsPanelProps {
  serverDir: string
  maps: string[]
  onClose: () => void
}

export default function ServerLogsPanel({ serverDir, maps, onClose }: ServerLogsPanelProps) {
  const [selectedMap, setSelectedMap] = useState(maps[0] ?? 'TheIsland_WP')
  const [lines, setLines] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)
  const [autoRefresh, setAutoRefresh] = useState(true)
  const [filterText, setFilterText] = useState('')
  const bottomRef = useRef<HTMLDivElement>(null)

  const loadLogs = async (map?: string) => {
    try {
      const result = await invoke<string[]>('read_server_log', {
        serverDir,
        map: map ?? selectedMap,
        lines: 200,
      })
      setLines(result)
      setError(null)
    } catch (e) {
      setError(String(e))
    }
  }

  useEffect(() => {
    setLines([])
    setError(null)
    loadLogs(selectedMap)
  }, [serverDir, selectedMap]) // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!autoRefresh) return
    const interval = setInterval(() => loadLogs(), 3000)
    return () => clearInterval(interval)
  }, [autoRefresh, serverDir, selectedMap]) // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (autoRefresh) {
      bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
    }
  }, [lines, autoRefresh])

  const getLineColor = (line: string) => {
    const l = line.toLowerCase()
    if (l.includes('error') || l.includes('fatal') || l.includes('crash')) return 'text-red-400/80'
    if (l.includes('warn')) return 'text-yellow-400/70'
    if (l.includes('join') || l.includes('connected') || l.includes('player')) return 'text-green-400/70'
    return 'text-ark-cyan/50'
  }

  const filtered = filterText
    ? lines.filter((l) => l.toLowerCase().includes(filterText.toLowerCase()))
    : lines

  const MAP_LABELS: Record<string, string> = {
    TheIsland_WP: 'The Island',
    Ragnarok_WP: 'Ragnarok',
    TheCenter_WP: 'The Center',
    ScorchedEarth_WP: 'Scorched Earth',
    Aberration_WP: 'Aberration',
    Extinction_WP: 'Extinction',
    Valguero_WP: 'Valguero',
    Genesis_WP: 'Genesis',
    CrystalIsles_WP: 'Crystal Isles',
    Gen2_WP: 'Genesis 2',
    LostIsland_WP: 'Lost Island',
    Fjordur_WP: 'Fjordur',
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-end justify-stretch"
      style={{ background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(2px)' }}
    >
      <div
        className="w-full max-h-[55vh] flex flex-col ark-panel"
        style={{ borderTop: '1px solid rgba(0,200,255,0.3)' }}
      >
        {/* Toolbar */}
        <div className="flex items-center gap-3 px-4 py-2 border-b border-ark-cyan/20 flex-shrink-0">
          <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase flex-shrink-0">
            📋 ShooterGame.log
          </span>

          {/* Map selector — only shown when multiple maps are configured */}
          {maps.length > 1 && (
            <select
              value={selectedMap}
              onChange={(e) => setSelectedMap(e.target.value)}
              className="bg-ark-secondary/60 border border-ark-cyan/30 text-ark-cyan/80 text-xs px-2 py-1 rounded focus:outline-none focus:border-ark-cyan/60"
            >
              {maps.map((m) => (
                <option key={m} value={m}>
                  {MAP_LABELS[m] ?? m}
                </option>
              ))}
            </select>
          )}

          <input
            type="text"
            value={filterText}
            onChange={(e) => setFilterText(e.target.value)}
            placeholder="Filtrar..."
            className="flex-1 bg-transparent border border-ark-cyan/20 text-ark-cyan/80 text-xs px-2 py-1 rounded focus:outline-none focus:border-ark-cyan/50 placeholder-ark-cyan/20 font-mono"
          />
          <label className="flex items-center gap-1.5 text-xs text-ark-cyan/50 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              className="accent-ark-cyan"
            />
            Auto-refresh
          </label>
          <button
            onClick={() => loadLogs()}
            className="ark-action-btn text-[10px] px-2 py-0.5"
          >
            ↻
          </button>
          <button
            onClick={onClose}
            className="ark-action-btn text-[10px] px-3 py-0.5"
            style={{ color: 'rgba(239,68,68,0.7)' }}
          >
            ✕ CERRAR
          </button>
        </div>

        {/* Log content */}
        <div className="flex-1 overflow-y-auto font-mono text-[11px] leading-relaxed px-4 py-2">
          {error ? (
            <p className="text-red-400/70 py-4">{error}</p>
          ) : filtered.length === 0 ? (
            <p className="text-ark-cyan/30 py-4">Sin líneas de log...</p>
          ) : (
            filtered.map((line, i) => (
              <div key={i} className={`py-px ${getLineColor(line)}`}>
                {line}
              </div>
            ))
          )}
          <div ref={bottomRef} />
        </div>
      </div>
    </div>
  )
}
