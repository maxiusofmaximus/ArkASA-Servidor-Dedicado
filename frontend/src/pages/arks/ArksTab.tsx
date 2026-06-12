import React, { useMemo, useState, useEffect } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import { invoke } from '../../services/tauri'
import type { ServerConfig } from '../../types'

interface ArksTabProps {
  config: ServerConfig
}

export default function ArksTab({ config }: ArksTabProps) {
  const updateId = useConfigUpdate('identification')
  const updateNetwork = useConfigUpdate('network')

  // Ping state — persisted in localStorage so it survives tab switches and restarts
  const [pingIp, setPingIp] = useState(() => localStorage.getItem('ark-ping-ip') || '')
  // Restore running state when component remounts (tab switch) — the OS process keeps running
  const [pinging, setPinging] = useState(() => localStorage.getItem('ark-ping-active') === 'true')
  const [pingError, setPingError] = useState('')

  const settings = useMemo(
    () => [
      {
        label: 'Server Name',
        value: config.identification.session_name,
        type: 'text' as const,
        onChange: (v: string) => updateId('session_name', v),
      },
      {
        label: 'Server Password',
        value: config.identification.server_password,
        type: 'text' as const,
        onChange: (v: string) => updateId('server_password', v),
      },
      {
        label: 'Admin Password',
        value: config.identification.admin_password,
        type: 'text' as const,
        onChange: (v: string) => updateId('admin_password', v),
      },
      {
        label: 'MOTD',
        value: config.identification.server_message_of_the_day,
        type: 'text' as const,
        onChange: (v: string) => updateId('server_message_of_the_day', v),
      },
      {
        label: 'Game Port',
        value: config.network.port,
        type: 'number' as const,
        onChange: (v: number) => updateNetwork('port', v as any),
        min: 1024,
        max: 65535,
      },
      {
        label: 'Query Port',
        value: config.network.query_port,
        type: 'number' as const,
        onChange: (v: number) => updateNetwork('query_port', v as any),
        min: 1024,
        max: 65535,
      },
      {
        label: 'RCON Port',
        value: config.network.rcon_port,
        type: 'number' as const,
        onChange: (v: number) => updateNetwork('rcon_port', v as any),
        min: 1024,
        max: 65535,
      },
    ],
    [config.identification, config.network, updateId, updateNetwork]
  )

  const handleStartPing = async () => {
    const ip = pingIp.trim()
    if (!ip) { setPingError('Ingresa una IP'); return }
    setPingError('')
    try {
      await invoke('start_ping', { ip })
      setPinging(true)
      localStorage.setItem('ark-ping-ip', ip)
      localStorage.setItem('ark-ping-active', 'true')
    } catch (e) {
      setPingError(String(e))
    }
  }

  const handleStopPing = async () => {
    try {
      await invoke('stop_ping')
    } catch { /* ignore */ }
    setPinging(false)
    localStorage.removeItem('ark-ping-active')
  }

  return (
    <>
      <SettingsPanel>
        {settings.map((setting, i) => (
          <SettingRow
            key={i}
            label={setting.label}
            value={setting.value}
            type={setting.type}
            onChange={setting.onChange}
            min={setting.min}
            max={setting.max}
            testId={`arks-${setting.label.toLowerCase().replace(/ /g, '-')}`}
          />
        ))}
      </SettingsPanel>

      {/* Tailscale ping panel */}
      <div className="max-w-lg mx-auto px-8 pb-6">
        <div className="ark-panel rounded-lg p-4 space-y-3">
          <div className="flex items-center gap-2">
            <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
              Tailscale / Ping Keep-alive
            </span>
            {pinging && (
              <span className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse inline-block" />
                <span className="text-green-400/80 text-[10px] font-bold tracking-widest">ACTIVO</span>
              </span>
            )}
          </div>

          <p className="text-ark-cyan/40 text-xs leading-relaxed">
            Hace <code className="text-ark-cyan/60 font-mono">ping -t</code> continuo a la IP Tailscale de tu amigo para mantener la ruta activa. Necesario cuando el servidor y el cliente comparten la misma cuenta Tailscale.
          </p>

          <div className="flex gap-2 items-center">
            <input
              type="text"
              value={pingIp}
              onChange={e => { setPingIp(e.target.value); setPingError('') }}
              onKeyDown={e => !pinging && e.key === 'Enter' && handleStartPing()}
              placeholder="100.x.x.x  (IP Tailscale)"
              disabled={pinging}
              className="flex-1 bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/70 placeholder-ark-cyan/25 font-mono disabled:opacity-50"
            />
            {pinging ? (
              <button
                onClick={handleStopPing}
                className="ark-action-btn text-xs px-4 py-1.5 flex-shrink-0"
                style={{ color: 'rgba(239,68,68,0.80)', outlineColor: 'rgba(239,68,68,0.4)' }}
              >
                ■ DETENER
              </button>
            ) : (
              <button
                onClick={handleStartPing}
                className="ark-action-btn text-xs px-4 py-1.5 flex-shrink-0"
              >
                ▶ PING -t
              </button>
            )}
          </div>

          {pingError && (
            <p className="text-red-400/70 text-xs">{pingError}</p>
          )}
          {pinging && (
            <p className="text-green-400/55 text-xs animate-pulse">
              Pingueando <span className="font-mono">{pingIp}</span> en background... cierra con ■ DETENER o cerrando la app.
            </p>
          )}
        </div>
      </div>
    </>
  )
}
