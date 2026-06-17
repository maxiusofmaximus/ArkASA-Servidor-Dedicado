import type { ServerConfig, ConnectionMethod, NetworkConfig } from '../../types'

interface ConnectionManagerProps {
  config: ServerConfig
  updateNetwork: (field: keyof NetworkConfig, value: string) => void
}

type MethodMeta = {
  label: string
  field: keyof Pick<NetworkConfig, 'tailscale_ip' | 'public_ip' | 'duckdns_host' | 'local_ip' | 'manual_ip'>
  placeholder: string
  hint: string
}

const METHODS: Record<ConnectionMethod, MethodMeta> = {
  tailscale: {
    label: 'Tailscale',
    field: 'tailscale_ip',
    placeholder: '100.x.x.x',
    hint: 'VPN — amigos deben tener Tailscale instalado y estar en tu red',
  },
  public: {
    label: 'IP Pública',
    field: 'public_ip',
    placeholder: '181.237.x.x',
    hint: 'IP pública directa — requiere port forwarding en el router',
  },
  duckdns: {
    label: 'DuckDNS',
    field: 'duckdns_host',
    placeholder: 'ark-max.duckdns.org',
    hint: 'Hostname DNS fijo — actualizado automáticamente aunque cambie la IP pública',
  },
  local: {
    label: 'IP Local',
    field: 'local_ip',
    placeholder: '192.168.x.x',
    hint: 'Solo LAN — amigos deben estar en la misma red Wi-Fi/ethernet',
  },
  manual: {
    label: 'Manual',
    field: 'manual_ip',
    placeholder: 'cualquier IP o hostname',
    hint: 'Cualquier valor personalizado — también usado si solo tienes el campo antiguo',
  },
}

const METHOD_ORDER: ConnectionMethod[] = ['tailscale', 'public', 'duckdns', 'local', 'manual']

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
  const meta = METHODS[method]
  const ip = effectiveIp(net)

  return (
    <div className="ark-panel rounded-lg p-4 space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
          Conexión del Servidor
        </span>
        {ip ? (
          <span
            className="text-[10px] font-bold tracking-widest px-2 py-0.5 rounded font-mono"
            style={{ background: 'rgba(0,200,255,0.1)', color: 'rgba(0,200,255,0.8)', border: '1px solid rgba(0,200,255,0.25)' }}
          >
            -ip={ip}
          </span>
        ) : (
          <span className="text-[10px]" style={{ color: 'rgba(255,255,255,0.2)' }}>
            sin -ip — ARK escucha en todas las interfaces
          </span>
        )}
      </div>

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
              {METHODS[m].label}
            </button>
          )
        })}
      </div>

      {/* Active method input */}
      <div className="space-y-1">
        <input
          type="text"
          value={(net[meta.field] as string) ?? ''}
          onChange={(e) => updateNetwork(meta.field, e.target.value)}
          placeholder={meta.placeholder}
          className="w-full bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/70 placeholder-ark-cyan/25 font-mono"
        />
        <p className="text-ark-cyan/35 text-[10px] leading-relaxed">{meta.hint}</p>
      </div>
    </div>
  )
}
