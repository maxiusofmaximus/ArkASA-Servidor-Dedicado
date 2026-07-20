import { useCallback, useEffect, useState } from 'react'
import { open as openExternal } from '@tauri-apps/plugin-shell'
import { invoke } from '../../services/tauri'
import { useI18n } from '../../i18n/useI18n'

interface TailscaleStatus {
  installed: boolean
  up: boolean
  ip: string | null
  hostname: string | null
  cgnat_suspect: boolean
  public_ip: string | null
  hint: string
}

interface TailscalePanelProps {
  onIpDetected?: (ip: string) => void
}

export default function TailscalePanel({ onIpDetected }: TailscalePanelProps) {
  const { tk } = useI18n()
  const [status, setStatus] = useState<TailscaleStatus | null>(null)
  const [downloadUrl, setDownloadUrl] = useState('')
  const [authKey, setAuthKey] = useState('')
  const [hostname, setHostname] = useState('')
  const [busy, setBusy] = useState(false)
  const [msg, setMsg] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    try {
      const [nextStatus, url] = await Promise.all([
        invoke<TailscaleStatus>('tailscale_status_combined'),
        invoke<string>('tailscale_download_url'),
      ])
      setStatus(nextStatus)
      setDownloadUrl(url)
      if (nextStatus.ip) onIpDetected?.(nextStatus.ip)
    } catch (error) {
      setMsg(String(error))
    }
  }, [onIpDetected])

  useEffect(() => { void refresh() }, [refresh])

  const setup = async () => {
    if (!authKey.trim()) {
      setMsg(tk('tailscale_missing_auth_key',
        'Paste an auth key — get one at https://login.tailscale.com/admin/settings/keys'))
      return
    }
    if (!hostname.trim()) {
      setMsg(tk('tailscale_missing_hostname',
        'Pick a hostname (for example arkasa-pi5) for this device.'))
      return
    }

    setBusy(true)
    setMsg(null)
    try {
      const nextStatus = await invoke<TailscaleStatus>('tailscale_setup', {
        authKey: authKey.trim(),
        hostname: hostname.trim(),
        publiclyDnsLabel: null,
      })
      setStatus(nextStatus)
      if (nextStatus.ip) onIpDetected?.(nextStatus.ip)
      setMsg(nextStatus.hint)
      setAuthKey('')
    } catch (error) {
      setMsg(String(error))
    } finally {
      setBusy(false)
      void refresh()
    }
  }

  const installTailscale = async () => {
    if (!downloadUrl) {
      await refresh()
      return
    }
    try { await openExternal(downloadUrl) } catch { /* browser opening is best effort */ }
  }

  const installed = status?.installed ?? false
  const up = status?.up ?? false
  const ip = status?.ip ?? null

  return (
    <div className="ark-panel rounded-lg p-4 space-y-3">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-ark-cyan/85 font-semibold">Tailscale network plugin</p>
          <p className="text-ark-cyan/45 text-xs leading-relaxed">
            Private 100.x.x.x connectivity for ARK servers behind CGNAT or without port forwarding.
          </p>
        </div>
        <StatusBadge connected={up} />
      </div>

      <div className="flex flex-wrap items-center gap-3 text-[10px] font-mono">
        <span className={up ? 'text-emerald-400' : 'text-amber-400'}>
          {up ? '● UP' : installed ? '○ NOT CONNECTED' : '○ NOT INSTALLED'}
        </span>
        {ip && <span className="text-ark-cyan/70">{ip}</span>}
        {status?.hostname && <span className="text-ark-cyan/45">{status.hostname}</span>}
      </div>

      {!installed && (
        <div className="rounded border border-amber-400/30 bg-amber-400/5 p-2 text-[11px] space-y-1">
          <p className="text-amber-300">{status?.hint ?? 'Tailscale status is being checked…'}</p>
          {downloadUrl && (
            <button onClick={installTailscale} className="text-ark-cyan/80 underline text-[11px] tracking-widest">
              ↓ INSTALL TAILSCALE
            </button>
          )}
        </div>
      )}

      {installed && !up && (
        <div className="space-y-2">
          <p className="text-ark-cyan/45 text-[10px]">
            Paste a one-time auth key and choose the MagicDNS hostname for this device.
          </p>
          <input
            type="password"
            value={authKey}
            onChange={(event) => setAuthKey(event.target.value)}
            placeholder="tskey-auth-…"
            className="w-full bg-black/40 border border-ark-cyan/15 rounded px-2 py-1.5 text-xs font-mono text-ark-cyan/90 placeholder:text-ark-cyan/30"
          />
          <input
            type="text"
            value={hostname}
            onChange={(event) => setHostname(event.target.value)}
            placeholder="arkasa-pi5"
            className="w-full bg-black/40 border border-ark-cyan/15 rounded px-2 py-1.5 text-xs font-mono text-ark-cyan/90 placeholder:text-ark-cyan/30"
          />
          <button
            disabled={busy}
            onClick={() => void setup()}
            className="ark-action-btn px-4 py-1.5 text-xs disabled:opacity-40"
          >
            {busy ? 'SETTING UP…' : 'SET UP TAILSCALE'}
          </button>
        </div>
      )}

      {up && ip && (
        <div className="rounded border border-emerald-400/30 bg-emerald-400/5 p-2 text-[11px] text-emerald-300 space-y-1">
          <p>● Tailscale is ready. Use this address for ARK connections:</p>
          <code className="font-mono text-emerald-200 text-sm select-all">{ip}</code>
        </div>
      )}

      <div className="flex flex-wrap items-center gap-3 pt-1">
        <button onClick={() => void refresh()} className="text-ark-cyan/60 hover:text-ark-cyan text-[10px] tracking-widest uppercase">
          ↻ {tk('btn_refresh', 'REFRESH')}
        </button>
        <button
          onClick={() => void openExternal('https://login.tailscale.com/admin/settings/keys')}
          className="text-ark-cyan/60 hover:text-ark-cyan text-[10px] tracking-widest uppercase"
        >
          → GET AUTH KEY
        </button>
      </div>
      {msg && <p className="text-ark-cyan/55 text-[10px] font-mono whitespace-pre-wrap leading-tight">{msg}</p>}
    </div>
  )
}

function StatusBadge({ connected }: { connected: boolean }) {
  return (
    <span
      className="text-[10px] uppercase tracking-widest px-2 py-0.5 rounded flex-shrink-0"
      style={{
        color: connected ? 'rgba(74,222,128,0.9)' : 'rgba(239,68,68,0.9)',
        background: connected ? 'rgba(74,222,128,0.1)' : 'rgba(239,68,68,0.1)',
        border: connected ? '1px solid rgba(74,222,128,0.4)' : '1px solid rgba(239,68,68,0.4)',
      }}
    >
      {connected ? '● connected' : '○ not connected'}
    </span>
  )
}
