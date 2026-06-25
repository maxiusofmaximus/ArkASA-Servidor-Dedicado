import { useState } from 'react'
import { invoke } from '../../services/tauri'
import type { ServerConfig, ConnectionMethod, NetworkConfig, DetectedIps } from '../../types'
import { useI18n } from '../../i18n/useI18n'

interface ConnectionManagerProps {
  config: ServerConfig
  updateNetwork: (field: keyof NetworkConfig, value: string) => void
}

const METHOD_ORDER: ConnectionMethod[] = ['tailscale', 'public', 'duckdns', 'local', 'manual']
const METHOD_FIELDS: Record<ConnectionMethod, keyof Pick<NetworkConfig, 'tailscale_ip' | 'public_ip' | 'duckdns_host' | 'local_ip' | 'manual_ip'>> = {
  tailscale: 'tailscale_ip',
  public:    'public_ip',
  duckdns:   'duckdns_host',
  local:     'local_ip',
  manual:    'manual_ip',
}
const METHOD_PLACEHOLDERS: Record<ConnectionMethod, string> = {
  tailscale: '100.x.x.x',
  public:    '181.237.x.x',
  duckdns:   'ark-max.duckdns.org',
  local:     '192.168.x.x',
  manual:    '...',
}
const METHOD_LABELS: Record<ConnectionMethod, string> = {
  tailscale: 'Tailscale',
  public:    'Public IP',
  duckdns:   'DuckDNS',
  local:     'Local IP',
  manual:    'Manual',
}

function effectiveIp(n: NetworkConfig): string {
  const map: Record<ConnectionMethod, string> = {
    tailscale: n.tailscale_ip ?? '',
    public:    n.public_ip ?? '',
    duckdns:   n.duckdns_host ?? '',
    local:     n.local_ip ?? '',
    manual:    n.manual_ip ?? '',
  }
  return (map[n.connection_method] ?? '').trim() || (n.server_ip ?? '').trim()
}

export default function ConnectionManager({ config, updateNetwork }: ConnectionManagerProps) {
  const net = config.network
  const method = net.connection_method ?? 'manual'
  const ip = effectiveIp(net)
  const { tk } = useI18n()

  const [detecting, setDetecting] = useState(false)
  const [detectMsg, setDetectMsg] = useState<string | null>(null)

  const handleDetect = async () => {
    setDetecting(true)
    setDetectMsg(null)
    try {
      const ips = await invoke<DetectedIps>('detect_ips')
      const found: string[] = []
      if (ips.public_ip)    { updateNetwork('public_ip',    ips.public_ip);    found.push(`${tk('public_ip_label', 'Public')}: ${ips.public_ip}`) }
      if (ips.tailscale_ip) { updateNetwork('tailscale_ip', ips.tailscale_ip); found.push(`Tailscale: ${ips.tailscale_ip}`) }
      if (ips.local_ip)     { updateNetwork('local_ip',     ips.local_ip);     found.push(`Local: ${ips.local_ip}`) }
      setDetectMsg(found.length > 0 ? found.join(' · ') : tk('no_ip_detected', 'No IPs detected automatically'))
    } catch (e) {
      setDetectMsg(`Error: ${String(e)}`)
    } finally {
      setDetecting(false)
    }
  }

  const hintKey: Record<ConnectionMethod, string> = {
    tailscale: 'method_tailscale_hint',
    public:    'method_public_hint',
    duckdns:   'method_duckdns_hint',
    local:     'method_local_hint',
    manual:    'method_manual_hint',
  }

  return (
    <div className="ark-panel rounded-lg p-4 space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between gap-2">
        <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase flex-shrink-0">
          {tk('server_connection', 'Server Connection')}
        </span>
        <div className="flex items-center gap-2 min-w-0">
          {ip ? (
            <span
              className="text-[10px] font-bold tracking-widest px-2 py-0.5 rounded font-mono truncate"
              style={{ background: 'rgba(0,200,255,0.1)', color: 'rgba(0,200,255,0.8)', border: '1px solid rgba(0,200,255,0.25)' }}
            >
              -ip={ip}
            </span>
          ) : (
            <span className="text-[10px] truncate" style={{ color: 'rgba(255,255,255,0.2)' }}>
              {tk('no_ip', 'no -ip')}
            </span>
          )}
          <button
            onClick={handleDetect}
            disabled={detecting}
            className="flex-shrink-0 ark-action-btn text-[10px] px-2.5 py-0.5"
          >
            {detecting ? '⏳' : `🔍 ${tk('detect', 'Detect')}`}
          </button>
        </div>
      </div>

      {/* Detection result */}
      {detectMsg && (
        <p
          className="text-[10px] leading-relaxed"
          style={{ color: detectMsg.startsWith('Error') ? 'rgba(239,68,68,0.7)' : 'rgba(74,222,128,0.8)' }}
        >
          {detectMsg.startsWith('Error') ? detectMsg : `✓ ${detectMsg}`}
        </p>
      )}

      {/* Method selector — horizontal chips */}
      <div className="flex flex-wrap gap-1.5">
        {METHOD_ORDER.map((m) => {
          const active = m === method
          return (
            <button
              key={m}
              onClick={() => updateNetwork('connection_method', m)}
              className="text-[10px] font-bold tracking-widest px-2.5 py-1 rounded transition-all"
              style={active ? {
                background: 'rgba(0,200,255,0.15)',
                color: 'rgba(0,200,255,0.95)',
                border: '1px solid rgba(0,200,255,0.5)',
              } : {
                background: 'rgba(255,255,255,0.03)',
                color: 'rgba(255,255,255,0.35)',
                border: '1px solid rgba(255,255,255,0.1)',
              }}
            >
              {METHOD_LABELS[m]}
            </button>
          )
        })}
      </div>

      {/* Active method input */}
      <div className="space-y-1">
        <input
          type="text"
          value={(net[METHOD_FIELDS[method]] as string) ?? ''}
          onChange={(e) => updateNetwork(METHOD_FIELDS[method], e.target.value)}
          placeholder={METHOD_PLACEHOLDERS[method]}
          className="w-full bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/70 placeholder-ark-cyan/25 font-mono"
        />
        <p className="text-ark-cyan/35 text-[10px] leading-relaxed">{tk(hintKey[method], '')}</p>
      </div>
    </div>
  )
}
