import { useState, useCallback } from 'react'
import { invoke } from '../../services/tauri'
import { useConfigStore, type ConfigStore } from '../../stores/configStore'
import { useUiStore } from '../../stores/uiStore'
import { useShallow } from 'zustand/react/shallow'
import type { ServerConfig, ConnectionEntry, ConnectionType, DetectedIps } from '../../types'
import { useI18n } from '../../i18n/useI18n'

interface ConnectionManagerProps {
  config: ServerConfig
}

const CONN_TYPES: { value: ConnectionType; label: string; placeholder: string }[] = [
  { value: 'tailscale',    label: 'Tailscale',     placeholder: '100.x.x.x' },
  { value: 'public_ip',   label: 'Public IP',     placeholder: '181.237.x.x' },
  { value: 'duckdns',     label: 'DuckDNS',       placeholder: 'ark-max.duckdns.org' },
  { value: 'local_ip',    label: 'Local IP',      placeholder: '192.168.x.x' },
  { value: 'manual',      label: 'Manual',        placeholder: '...' },
  { value: 'playit_tunnel', label: 'Playit.gg',  placeholder: 'december-tribesman.gl.at.ply.gg:32181' },
  { value: 'custom',      label: 'Custom',        placeholder: '...' },
]

function primaryIp(entries: ConnectionEntry[]): string {
  const primary = entries.find((e) => e.is_primary)
  return primary?.address?.trim() ?? ''
}

export default function ConnectionManager({ config }: ConnectionManagerProps) {
  const entries = config.network.connection_entries ?? []
  const ip = primaryIp(entries)
  const { tk } = useI18n()
  const setConfig = useConfigStore(useShallow((s: ConfigStore) => s.setConfig))

  const [detecting, setDetecting] = useState(false)
  const [detectMsg, setDetectMsg] = useState<string | null>(null)
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editField, setEditField] = useState<'label' | 'address' | 'conn_type' | 'tunnel_port'>('address')
  const [editValue, setEditValue] = useState('')
  const [copiedId, setCopiedId] = useState<string | null>(null)
  const [visibleIds, setVisibleIds] = useState<Set<string>>(new Set())
  const showIpInfo = useUiStore((s) => s.ipInfoVisible)
  const toggleIpInfo = useUiStore((s) => s.toggleIpInfo)

  const toggleEntryVisible = (id: string) => {
    setVisibleIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const updateEntries = useCallback(
    (next: ConnectionEntry[]) => {
      if (!config) return
      setConfig({ ...config, network: { ...config.network, connection_entries: next } })
    },
    [config, setConfig]
  )

  const addEntry = () => {
    const id = crypto.randomUUID()
    const entry: ConnectionEntry = {
      id,
      conn_type: 'manual',
      label: tk('conn_new_entry', 'New Connection'),
      address: '',
      is_primary: entries.length === 0,
      tunnel_port: null,
    }
    updateEntries([...entries, entry])
    setEditingId(id)
    setEditField('address')
    setEditValue('')
  }

  const removeEntry = (id: string) => {
    const next = entries.filter((e) => e.id !== id)
    if (next.length > 0 && !next.some((e) => e.is_primary)) {
      next[0] = { ...next[0], is_primary: true }
    }
    updateEntries(next)
  }

  const setPrimary = (id: string) => {
    updateEntries(entries.map((e) => ({ ...e, is_primary: e.id === id })))
  }

  const updateEntry = (id: string, patch: Partial<ConnectionEntry>) => {
    updateEntries(entries.map((e) => (e.id === id ? { ...e, ...patch } : e)))
  }

  const startEdit = (id: string, field: 'label' | 'address' | 'conn_type' | 'tunnel_port', current: string) => {
    setEditingId(id)
    setEditField(field)
    setEditValue(current)
  }

  const commitEdit = () => {
    if (!editingId) return
    if (editField === 'tunnel_port') {
      updateEntry(editingId, { tunnel_port: editValue ? parseInt(editValue, 10) || null : null })
    } else {
      updateEntry(editingId, { [editField]: editValue.trim() })
    }
    setEditingId(null)
  }

  const handleCopyIp = async (id: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopiedId(id)
      setTimeout(() => setCopiedId(null), 1500)
    } catch { /* ignore */ }
  }

  const handleDetect = async () => {
    setDetecting(true)
    setDetectMsg(null)
    try {
      const ips = await invoke<DetectedIps>('detect_ips')
      const found: string[] = []
      const next = [...entries]
      if (ips.public_ip) {
        const idx = next.findIndex((e) => e.conn_type === 'public_ip')
        if (idx >= 0) { next[idx] = { ...next[idx], address: ips.public_ip } }
        else { next.push({ id: crypto.randomUUID(), conn_type: 'public_ip', label: 'Public IP', address: ips.public_ip, is_primary: !next.some((e) => e.is_primary), tunnel_port: null }) }
        found.push(`${tk('public_ip_label', 'Public')}: ${ips.public_ip}`)
      }
      if (ips.tailscale_ip) {
        const idx = next.findIndex((e) => e.conn_type === 'tailscale')
        if (idx >= 0) { next[idx] = { ...next[idx], address: ips.tailscale_ip } }
        else { next.push({ id: crypto.randomUUID(), conn_type: 'tailscale', label: 'Tailscale', address: ips.tailscale_ip, is_primary: !next.some((e) => e.is_primary), tunnel_port: null }) }
        found.push(`Tailscale: ${ips.tailscale_ip}`)
      }
      if (ips.local_ip) {
        const idx = next.findIndex((e) => e.conn_type === 'local_ip')
        if (idx >= 0) { next[idx] = { ...next[idx], address: ips.local_ip } }
        else { next.push({ id: crypto.randomUUID(), conn_type: 'local_ip', label: 'Local IP', address: ips.local_ip, is_primary: !next.some((e) => e.is_primary), tunnel_port: null }) }
        found.push(`Local: ${ips.local_ip}`)
      }
      updateEntries(next)
      setDetectMsg(found.length > 0 ? found.join(' · ') : tk('no_ip_detected', 'No IPs detected automatically'))
    } catch (e) {
      setDetectMsg(`Error: ${String(e)}`)
    } finally {
      setDetecting(false)
    }
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
          <button
            onClick={addEntry}
            className="flex-shrink-0 ark-action-btn text-[10px] px-2.5 py-0.5"
          >
            {tk('add_connection', '+ Add')}
          </button>
          <button
            onClick={toggleIpInfo}
            className="flex-shrink-0 text-[10px] w-5 text-center py-0.5 rounded transition-colors"
            style={{ color: showIpInfo ? 'rgba(0,212,255,0.9)' : 'rgba(255,255,255,0.25)' }}
            title={showIpInfo ? tk('hide_ip_info', '◂ IP Info') : tk('show_ip_info', 'IP Info ▸')}
          >
            ⓘ
          </button>
        </div>
      </div>

      {/* IP info panel */}
      {showIpInfo && (
        <div
          className="rounded-md p-3 space-y-1.5 text-xs"
          style={{ background: 'rgba(0,200,255,0.04)', border: '1px solid rgba(0,200,255,0.15)' }}
        >
          {entries.length === 0 ? (
            <p style={{ color: 'rgba(255,255,255,0.3)' }}>{tk('no_ip_info', 'No connections configured. Use Detect or + Add.')}</p>
          ) : (
            entries.map((entry) => {
              const typeInfo = CONN_TYPES.find((t) => t.value === entry.conn_type)
              return (
                <div key={entry.id} className="flex items-center gap-2">
                  <span
                    className="text-[10px] font-bold tracking-widest px-1.5 py-0.5 rounded"
                    style={{ background: 'rgba(0,200,255,0.08)', color: entry.is_primary ? 'rgba(0,212,255,0.9)' : 'rgba(0,200,255,0.5)', border: '1px solid rgba(0,200,255,0.2)' }}
                  >
                    {typeInfo?.label ?? entry.conn_type}
                    {entry.is_primary ? ' ★' : ''}
                  </span>
                  <span className="font-mono" style={{ color: entry.address ? 'rgba(0,212,255,0.8)' : 'rgba(255,255,255,0.25)' }}>
                    {entry.address || tk('no_ip_short', 'no IP')}
                  </span>
                  {entry.conn_type === 'playit_tunnel' && entry.tunnel_port && (
                    <span className="font-mono text-[10px]" style={{ color: 'rgba(0,200,255,0.5)' }}>:{entry.tunnel_port}</span>
                  )}
                </div>
              )
            })
          )}
        </div>
      )}

      {/* Detection result */}
      {detectMsg && (
        <p
          className="text-[10px] leading-relaxed"
          style={{ color: detectMsg.startsWith('Error') ? 'rgba(239,68,68,0.7)' : 'rgba(74,222,128,0.8)' }}
        >
          {detectMsg.startsWith('Error') ? detectMsg : `✓ ${detectMsg}`}
        </p>
      )}

      {/* Connection entries list */}
      {entries.length === 0 ? (
        <p className="text-ark-cyan/25 text-xs py-2 text-center">
          {tk('no_connections', 'No connections — add one with the + button or detect IPs')}
        </p>
      ) : (
        <div className="space-y-1.5">
          {entries.map((entry) => {
            const typeInfo = CONN_TYPES.find((t) => t.value === entry.conn_type)
            const isEditing = editingId === entry.id

            return (
              <div
                key={entry.id}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded"
                style={{
                  background: entry.is_primary ? 'rgba(0,212,255,0.06)' : 'rgba(255,255,255,0.03)',
                  border: `1px solid ${entry.is_primary ? 'rgba(0,212,255,0.3)' : 'rgba(255,255,255,0.07)'}`,
                }}
              >
                {/* Primary star */}
                <button
                  onClick={() => setPrimary(entry.id)}
                  className="flex-shrink-0 text-sm leading-none transition-colors"
                  style={{ color: entry.is_primary ? 'rgba(0,212,255,0.9)' : 'rgba(255,255,255,0.15)' }}
                  title={entry.is_primary ? tk('primary_active', 'Primary (used for -ip= flag)') : tk('set_primary', 'Set as primary')}
                >
                  ★
                </button>

                {/* Type chip */}
                {isEditing && editField === 'conn_type' ? (
                  <select
                    autoFocus
                    value={editValue}
                    onChange={(e) => { setEditValue(e.target.value) }}
                    onBlur={commitEdit}
                    className="flex-shrink-0 text-[10px] font-bold tracking-widest px-1.5 py-0.5 rounded bg-transparent border border-ark-cyan/40 text-ark-cyan/90 focus:outline-none"
                    style={{ background: 'rgba(0,200,255,0.08)' }}
                  >
                    {CONN_TYPES.map((t) => (
                      <option key={t.value} value={t.value} style={{ background: '#1a1a2e' }}>
                        {t.label}
                      </option>
                    ))}
                  </select>
                ) : (
                  <button
                    onClick={() => startEdit(entry.id, 'conn_type', entry.conn_type)}
                    className="flex-shrink-0 text-[10px] font-bold tracking-widest px-2 py-0.5 rounded cursor-pointer"
                    style={{
                      background: 'rgba(0,200,255,0.08)',
                      color: entry.is_primary ? 'rgba(0,212,255,0.9)' : 'rgba(0,200,255,0.5)',
                      border: '1px solid rgba(0,200,255,0.2)',
                    }}
                  >
                    {typeInfo?.label ?? entry.conn_type}
                  </button>
                )}

                {/* Label */}
                {isEditing && editField === 'label' ? (
                  <input
                    autoFocus
                    className="flex-shrink-0 w-20 bg-transparent border border-ark-cyan/40 text-ark-cyan/90 text-xs px-1.5 py-0.5 rounded focus:outline-none font-mono"
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    onBlur={commitEdit}
                    onKeyDown={(e) => { if (e.key === 'Enter') commitEdit() }}
                  />
                ) : (
                  <span
                    className="flex-shrink-0 w-20 text-xs font-semibold cursor-pointer truncate"
                    style={{ color: entry.is_primary ? 'rgba(0,212,255,0.9)' : 'rgba(255,255,255,0.7)' }}
                    onClick={() => startEdit(entry.id, 'label', entry.label)}
                  >
                    {entry.label || tk('no_label', 'no label')}
                  </span>
                )}

                {/* Address */}
                {isEditing && editField === 'address' ? (
                  <input
                    autoFocus
                    className="flex-1 bg-transparent border border-ark-cyan/40 text-ark-cyan/90 text-xs px-1.5 py-0.5 rounded focus:outline-none font-mono"
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    onBlur={commitEdit}
                    onKeyDown={(e) => { if (e.key === 'Enter') commitEdit() }}
                    placeholder={typeInfo?.placeholder}
                  />
                ) : (
                  <span
                    className="flex-1 text-xs font-mono cursor-pointer truncate select-none"
                    style={{
                      color: entry.is_primary ? 'rgba(0,212,255,0.7)' : 'rgba(0,200,255,0.5)',
                      filter: entry.address && !visibleIds.has(entry.id) ? 'blur(5px)' : 'none',
                    }}
                    onClick={() => startEdit(entry.id, 'address', entry.address)}
                  >
                    {entry.address || <span style={{ color: 'rgba(255,255,255,0.2)', filter: 'none' }}>{tk('no_ip_short', 'no IP')}</span>}
                  </span>
                )}

                {/* Show/hide toggle */}
                {entry.address && !isEditing && (
                  <button
                    onClick={() => toggleEntryVisible(entry.id)}
                    className="flex-shrink-0 text-[10px] w-5 text-center py-0.5 rounded transition-colors"
                    style={{ color: visibleIds.has(entry.id) ? 'rgba(0,212,255,0.9)' : 'rgba(255,255,255,0.15)' }}
                    title={visibleIds.has(entry.id) ? tk('hide', 'Hide') : tk('show', 'Show')}
                  >
                    {visibleIds.has(entry.id) ? '◉' : '○'}
                  </button>
                )}

                {/* Copy button */}
                {entry.address && !isEditing && (
                  <button
                    onClick={() => handleCopyIp(entry.id, entry.address)}
                    className="flex-shrink-0 text-[10px] px-1 py-0.5 rounded transition-colors"
                    style={{ color: copiedId === entry.id ? 'rgba(74,222,128,0.8)' : 'rgba(255,255,255,0.25)' }}
                    title={tk('copy', 'Copy')}
                  >
                    {copiedId === entry.id ? '✓' : '⧉'}
                  </button>
                )}

                {/* Tunnel port (only for playit_tunnel) */}
                {entry.conn_type === 'playit_tunnel' && (
                  isEditing && editField === 'tunnel_port' ? (
                    <input
                      autoFocus
                      type="number"
                      className="w-14 bg-transparent border border-ark-cyan/40 text-ark-cyan/90 text-[10px] px-1 py-0.5 rounded focus:outline-none font-mono"
                      value={editValue}
                      onChange={(e) => setEditValue(e.target.value)}
                      onBlur={commitEdit}
                      onKeyDown={(e) => { if (e.key === 'Enter') commitEdit() }}
                      placeholder="32181"
                    />
                  ) : (
                    <span
                      className="text-[10px] font-mono cursor-pointer"
                      style={{ color: entry.tunnel_port ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.15)' }}
                      onClick={() => startEdit(entry.id, 'tunnel_port', String(entry.tunnel_port ?? ''))}
                    >
                      :{entry.tunnel_port ?? 'port'}
                    </span>
                  )
                )}

                {/* Delete */}
                <button
                  onClick={() => removeEntry(entry.id)}
                  className="flex-shrink-0 text-[10px] px-1.5 py-0.5 rounded transition-colors"
                  style={{ color: 'rgba(239,68,68,0.4)' }}
                >
                  ×
                </button>
              </div>
            )
          })}
        </div>
      )}

      <p className="text-ark-cyan/25 text-[10px]">
        {tk('footer_connections_hint', '★ = primary (used for -ip= flag) · Click type/label/address to edit')}
      </p>
    </div>
  )
}
