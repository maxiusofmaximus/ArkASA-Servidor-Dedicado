import { useCallback, useEffect, useRef, useState } from 'react'
import { invoke } from '../../services/tauri'
import { logger } from '../../services/logger'
import type { ServerConfig, MapInstanceStatus } from '../../types'

interface Metrics {
  running?: boolean
  process_id?: number | null
  memory_mb?: number | null
  fps?: number | null
  udp_conns?: number | null
  session_name?: string
  max_players?: number
  mods_count?: number
  maps_total?: number
  game_port?: number
  query_port?: number
  rcon_port?: number
  server_platform?: string
  server_dir?: string
  timestamp_ms?: number
}

interface Sample { t: number; mem: number | null; fps: number | null }

const POLL_MS = 5000
const MAX_SAMPLES = 60

function Card({ label, value, sub, accent = false }: {
  label: string
  value: string
  sub?: string
  accent?: boolean
}) {
  return (
    <div className={`ark-panel rounded border p-4 ${accent ? 'border-ark-cyan/60' : 'border-ark-cyan/25'}`}>
      <div className="text-[10px] uppercase tracking-[0.2em] text-ark-cyan/60">{label}</div>
      <div className={`mt-1 text-2xl font-bold ${accent ? 'text-ark-cyan' : 'text-white/90'}`}>{value}</div>
      {sub && <div className="mt-0.5 text-[11px] text-white/45">{sub}</div>}
    </div>
  )
}

/** Tiny inline sparkline — no chart dependency. */
function Sparkline({ samples, stroke }: { samples: (number | null)[]; stroke: string }) {
  const pts = samples.filter((v): v is number => v !== null)
  if (pts.length < 2) {
    return <div className="h-16 flex items-center justify-center text-[11px] text-white/30">Reuniendo datos…</div>
  }
  const w = 300, h = 56
  const min = Math.min(...pts), max = Math.max(...pts)
  const span = max - min || 1
  const step = w / (pts.length - 1)
  const d = pts.map((v, i) => `${i === 0 ? 'M' : 'L'} ${(i * step).toFixed(1)} ${(h - ((v - min) / span) * (h - 6) - 3).toFixed(1)}`).join(' ')
  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="w-full h-16" preserveAspectRatio="none">
      <path d={d} fill="none" stroke={stroke} strokeWidth="2" />
    </svg>
  )
}

export default function AnalyticsTab({ config }: { config: ServerConfig }) {
  const [metrics, setMetrics] = useState<Metrics | null>(null)
  const [maps, setMaps] = useState<MapInstanceStatus[]>([])
  const [history, setHistory] = useState<Sample[]>([])
  const [err, setErr] = useState<string | null>(null)
  const timer = useRef<number | null>(null)

  const poll = useCallback(async () => {
    try {
      const [m, statuses] = await Promise.all([
        invoke<Metrics>('get_server_metrics'),
        invoke<MapInstanceStatus[]>('get_cluster_instance_status', { config }),
      ])
      setMetrics(m)
      setMaps(statuses ?? [])
      setErr(null)
      setHistory((prev) => {
        const next = [...prev, {
          t: m?.timestamp_ms ?? Date.now(),
          mem: typeof m?.memory_mb === 'number' ? m.memory_mb : null,
          fps: typeof m?.fps === 'number' ? m.fps : null,
        }]
        return next.slice(-MAX_SAMPLES)
      })
    } catch (e) {
      logger.warn('analytics poll failed', e)
      setErr(e instanceof Error ? e.message : String(e))
    }
  }, [config])

  useEffect(() => {
    void poll()
    timer.current = window.setInterval(() => void poll(), POLL_MS)
    return () => { if (timer.current) window.clearInterval(timer.current) }
  }, [poll])

  const running = metrics?.running ?? false
  const runningMaps = maps.filter((m) => m.running).length

  return (
    <div className="px-6 py-4 space-y-5">
      <div className="flex items-center justify-between">
        <h2 className="text-ark-cyan font-bold tracking-[0.2em] uppercase text-sm">Rendimiento</h2>
        <span className="text-[11px] text-white/40">
          {metrics?.timestamp_ms ? `Actualizado ${new Date(metrics.timestamp_ms).toLocaleTimeString()}` : '—'} · cada {POLL_MS / 1000}s
        </span>
      </div>

      {err && (
        <div className="border border-ark-accent/50 text-ark-accent text-[12px] rounded p-2">
          No se pudieron leer las métricas: {err}
        </div>
      )}

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Card label="Estado" value={running ? 'RUNNING' : 'STOPPED'} sub={metrics?.process_id ? `PID ${metrics.process_id}` : 'sin proceso'} accent={running} />
        <Card label="Memoria" value={typeof metrics?.memory_mb === 'number' ? `${metrics.memory_mb.toFixed(0)} MB` : '—'} sub="proceso ArkAscendedServer" />
        <Card label="FPS (servidor)" value={typeof metrics?.fps === 'number' ? String(metrics.fps) : '—'} sub="del log ShooterGame" />
        <Card label="Conexiones UDP" value={typeof metrics?.udp_conns === 'number' ? String(metrics.udp_conns) : '—'} sub="públicas del proceso" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="ark-panel rounded border border-ark-cyan/25 p-4">
          <div className="text-[10px] uppercase tracking-[0.2em] text-ark-cyan/60 mb-2">Memoria (últimos {MAX_SAMPLES * POLL_MS / 60000} min)</div>
          <Sparkline samples={history.map((s) => s.mem)} stroke="#00d4ff" />
          <div className="text-[10px] uppercase tracking-[0.2em] text-ark-cyan/60 mt-3 mb-2">FPS</div>
          <Sparkline samples={history.map((s) => s.fps)} stroke="#9d4edd" />
        </div>

        <div className="ark-panel rounded border border-ark-cyan/25 p-4">
          <div className="text-[10px] uppercase tracking-[0.2em] text-ark-cyan/60 mb-3">Instancias del cluster</div>
          <table className="w-full text-[12px]">
            <thead>
              <tr className="text-white/40 text-left">
                <th className="font-normal pb-1">Mapa</th>
                <th className="font-normal pb-1">Estado</th>
              </tr>
            </thead>
            <tbody>
              {maps.map((m) => (
                <tr key={m.map_index} className="border-t border-white/5">
                  <td className="py-1 text-white/80">[{m.map_index}] {m.map_label}</td>
                  <td className="py-1">
                    <span className={m.running ? 'text-ark-cyan' : 'text-white/40'}>
                      {m.running ? '● RUNNING' : '○ stopped'}
                    </span>
                  </td>
                </tr>
              ))}
              {maps.length === 0 && (
                <tr><td colSpan={2} className="py-2 text-white/30">Sin datos de instancias.</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      <div className="ark-panel rounded border border-ark-cyan/25 p-4">
        <div className="text-[10px] uppercase tracking-[0.2em] text-ark-cyan/60 mb-3">Configuración del perfil activo</div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-y-2 gap-x-6 text-[12px]">
          <div><span className="text-white/40">Sesión</span><div className="text-white/85">{metrics?.session_name ?? config.identification.session_name}</div></div>
          <div><span className="text-white/40">Máx. jugadores</span><div className="text-white/85">{metrics?.max_players ?? config.gameplay.max_players}</div></div>
          <div><span className="text-white/40">Mods activos</span><div className="text-white/85">{metrics?.mods_count ?? config.mods.active_mods.length}</div></div>
          <div><span className="text-white/40">Mapas</span><div className="text-white/85">{runningMaps} / {metrics?.maps_total ?? config.cluster_maps.length}</div></div>
          <div><span className="text-white/40">Puerto juego</span><div className="text-white/85">{metrics?.game_port ?? config.network.port}</div></div>
          <div><span className="text-white/40">Query</span><div className="text-white/85">{metrics?.query_port ?? config.network.query_port}</div></div>
          <div><span className="text-white/40">RCON</span><div className="text-white/85">{metrics?.rcon_port ?? config.network.rcon_port}</div></div>
          <div><span className="text-white/40">Plataforma</span><div className="text-white/85">{metrics?.server_platform ?? config.network.server_platform}</div></div>
          <div className="col-span-2 md:col-span-4"><span className="text-white/40">Directorio</span><div className="text-white/60 truncate" title={metrics?.server_dir ?? config.paths.server_dir}>{metrics?.server_dir ?? config.paths.server_dir}</div></div>
        </div>
      </div>
    </div>
  )
}
