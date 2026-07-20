import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import { useConfigStore, type ConfigStore } from '../../stores/configStore'
import { useBackupStore, type BackupStore } from '../../stores/backupStore'
import { useShallow } from 'zustand/react/shallow'
import ConnectionManager from './ConnectionManager'
import FriendContacts from './FriendContacts'
import type { ServerConfig } from '../../types'
import { useI18n } from '../../i18n/useI18n'

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

/**
 * ARK ASA does **not** expose a region flag — the server browser derives it
 * from the public IP geolocation. With LAN / playit.gg tunnels or CG-NAT,
 * the geolocation is unknown, so the field is always blank in the browser.
 *
 * We still render a non-editable placeholder row below MOTD so the user
 * understands why and so the field doesn't appear as "missing UI".
 */
const GEO_FALLBACK = '— auto —'

export default function ArksTab({ config }: ArksTabProps) {
  const updateId = useConfigUpdate('identification')
  const updateNetwork = useConfigUpdate('network')
  const setConfig = useConfigStore(useShallow((s: ConfigStore) => s.setConfig))
  const { onDemandEnabled, onDemandMaps, toggleOnDemandMap, autoShutdownMin, setAutoShutdownMin } = useBackupStore(
    useShallow((s: BackupStore) => ({
      onDemandEnabled: s.onDemandEnabled, onDemandMaps: s.onDemandMaps,
      toggleOnDemandMap: s.toggleOnDemandMap, autoShutdownMin: s.autoShutdownMin,
      setAutoShutdownMin: s.setAutoShutdownMin,
    }))
  )
  const { tk } = useI18n()


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

  const idSettings = useMemo(
    () => [
      {
        label: 'Server Name',
        value: config.identification.session_name,
        type: 'copyable' as const,
        onChange: (v: string) => updateId('session_name', v),
      },
      {
        label: 'Server Password',
        value: config.identification.server_password,
        type: 'secret' as const,
        onChange: (v: string) => updateId('server_password', v),
      },
      {
        label: 'Admin Password',
        value: config.identification.admin_password,
        type: 'secret' as const,
        onChange: (v: string) => updateId('admin_password', v),
      },
    ],
    [config.identification, updateId]
  )

  const motdSettings = useMemo(
    () => [
      {
        label: 'MOTD',
        value: config.identification.server_message_of_the_day,
        type: 'text' as const,
        onChange: (v: string) => updateId('server_message_of_the_day', v),
      },
    ],
    [config.identification, updateId]
  )

  const netSettings = useMemo(
    () => [
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
    [config.network, updateNetwork]
  )

  return (
    <>
      {/* Identification panel: rows with action column (copy/show/hide) */}
      <div className="max-w-lg mx-auto px-8 pt-6 pb-3">
        <div className="ark-panel rounded-lg overflow-hidden">
          {idSettings.map((setting, i) => (
            <SettingRow
              key={i}
              label={setting.label}
              value={setting.value}
              type={setting.type}
              onChange={setting.onChange}
              testId={`arks-${setting.label.toLowerCase().replace(/ /g, '-')}`}
            />
          ))}
        </div>
      </div>

      {/* MOTD panel */}
      <div className="max-w-lg mx-auto px-8 pb-3">
        <div className="ark-panel rounded-lg overflow-hidden">
          {motdSettings.map((setting, i) => (
            <SettingRow
              key={i}
              label={setting.label}
              value={setting.value}
              type={setting.type}
              onChange={setting.onChange}
              testId={`arks-${setting.label.toLowerCase().replace(/ /g, '-')}`}
            />
          ))}
          {/* Region — read-only informational row.
              ARK ASA does NOT expose a region command flag or INI setting;
              the server browser derives the region from the public IP's
              geolocation. With CG-NAT (LAN / playit tunnels) the region shows
              empty in the Steam/EOS server browser until the operator finds the
              server by name. This row exists to explain the gap to the user. */}
          <div
            className="flex items-center gap-3 px-4 py-2 border-t border-ark-cyan/10"
            data-testid="arks-region-info"
          >
            <div className="flex items-center gap-1.5 min-w-0 flex-1 pr-2">
              <span className="text-ark-cyan/80 text-sm tracking-wide">Region</span>
              <span
                className="flex-shrink-0 text-[9px] text-ark-cyan/30 hover:text-ark-cyan/70 transition-colors cursor-default select-none"
                style={{ lineHeight: 1 }}
                title={tk('region_info_tooltip',
                  'ARK Survival Ascended does not expose a region command flag or INI setting. The Steam / EOS server browser derives the region from the public IP geolocation.')}
              >
                ⓘ
              </span>
            </div>
            <span
              className="text-ark-cyan/45 font-mono text-sm w-32 text-right"
              style={{ filter: 'blur(1px)', userSelect: 'none' }}
              aria-hidden="true"
            >
              {GEO_FALLBACK}
            </span>
            <span className="text-ark-cyan/35 text-xs ml-3 pl-3" style={{ borderLeft: '1px solid rgba(0,212,255,0.12)' }}>
              auto
            </span>
          </div>
        </div>
      </div>

      {/* Network panel */}
      <div className="max-w-lg mx-auto px-8 pb-6">
        <div className="ark-panel rounded-lg overflow-hidden">
          {netSettings.map((setting, i) => (
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
        </div>
      </div>

      {/* Map selection */}
      <div className="max-w-lg mx-auto px-8 pb-6">
        <div className="ark-panel rounded-lg p-4 space-y-3">
          {/* Header */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
                {isCluster ? tk('cluster_maps', 'Map Cluster') : tk('server_map', 'Server Map')}
              </span>
              {isCluster && (
                <span
                  className="text-[9px] font-bold tracking-widest px-1.5 py-0.5 rounded"
                  style={{ background: 'rgba(0,200,255,0.15)', color: 'rgba(0,200,255,0.9)', border: '1px solid rgba(0,200,255,0.35)' }}
                >
                  {tk('n_instances', '{{count}} INSTANCES').replace('{{count}}', String(selectedMaps.length))}
                </span>
              )}
            </div>
            <span className="text-ark-cyan/30 text-[10px]">
              {isCluster ? tk('click_to_add_remove', 'click to add/remove') : selectedMaps[0]}
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
              {/* ── Status line: subtle hint, points operator to Options for the toggle ── */}
              <div
                className="px-2 py-1 text-[10px] flex items-center gap-2"
                style={{
                  background: (config.network?.fixed_port_assignment_per_map ?? false)
                    ? 'rgba(251,191,36,0.05)'
                    : 'rgba(74,222,128,0.05)',
                  borderBottom: '1px solid rgba(255,255,255,0.05)',
                }}
              >
                <span style={{
                  color: (config.network?.fixed_port_assignment_per_map ?? false)
                    ? 'rgba(251,191,36,0.8)'
                    : 'rgba(74,222,128,0.8)',
                }}>
                  {(config.network?.fixed_port_assignment_per_map ?? false)
                    ? `⚠️ ${tk('cluster_mode_hash', 'Hash-slot mode (FNV-1a) — ports NOT consecutive. Toggle in Options → Server Cluster.')}`
                    : `✅ ${tk('cluster_mode_consecutive', 'Consecutive ports per ARK ASA guide: Game 2× stride (7777/7779/7781), Peer = Game+1, Query +1, RCON +1')}`}
                </span>
              </div>

              {/* Header — fully fluid grid (fr units only, no fixed rem) */}
              <div
                className="grid text-[10px] font-bold tracking-widest px-2 py-1.5 gap-1"
                style={{ gridTemplateColumns: 'minmax(0,1.4fr) minmax(0,2.2rem) repeat(4, minmax(0,1fr)) minmax(0,4.5rem)', background: 'rgba(0,200,255,0.08)', color: 'rgba(0,200,255,0.55)' }}
              >
                <span>{tk('map_col', 'MAP')}</span>
                <span className="text-center">#</span>
                <span className="text-right">PEER</span>
                <span className="text-right">GAME</span>
                <span className="text-right">QUERY</span>
                <span className="text-right">RCON</span>
                <span className="text-right">{tk('mode_col', 'MODE')}</span>
              </div>

              {selectedMaps.map((mapId, i) => {
                const mapInfo   = ARK_MAPS.find((m) => m.id === mapId)
                // Mirror the Rust `ports_for_index` / `ports_for_map_id`
                // logic in JS so the operator sees the exact quartet (game,
                // peer, query, rcon) the launcher will dial. Hash must stay
                // in lockstep with port_slot_for in schema.rs.
                const fnv1aSlot = (id: string) => {
                  let h = 0x811c9dc5;
                  for (const ch of id) {
                    h ^= ch.charCodeAt(0);
                    h = Math.imul(h, 0x01000193) >>> 0;
                  }
                  return (h % 254) >>> 0;
                };
                const fixed     = config.network?.fixed_port_assignment_per_map ?? false;
                const baseGame  = config.network?.port       ?? 7777;
                const baseQuery = config.network?.query_port ?? 27015;
                const baseRcon  = config.network?.rcon_port  ?? 27020;
                const slot      = fixed ? fnv1aSlot(mapId) : i;
                const gamePort  = baseGame  + slot * 2;
                const peerPort  = gamePort + 1;
                const queryPort = baseQuery + slot;
                const rconPort  = baseRcon  + slot;
                const baseName  = config.identification?.session_name || tk('server_label', 'Server')
                const mapLabel  = mapId.replace(/_WP$/, '')
                const sessionName = i === 0 ? baseName : `${baseName} · ${mapLabel}`
                const isDormant = onDemandEnabled && onDemandMaps.includes(mapId)

                return (
                  <div key={mapId} style={{ borderTop: '1px solid rgba(255,255,255,0.05)' }}>
                    {/* Session name shown in browser */}
                    <div className="px-2 pt-1 text-[10px] truncate" style={{ color: 'rgba(255,255,255,0.35)' }}>
                      {tk('browser_name', 'Browser name')}:{' '}
                      <span className="font-mono" style={{ color: mapInfo?.color ?? '#aaa' }}>
                        {isDormant ? `${sessionName} [${tk('dormant_label', '💤 DORMANT').replace('💤 ', '')}]` : sessionName}
                      </span>
                    </div>
                    <div
                      className="grid items-center px-2 pb-1 text-[10px] sm:text-[11px] gap-1"
                      style={{ gridTemplateColumns: 'minmax(0,1.4fr) minmax(0,2.2rem) repeat(4, minmax(0,1fr)) minmax(0,4.5rem)', color: 'rgba(255,255,255,0.7)' }}
                    >
                      <span className="truncate" style={{ color: mapInfo?.color ?? '#fff' }}>{mapInfo?.name ?? mapId}</span>
                      <span className="text-center font-bold" style={{ color: mapInfo?.color ?? '#fff' }}>{i + 1}</span>
                      <span className="text-right font-mono text-ark-cyan/70" title={tk('peer_port_hint', 'ARK ASA: Steam/EOS P2P, always Game+1')}>{peerPort}</span>
                      <span className="text-right font-mono text-ark-cyan" title={tk('game_port_hint', 'Primary UDP game port — `open IP` lands here')}>{gamePort}</span>
                      <span className="text-right font-mono text-ark-cyan/70">{queryPort}</span>
                      <span className="text-right font-mono text-ark-cyan/70">{rconPort}</span>
                      {/* On-demand toggle — only shown when feature is enabled */}
                      <div className="flex justify-end">
                        {onDemandEnabled ? (
                          <button
                            onClick={() => toggleOnDemandMap(mapId)}
                            title={isDormant ? tk('dormant_mode_hint', 'Sleep mode: starts on connect') : tk('active_mode_hint', 'Always-on mode')}
                            className="flex items-center gap-1 text-[9px] font-bold tracking-wider px-1.5 py-0.5 rounded transition-all whitespace-nowrap"
                            style={isDormant ? {
                              background: 'rgba(99,102,241,0.2)',
                              color: 'rgba(165,180,252,0.9)',
                              border: '1px solid rgba(99,102,241,0.4)',
                              animation: 'pulse-dormant 3s ease-in-out infinite',
                            } : {
                              background: 'rgba(0,212,255,0.15)',
                              color: 'rgba(0,212,255,0.95)',
                              border: '1px solid rgba(0,212,255,0.4)',
                              boxShadow: '0 0 8px rgba(0,212,255,0.15)',
                            }}
                          >
                            {isDormant ? tk('dormant_label', '💤 DORMANT') : tk('active_label', '⚡ ACTIVE')}
                          </button>
                        ) : (
                          <span className="text-[9px] font-bold tracking-wider px-1.5 py-0.5 whitespace-nowrap" style={{ color: 'rgba(255,255,255,0.15)' }}>
                            {tk('active_label', '⚡ ACTIVE')}
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
                    {tk('auto_shutdown_after', '🌙 AUTO-SHUTDOWN AFTER')}
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
                    <span className="text-[10px]" style={{ color: 'rgba(165,180,252,0.5)' }}>{tk('min_empty', 'min empty (0 = never)')}</span>
                  </div>
                </div>
              )}

              <div
                className="px-3 py-1.5 text-[10px]"
                style={{ borderTop: '1px solid rgba(255,255,255,0.05)', color: 'rgba(0,200,255,0.4)' }}
              >
                {tk('cluster_travel_hint', 'Players travel between servers from obelisks / game terminals.')}
                {onDemandEnabled && onDemandMaps.some((m) => selectedMaps.includes(m)) && (
                  <span style={{ color: 'rgba(165,180,252,0.55)' }}>
                    {' '}· {tk('dormant_browser_hint', 'DORMANT maps appear in the browser but start ARK only on connect.')}
                  </span>
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Connection Manager */}
      <div className="max-w-lg mx-auto px-8 pb-3">
        <ConnectionManager config={config} />
      </div>

      {/* Friend Contacts */}
      <div className="max-w-lg mx-auto px-8 pb-6">
        <FriendContacts />
      </div>
    </>
  )
}
