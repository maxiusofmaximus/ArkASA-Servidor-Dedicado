import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import { useConfigStore } from '../../stores/configStore'
import { useBackupStore } from '../../stores/backupStore'
import ConnectionManager from './ConnectionManager'
import FriendContacts from './FriendContacts'
import type { ServerConfig } from '../../types'

interface ArksTabProps {
  config: ServerConfig
}

interface ArkMap {
  id: string
  name: string
  dlc: 'free' | 'paid' | 'base'
  color: string
}

const ARK_MAPS: ArkMap[] = [
  { id: 'TheIsland_WP',     name: 'The Island',      dlc: 'base', color: '#22c55e' },
  { id: 'TheCenter_WP',     name: 'The Center',      dlc: 'free', color: '#3b82f6' },
  { id: 'ScorchedEarth_WP', name: 'Scorched Earth',  dlc: 'paid', color: '#f97316' },
  { id: 'Aberration_WP',    name: 'Aberration',      dlc: 'paid', color: '#a855f7' },
  { id: 'Extinction_WP',    name: 'Extinction',      dlc: 'paid', color: '#ef4444' },
  { id: 'Ragnarok_WP',      name: 'Ragnarok',        dlc: 'free', color: '#0ea5e9' },
  { id: 'CrystalIsles_WP',  name: 'Crystal Isles',  dlc: 'free', color: '#06b6d4' },
  { id: 'Gen1_WP',          name: 'Genesis Part 1',  dlc: 'paid', color: '#8b5cf6' },
  { id: 'Gen2_WP',          name: 'Genesis Part 2',  dlc: 'paid', color: '#ec4899' },
  { id: 'LostIsland_WP',    name: 'Lost Island',     dlc: 'free', color: '#14b8a6' },
  { id: 'Fjordur_WP',       name: 'Fjordur',         dlc: 'free', color: '#6366f1' },
]

const DLC_LABEL: Record<ArkMap['dlc'], string> = {
  base: 'BASE',
  free: 'FREE',
  paid: 'DLC',
}

export default function ArksTab({ config }: ArksTabProps) {
  const updateId = useConfigUpdate('identification')
  const updateNetwork = useConfigUpdate('network')
  const { setConfig } = useConfigStore()
  const { onDemandEnabled, onDemandMaps, toggleOnDemandMap, autoShutdownMin, setAutoShutdownMin } = useBackupStore()


  const selectedMaps: string[] = config.cluster_maps?.length
    ? config.cluster_maps
    : ['TheIsland_WP']

  const handleToggleMap = (mapId: string) => {
    const isSelected = selectedMaps.includes(mapId)
    if (isSelected) {
      // Cannot deselect the last map
      if (selectedMaps.length === 1) return
      setConfig({ ...config, cluster_maps: selectedMaps.filter((m) => m !== mapId) })
    } else {
      setConfig({ ...config, cluster_maps: [...selectedMaps, mapId] })
    }
  }

  const isCluster = selectedMaps.length > 1

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

      {/* Map selection */}
      <div className="max-w-lg mx-auto px-8 pb-6">
        <div className="ark-panel rounded-lg p-4 space-y-3">
          {/* Header */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
                {isCluster ? 'Cluster de Mapas' : 'Mapa del Servidor'}
              </span>
              {isCluster && (
                <span
                  className="text-[9px] font-bold tracking-widest px-1.5 py-0.5 rounded"
                  style={{ background: 'rgba(0,200,255,0.15)', color: 'rgba(0,200,255,0.9)', border: '1px solid rgba(0,200,255,0.35)' }}
                >
                  {selectedMaps.length} INSTANCIAS
                </span>
              )}
            </div>
            <span className="text-ark-cyan/30 text-[10px]">
              {isCluster ? 'clic para añadir/quitar' : selectedMaps[0]}
            </span>
          </div>

          {/* Map grid */}
          <div className="grid grid-cols-3 gap-2">
            {ARK_MAPS.map((m) => {
              const idx = selectedMaps.indexOf(m.id)
              const isSelected = idx !== -1
              return (
                <button
                  key={m.id}
                  onClick={() => handleToggleMap(m.id)}
                  className="relative rounded-md p-2.5 text-left transition-all duration-150 focus:outline-none"
                  style={{
                    background: isSelected ? `${m.color}18` : 'rgba(255,255,255,0.03)',
                    border: isSelected ? `1px solid ${m.color}80` : '1px solid rgba(255,255,255,0.08)',
                    boxShadow: isSelected ? `0 0 10px ${m.color}30` : 'none',
                  }}
                >
                  {/* Top accent bar */}
                  <div
                    className="absolute top-0 left-0 right-0 h-0.5 rounded-t-md"
                    style={{ background: isSelected ? m.color : 'transparent' }}
                  />

                  <div className="flex flex-col gap-1">
                    <span
                      className="text-[10px] font-bold tracking-widest"
                      style={{ color: isSelected ? m.color : 'rgba(255,255,255,0.25)' }}
                    >
                      {DLC_LABEL[m.dlc]}
                    </span>
                    <span
                      className="text-xs font-semibold leading-tight"
                      style={{ color: isSelected ? 'rgba(255,255,255,0.95)' : 'rgba(255,255,255,0.55)' }}
                    >
                      {m.name}
                    </span>
                  </div>

                  {/* Badge: dot for single, number for cluster */}
                  {isSelected && (
                    <div
                      className="absolute top-1.5 right-1.5 flex items-center justify-center rounded-full text-[9px] font-bold"
                      style={{
                        background: m.color,
                        color: '#000',
                        width: isCluster ? '1.1rem' : '0.45rem',
                        height: isCluster ? '1.1rem' : '0.45rem',
                      }}
                    >
                      {isCluster ? idx + 1 : ''}
                    </div>
                  )}
                </button>
              )
            })}
          </div>

          {/* Cluster port table */}
          {isCluster && (
            <div className="mt-1 rounded-md overflow-hidden" style={{ border: '1px solid rgba(0,200,255,0.15)' }}>
              {/* Header */}
              <div
                className="grid text-[10px] font-bold tracking-widest px-3 py-1.5"
                style={{ gridTemplateColumns: '1fr 3rem 4rem 4rem 4rem 5.5rem', background: 'rgba(0,200,255,0.08)', color: 'rgba(0,200,255,0.55)' }}
              >
                <span>MAPA</span>
                <span className="text-center">#</span>
                <span className="text-right">GAME</span>
                <span className="text-right">QUERY</span>
                <span className="text-right">RCON</span>
                <span className="text-right">MODO</span>
              </div>

              {selectedMaps.map((mapId, i) => {
                const mapInfo   = ARK_MAPS.find((m) => m.id === mapId)
                const gamePort  = (config.network?.port       ?? 7777)  + i * 2
                const queryPort = (config.network?.query_port ?? 27015) + i
                const rconPort  = (config.network?.rcon_port  ?? 27020) + i
                const baseName  = config.identification?.session_name || 'Servidor'
                const mapLabel  = mapId.replace(/_WP$/, '')
                const sessionName = i === 0 ? baseName : `${baseName} · ${mapLabel}`
                const isDormant = onDemandEnabled && onDemandMaps.includes(mapId)

                return (
                  <div key={mapId} style={{ borderTop: '1px solid rgba(255,255,255,0.05)' }}>
                    {/* Session name shown in browser */}
                    <div className="px-3 pt-1.5 text-[10px]" style={{ color: 'rgba(255,255,255,0.35)' }}>
                      Nombre en browser:{' '}
                      <span className="font-mono" style={{ color: mapInfo?.color ?? '#aaa' }}>
                        {isDormant ? `${sessionName} [DORMIDO]` : sessionName}
                      </span>
                    </div>
                    <div
                      className="grid items-center px-3 pb-1.5 text-[11px]"
                      style={{ gridTemplateColumns: '1fr 3rem 4rem 4rem 4rem 5.5rem', color: 'rgba(255,255,255,0.7)' }}
                    >
                      <span style={{ color: mapInfo?.color ?? '#fff' }}>{mapInfo?.name ?? mapId}</span>
                      <span className="text-center font-bold" style={{ color: mapInfo?.color ?? '#fff' }}>{i + 1}</span>
                      <span className="text-right font-mono text-ark-cyan/70">{gamePort}</span>
                      <span className="text-right font-mono text-ark-cyan/70">{queryPort}</span>
                      <span className="text-right font-mono text-ark-cyan/70">{rconPort}</span>
                      {/* On-demand toggle — only shown when feature is enabled */}
                      <div className="flex justify-end">
                        {onDemandEnabled ? (
                          <button
                            onClick={() => toggleOnDemandMap(mapId)}
                            title={isDormant ? 'Modo dormido: inicia al conectar' : 'Modo siempre activo'}
                            className="flex items-center gap-1 text-[9px] font-bold tracking-wider px-2 py-0.5 rounded transition-all"
                            style={isDormant ? {
                              background: 'rgba(99,102,241,0.2)',
                              color: 'rgba(165,180,252,0.9)',
                              border: '1px solid rgba(99,102,241,0.4)',
                            } : {
                              background: 'rgba(255,255,255,0.05)',
                              color: 'rgba(255,255,255,0.3)',
                              border: '1px solid rgba(255,255,255,0.1)',
                            }}
                          >
                            {isDormant ? '🌙 DORMIDO' : '▶ ACTIVO'}
                          </button>
                        ) : (
                          <span className="text-[9px] font-bold tracking-wider px-2 py-0.5" style={{ color: 'rgba(255,255,255,0.15)' }}>
                            ▶ ACTIVO
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                )
              })}

              {/* Auto-shutdown setting (only shown when feature is enabled and any map is dormant) */}
              {onDemandEnabled && onDemandMaps.some((m) => selectedMaps.includes(m)) && (
                <div
                  className="px-3 py-2 flex items-center gap-3"
                  style={{ borderTop: '1px solid rgba(99,102,241,0.2)', background: 'rgba(99,102,241,0.05)' }}
                >
                  <span className="text-[10px] font-bold tracking-widest" style={{ color: 'rgba(165,180,252,0.7)' }}>
                    🌙 AUTO-APAGAR TRAS
                  </span>
                  <div className="flex items-center gap-1.5">
                    <input
                      type="number"
                      min={0}
                      max={1440}
                      value={autoShutdownMin}
                      onChange={(e) => setAutoShutdownMin(Math.max(0, parseInt(e.target.value) || 0))}
                      className="w-16 bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-xs px-2 py-0.5 rounded focus:outline-none focus:border-ark-cyan/70 text-center font-mono"
                    />
                    <span className="text-[10px]" style={{ color: 'rgba(165,180,252,0.5)' }}>min vacío (0 = nunca)</span>
                  </div>
                </div>
              )}

              <div
                className="px-3 py-1.5 text-[10px]"
                style={{ borderTop: '1px solid rgba(255,255,255,0.05)', color: 'rgba(0,200,255,0.4)' }}
              >
                Los jugadores viajan entre servidores desde los obeliscos / terminales del juego.
                {onDemandEnabled && onDemandMaps.some((m) => selectedMaps.includes(m)) && (
                  <span style={{ color: 'rgba(165,180,252,0.55)' }}>
                    {' '}· Mapas <span style={{ color: 'rgba(165,180,252,0.8)' }}>DORMIDOS</span> aparecen en el browser pero inician ARK solo al conectar.
                  </span>
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Connection Manager */}
      <div className="max-w-lg mx-auto px-8 pb-3">
        <ConnectionManager config={config} updateNetwork={updateNetwork as any} />
      </div>

      {/* Friend Contacts */}
      <div className="max-w-lg mx-auto px-8 pb-6">
        <FriendContacts />
      </div>
    </>
  )
}
