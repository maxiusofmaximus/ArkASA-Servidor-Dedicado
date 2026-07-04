import { useEffect, useState, useCallback } from 'react'
import { useBackupStore, type BackupScope } from '../../stores/backupStore'
import { useConfigStore, type ConfigStore } from '../../stores/configStore'
import { useShallow } from 'zustand/react/shallow'
import { useI18n } from '../../i18n/useI18n'
import { Section, Toggle } from '../ui/OptionsUI'
import { invoke } from '@tauri-apps/api/core'
import { open as openExternal } from '@tauri-apps/plugin-shell'

async function pushConvex(): Promise<string> {
  return await invoke<string>('convex_push_schema')
}

async function deployVercel(): Promise<{ url: string; status: string }> {
  const r = await invoke<{
    connected: boolean
    last_deploy_url: string | null
    last_deploy_status: string | null
    last_deploy_at_unix: number | null
    project_name: string | null
  }>('vercel_deploy_web')
  return { url: r.last_deploy_url ?? '', status: r.last_deploy_status ?? '' }
}

async function getConvexStatus() {
  return await invoke<{
    connected: boolean
    deployment_url: string | null
    schema_pushed_at_unix: number | null
  }>('convex_status')
}

async function getVercelStatus() {
  return await invoke<{
    connected: boolean
    last_deploy_url: string | null
    last_deploy_status: string | null
  }>('vercel_status')
}

const SCOPE_VALUES: BackupScope[] = ['map', 'map_players_tribes', 'full']

const LANGUAGES: { code: string; label: string; native: string }[] = [
  { code: 'es', label: 'Español',            native: 'Español' },
  { code: 'en', label: 'English',            native: 'English' },
  { code: 'fr', label: 'Français',           native: 'Français' },
  { code: 'zh', label: 'Simplified Chinese', native: '简体中文' },
  { code: 'ja', label: 'Japanese',           native: '日本語' },
  { code: 'ko', label: 'Korean',             native: '한국어' },
  { code: 'pt', label: 'Português',          native: 'Português' },
  { code: 'de', label: 'Deutsch',            native: 'Deutsch' },
  { code: 'it', label: 'Italiano',           native: 'Italiano' },
  { code: 'ru', label: 'Russian',           native: 'Русский' },
]

export default function GeneralTab() {
  const store = useBackupStore()
  const cfg = useConfigStore(
    useShallow((s: ConfigStore) => ({
      network: s.config?.network,
      updateNetwork: (patch: Partial<NonNullable<typeof s.config>['network']>) => {
        if (!s.config) return
        s.setConfig({ ...s.config, network: { ...s.config.network, ...patch } })
      },
    })),
  )
  const { tk } = useI18n()

  return (
    <>
      <Section title={tk('section_language', 'Language')}>
        <div className="grid grid-cols-2 gap-2">
          {LANGUAGES.map((lang) => (
            <button
              key={lang.code}
              onClick={() => store.setLanguage(lang.code)}
              className="rounded-md p-2.5 text-left transition-all flex items-center gap-2"
              style={{
                background: store.language === lang.code ? 'rgba(0,200,255,0.1)' : 'rgba(255,255,255,0.03)',
                border: `1px solid ${store.language === lang.code ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.07)'}`,
              }}
            >
              <span className="text-xs font-semibold" style={{ color: store.language === lang.code ? 'rgba(0,200,255,0.9)' : 'rgba(255,255,255,0.45)' }}>
                {lang.native}
              </span>
              {lang.native !== lang.label && (
                <span className="text-[10px]" style={{ color: store.language === lang.code ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.2)' }}>
                  {lang.label}
                </span>
              )}
            </button>
          ))}
        </div>
        <p className="text-ark-cyan/30 text-[10px] mt-1">
          {tk('lang_pref_note', 'Language preference is saved. Translations are applied progressively.')}
        </p>
      </Section>

      <Section title={tk('section_on_demand', 'On-Demand Server')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('on_demand_title', 'Enable sleep mode')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('on_demand_desc', 'Lets individual maps appear in the ARK browser without the server running.')}</p>
          </div>
          <Toggle value={store.onDemandEnabled} onChange={store.setOnDemandEnabled} />
        </div>
      </Section>

      {!store.onDemandEnabled && (
        <Section title={tk('section_cluster', 'Server Cluster')}>
          <div className="space-y-4">
            <div>
              <p className="text-ark-cyan/80 text-sm">{tk('cluster_delay_title', 'Delay between cluster instances')}</p>
              <p className="text-ark-cyan/40 text-xs mt-0.5 mb-3">{tk('cluster_delay_desc', 'Wait time between each map startup in a cluster.')}</p>
              <div className="flex items-center gap-4">
                <input
                  type="range"
                  min={0}
                  max={180}
                  step={5}
                  value={store.clusterStartDelaySec}
                  onChange={(e) => store.setClusterStartDelaySec(Number(e.target.value))}
                  className="flex-1 accent-ark-cyan"
                />
                <span className="text-ark-cyan/80 font-mono text-sm w-20 text-right">
                  {store.clusterStartDelaySec === 0 ? tk('no_delay', 'No delay') : `${store.clusterStartDelaySec} s`}
                </span>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-ark-cyan/80 text-sm">{tk('fixed_ports_title', 'Fixed port assignment per map (Recommended)')}</p>
                <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('fixed_ports_desc', 'Each cluster map always lands on the same ports.')}</p>
              </div>
              <Toggle
                value={cfg.network?.fixed_port_assignment_per_map ?? true}
                onChange={(v) => cfg.updateNetwork({ fixed_port_assignment_per_map: v })}
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-ark-cyan/80 text-sm">{tk('no_battleye_title', 'Disable BattleEye anti-cheat')}</p>
                <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('no_battleye_desc', 'Adds -NoBattlEye to launch args.')}</p>
              </div>
              <Toggle
                value={cfg.network?.no_battleye ?? false}
                onChange={(v) => cfg.updateNetwork({ no_battleye: v })}
              />
            </div>
          </div>
        </Section>
      )}

      {/* ── Internet gate (v2.1) ────────────────────────────────────────── */}
      <Section title={tk('section_internet', 'Internet')}>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-ark-cyan/80 text-sm">{tk('allow_start_offline_title', 'Allow start without internet')}</p>
              <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('allow_start_offline_desc', 'Skip the offline check and let the server launch anyway.')}</p>
            </div>
            <Toggle
              value={cfg.network?.allow_start_without_internet ?? false}
              onChange={(v) => cfg.updateNetwork({ allow_start_without_internet: v })}
            />
          </div>
        </div>
      </Section>

      {/* ── Public network & Tailscale (v2.1) ──────────────────────────── */}
      <Section title={tk('section_tailscale', 'Public network & Tailscale')}>
        <TailscaleWizard />
      </Section>

      <Section title={tk('section_close_behavior', 'Close Behavior')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('minimize_tray_title', 'Minimize to system tray')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('minimize_tray_desc', 'When closing the window, the app minimizes to the system tray.')}</p>
          </div>
          <Toggle value={store.minimizeToTray} onChange={store.setMinimizeToTray} />
        </div>
      </Section>

      <Section title={tk('section_save', 'Save Settings')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('manual_save_title', 'Manual save')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('manual_save_desc', 'By default changes are saved automatically. Enable this to save only when you press SAVE SETTINGS.')}</p>
          </div>
          <Toggle value={store.manualSave} onChange={store.setManualSave} />
        </div>
      </Section>

      <Section title={tk('section_logs', 'Log Viewer')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('logs_btn_title', 'Server logs button')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('logs_btn_desc', 'Shows the LOGS button in the bottom bar to view ShooterGame.log in real time')}</p>
          </div>
          <Toggle value={store.logsEnabled} onChange={store.setLogsEnabled} />
        </div>
      </Section>

      {/* ── Cloud Services (v2.1) ──────────────────────────────────────── */}
      <Section title={tk('section_cloud_services', 'Cloud Services')}>
        <div className="space-y-4 text-sm">
          <ConvexCard />
          <VercelCard />
        </div>
      </Section>

      <Section title={tk('section_backup_scope', 'Backup Scope')}>
        <div className="space-y-2">
          {SCOPE_VALUES.map((val) => {
            const labelKey = val === 'map' ? 'scope_map_label' : val === 'map_players_tribes' ? 'scope_map_players_label' : 'scope_full_label'
            const descKey  = val === 'map' ? 'scope_map_desc'  : val === 'map_players_tribes' ? 'scope_map_players_desc'  : 'scope_full_desc'
            return (
              <label
                key={val}
                className="flex items-center gap-3 cursor-pointer p-2.5 rounded-md transition-colors"
                style={{
                  background: store.backupScope === val ? 'rgba(0,200,255,0.08)' : 'transparent',
                  border: `1px solid ${store.backupScope === val ? 'rgba(0,200,255,0.4)' : 'rgba(255,255,255,0.06)'}`,
                }}
              >
                <input
                  type="radio"
                  name="scope"
                  value={val}
                  checked={store.backupScope === val}
                  onChange={() => store.setBackupScope(val)}
                  className="accent-ark-cyan"
                />
                <div>
                  <p className="text-ark-cyan/80 text-sm font-semibold">{tk(labelKey, val)}</p>
                  <p className="text-ark-cyan/40 text-xs">{tk(descKey, '')}</p>
                </div>
              </label>
            )
          })}
        </div>
      </Section>

      <Section title={tk('section_saves_to_keep', 'Saves to Keep')}>
        <div className="flex items-center gap-4">
          <input
            type="range"
            min={1}
            max={10}
            value={store.maxSaves}
            onChange={(e) => store.setMaxSaves(Number(e.target.value))}
            className="flex-1 accent-ark-cyan"
          />
          <span className="text-ark-cyan/80 font-mono text-sm w-16 text-right">
            {store.maxSaves === 1 ? tk('only_last_save', 'Only the last') : tk('last_n_saves', 'Last {{n}}').replace('{{n}}', String(store.maxSaves))}
          </span>
        </div>
      </Section>
    </>
  )
}

/**
 * Convex plugin card — single button to connect via OAuth + push schema.
 *
 * The `invoke<string>(...)` calls Tauri commands registered by name in
 * `src-tauri/src/plugins/convex/mod.rs`. On success, the Tauri API
 * returns the OAuth URL the desktop should open in the default browser.
 * The browser redirects back to `http://127.0.0.1:8768/oauth/callback`
 * which the `http_api.rs` loopback (extended in Hito 12) intercepts.
 */
function ConvexCard() {
  const [status, setStatus] = useState<any>(null)
  const [busy, setBusy] = useState(false)
  const [msg, setMsg] = useState<string | null>(null)
  const refresh = useCallback(async () => {
    try { setStatus(await getConvexStatus()) } catch (e: any) { setMsg(String(e)) }
  }, [])
  useEffect(() => { void refresh() }, [refresh])

  const connect = async () => {
    setBusy(true); setMsg(null)
    try {
      const msg = await invoke<string>('begin_convex_link')
      setMsg(msg.includes('connected') ? msg : `${msg} — check the CLI window for the GitHub login screen.`)
    } catch (e: any) { setMsg(String(e)) }
    finally { setBusy(false) }
  }
  const push = async () => {
    setBusy(true); setMsg(null)
    try { const out = await pushConvex(); setMsg(out); refresh() }
    catch (e: any) { setMsg(String(e)) }
    finally { setBusy(false) }
  }

  return (
    <div className="rounded-md p-3 border border-ark-cyan/15">
      <div className="flex items-center justify-between mb-2">
        <div>
          <p className="text-ark-cyan/85 font-semibold">Convex BaaS</p>
          <p className="text-ark-cyan/40 text-xs">
            Backend that brokers our web admin and pushes every-server-state to the cloud.
          </p>
        </div>
        <PluginStatusBadge connected={status?.connected} />
      </div>
      <div className="flex gap-2">
        <button
          disabled={busy}
          onClick={connect}
          className="ark-action-btn px-4 py-1 text-xs disabled:opacity-40"
        >
          {status?.connected ? 'Reconnect' : 'Connect Convex'}
        </button>
        <button
          disabled={!status?.connected || busy}
          onClick={push}
          className="ark-action-btn px-4 py-1 text-xs disabled:opacity-40"
        >
          Push schema
        </button>
        {status?.deployment_url && (
          <span className="text-ark-cyan/40 text-xs font-mono truncate">
            {status.deployment_url}
          </span>
        )}
      </div>
      {msg && <p className="mt-2 text-ark-cyan/50 text-xs italic">{msg}</p>}
    </div>
  )
}

function VercelCard() {
  const [status, setStatus] = useState<any>(null)
  const [busy, setBusy] = useState(false)
  const [msg, setMsg] = useState<string | null>(null)
  const [token, setToken] = useState('')
  const [projectId, setProjectId] = useState('')
  const [oneClickBusy, setOneClickBusy] = useState(false)
  const [oneClickOutput, setOneClickOutput] = useState('')
  const refresh = useCallback(async () => {
    try { setStatus(await getVercelStatus()) } catch (e: any) { setMsg(String(e)) }
  }, [])
  useEffect(() => { void refresh() }, [refresh])
  const connect = async () => {
    setBusy(true); setMsg(null)
    try {
      const msg = await invoke<string>('begin_vercel_link')
      setMsg(msg.includes('connected') ? msg : `${msg} — check the CLI window for the vercel login browser.`)
    } catch (e: any) { setMsg(String(e)) }
    finally { setBusy(false) }
  }
  const deploy = async () => {
    setBusy(true); setMsg(null)
    try {
      const r = await deployVercel()
      setMsg(`deployed ${r.status}: ${r.url || '<see dashboard>'}`)
      refresh()
    } catch (e: any) { setMsg(String(e)) }
    finally { setBusy(false) }
  }
  const oneClickDeploy = async () => {
    if (!token.trim()) {
      setMsg('paste a Vercel token first — get one at https://vercel.com/account/tokens')
      return
    }
    setOneClickBusy(true)
    setMsg(null)
    setOneClickOutput('')
    try {
      const out = await invoke<string>('vercel_deploy_one_click', {
        token: token.trim(),
        projectId: projectId.trim() || null,
      })
      setOneClickOutput(out)
      // Try to extract the .vercel.app URL for the status badge link
      const url = out.split(/\s+/).find(t => t.startsWith('https://') && t.includes('.vercel.app')) ?? null
      if (url) {
        setStatus((s: any) => ({ ...s, last_deploy_url: url, connected: true }))
      }
      setMsg('One-click deploy complete. See output below.')
    } catch (e: any) {
      setOneClickOutput(String(e))
      setMsg(`One-click deploy failed: ${e}`)
    } finally {
      setOneClickBusy(false)
      refresh()
    }
  }
  return (
    <div className="rounded-md p-3 border border-ark-cyan/15">
      <div className="flex items-center justify-between mb-2">
        <div>
          <p className="text-ark-cyan/85 font-semibold">Vercel (web admin)</p>
          <p className="text-ark-cyan/40 text-xs">
            Hosts the public admin UI at <code className="text-ark-accent">ark-asa-admin.vercel.app</code>.
            Push schema first.
          </p>
        </div>
        <PluginStatusBadge connected={status?.connected} />
      </div>
      {/* Legacy CLI-based flow (kept as fallback) */}
      <div className="flex gap-2 mb-2">
        <button
          disabled={busy}
          onClick={connect}
          className="ark-action-btn px-4 py-1 text-xs disabled:opacity-40"
        >
          {status?.connected ? 'Reconnect' : 'Connect Vercel'}
        </button>
        <button
          disabled={!status?.connected || busy}
          onClick={deploy}
          className="ark-action-btn px-4 py-1 text-xs disabled:opacity-40"
        >
          Deploy web
        </button>
        {status?.last_deploy_url && (
          <a
            href={status.last_deploy_url}
            target="_blank"
            rel="noopener noreferrer"
            className="text-ark-cyan/60 text-xs font-mono truncate hover:text-ark-cyan"
          >
            {status.last_deploy_url}
          </a>
        )}
      </div>

      {/* ── One-click flow with pasted token (no CLI shell-out) ────────────── */}
      <details className="mt-2">
        <summary className="cursor-pointer text-ark-cyan/70 text-[11px] uppercase tracking-widest">
          ⚡ One-Click Deploy (paste a token)
        </summary>
        <div className="mt-3 space-y-2">
          <p className="text-ark-cyan/40 text-[10px]">
            Paste a Vercel token from <a href="https://vercel.com/account/tokens" target="_blank" rel="noopener" className="underline">vercel.com/account/tokens</a> and click DEPLOY. The app will save the token, run <code>vercel deploy --prod --yes</code>, and capture the production URL.
          </p>
          <input
            type="password"
            value={token}
            onChange={e => setToken(e.target.value)}
            placeholder="Vercel token (paste from dashboard)"
            className="w-full bg-black/40 border border-ark-cyan/15 rounded px-2 py-1.5 text-xs font-mono text-ark-cyan/90 placeholder:text-ark-cyan/30"
          />
          <input
            type="text"
            value={projectId}
            onChange={e => setProjectId(e.target.value)}
            placeholder="Optional: Vercel project id (pr_…) or project name"
            className="w-full bg-black/40 border border-ark-cyan/15 rounded px-2 py-1.5 text-xs font-mono text-ark-cyan/90 placeholder:text-ark-cyan/30"
          />
          <button
            disabled={oneClickBusy || !token.trim()}
            onClick={oneClickDeploy}
            className="ark-action-btn px-4 py-1.5 text-xs disabled:opacity-40"
            style={{ borderColor: 'rgba(0, 200, 255, 0.4)' }}
          >
            {oneClickBusy ? 'Deploying…' : 'DEPLOY'}
          </button>
          {oneClickOutput && (
            <pre className="text-[10px] bg-black/40 text-ark-cyan/60 p-3 rounded font-mono max-h-40 overflow-y-auto whitespace-pre-wrap leading-tight border border-ark-cyan/5">
              {oneClickOutput}
            </pre>
          )}
        </div>
      </details>

      {msg && <p className="mt-2 text-ark-cyan/50 text-xs italic">{msg}</p>}
    </div>
  )
}

function PluginStatusBadge({ connected }: { connected: boolean | undefined }) {
  return (
    <span
      className="text-[10px] uppercase tracking-widest px-2 py-0.5 rounded"
      style={{
        color:      connected ? 'rgba(74,222,128,0.9)' : 'rgba(239,68,68,0.9)',
        background: connected ? 'rgba(74,222,128,0.1)' : 'rgba(239,68,68,0.1)',
        border:     connected ? '1px solid rgba(74,222,128,0.4)' : 'rgba(239,68,68,0.4)',
      }}
    >
      {connected ? '● connected' : '○ not connected'}
    </span>
  )
}

/**
 * Tailscale wizard (v2.1, Network blocker #4) — surfaces the local
 * public-IP detection + CGNAT heuristic. When CGNAT is *suspected*
 * (no public IPv4 visible), the wizard offers a one-shot button to
 * run `tailscale up` against an auth key the operator pastes from
 * the Tailscale admin panel. The resulting `100.x.x.x` IP is
 * surfaced so the operator can plug it into the Web Admin player
 * connection entry.
 */
function TailscaleWizard() {
  const { tk } = useI18n()
  const [status, setStatus] = useState<{
    installed: boolean
    up: boolean
    ip: string | null
    hostname: string | null
    cgnat_suspect: boolean
    public_ip: string | null
    hint: string
  } | null>(null)
  const [downloadUrl, setDownloadUrl] = useState('')
  const [authKey, setAuthKey] = useState('')
  const [hostname, setHostname] = useState('')
  const [busy, setBusy] = useState(false)
  const [msg, setMsg] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    try {
      const s = await invoke<any>('tailscale_status_combined')
      setStatus(s)
      const url = await invoke<string>('tailscale_download_url')
      setDownloadUrl(url)
    } catch (e: any) {
      setMsg(String(e))
    }
  }, [])

  useEffect(() => { void refresh() }, [refresh])

  const setup = async () => {
    if (!authKey.trim()) {
      setMsg(tk('tailscale_missing_auth_key',
        'Paste an auth key — get one at https://login.tailscale.com/admin/settings/keys'))
      return
    }
    if (!hostname.trim()) {
      setMsg(tk('tailscale_missing_hostname',
        'Pick a hostname (e.g. arkasa-pi5) — this is the MagicDNS name your friends will use.'))
      return
    }
    setBusy(true)
    setMsg(null)
    try {
      const out = await invoke<any>('tailscale_setup', {
        authKey: authKey.trim(),
        hostname: hostname.trim(),
        publiclyDnsLabel: null,
      })
      setStatus(out)
      setMsg(out.hint ?? '')
    } catch (e: any) {
      setMsg(String(e))
    } finally {
      setBusy(false)
      refresh()
    }
  }

  const installTailscale = async () => {
    if (!downloadUrl) { await refresh(); return }
    try { await openExternal(downloadUrl) } catch { /* ignore */ }
  }

  const cgnat  = status?.cgnat_suspect ?? false
  const ip     = status?.ip ?? null
  const pub    = status?.public_ip ?? null
  const installed = status?.installed ?? false
  const up    = status?.up ?? false

  return (
    <div className="rounded-md p-3 border border-ark-cyan/15 space-y-3">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-ark-cyan/85 font-semibold">{tk('tailscale_title', 'Public network & Tailscale')}</p>
          <p className="text-ark-cyan/40 text-xs">
            {tk('tailscale_intro',
              'Detects if your ISP gives you a public IPv4 (port-forwarding works) or CGNAT-only (use Tailscale).')}
          </p>
        </div>
        <PluginStatusBadge connected={up} />
      </div>

      {/* 1. Status table */}
      <ul className="text-[11px] space-y-1 font-mono">
        <li className="flex justify-between text-ark-cyan/70">
          <span>Public IPv4</span>
          <span className={pub ? 'text-emerald-400' : 'text-amber-400'}>
            {pub ?? tk('tailscale_none', '(none)') }
          </span>
        </li>
        <li className="flex justify-between text-ark-cyan/70">
          <span>Tailscale CLI</span>
          <span className={installed ? 'text-emerald-400' : 'text-amber-400'}>
            {installed ? tk('tailscale_installed_yes', 'installed')
                       : tk('tailscale_installed_no',  'not installed')}
          </span>
        </li>
        <li className="flex justify-between text-ark-cyan/70">
          <span>Tailscale IP</span>
          <span className={ip ? 'text-emerald-400' : 'text-amber-400'}>
            {ip ?? tk('tailscale_none', '(none)')}
          </span>
        </li>
        <li className="flex justify-between text-ark-cyan/70">
          <span>CGNAT</span>
          <span className={cgnat ? 'text-amber-400' : 'text-emerald-400'}>
            {cgnat ? tk('tailscale_cgnat_yes', 'suspected')
                   : tk('tailscale_cgnat_no',  'no')}
          </span>
        </li>
      </ul>

      {/* 2. Hint box — when CGNAT or Tailscale missing */}
      {(cgnat || !installed) && (
        <div className="rounded border border-amber-400/30 bg-amber-400/5 p-2 text-[11px] space-y-1">
          <p className="text-amber-300">{status?.hint ?? tk('tailscale_probing', 'Probing status…')}</p>
          {!installed && downloadUrl && (
            <button
              onClick={installTailscale}
              className="text-ark-cyan/80 underline text-[11px] tracking-widest"
            >
              ↓ {tk('tailscale_install_btn', 'INSTALL TAILSCALE')}
            </button>
          )}
        </div>
      )}

      {/* 3. Setup form (only when installed but not up) */}
      {installed && !up && (
        <div className="space-y-2">
          <p className="text-ark-cyan/40 text-[10px]">
            {tk('tailscale_setup_intro',
              'Paste a one-off Auth Key and pick a Tailscale hostname. Click SET UP — the desktop app runs `tailscale up` via the official CLI.')}
          </p>
          <input
            type="password"
            value={authKey}
            onChange={e => setAuthKey(e.target.value)}
            placeholder="tskey-auth-…  (paste from Tailscale admin panel)"
            className="w-full bg-black/40 border border-ark-cyan/15 rounded px-2 py-1.5 text-xs font-mono text-ark-cyan/90 placeholder:text-ark-cyan/30"
          />
          <input
            type="text"
            value={hostname}
            onChange={e => setHostname(e.target.value)}
            placeholder="arkasa-pi5  (MagicDNS name)"
            className="w-full bg-black/40 border border-ark-cyan/15 rounded px-2 py-1.5 text-xs font-mono text-ark-cyan/90 placeholder:text-ark-cyan/30"
          />
          <button
            disabled={busy}
            onClick={setup}
            className="ark-action-btn px-4 py-1.5 text-xs disabled:opacity-40"
            style={{ borderColor: 'rgba(0, 200, 255, 0.4)' }}
          >
            {busy ? tk('tailscale_setting_up', 'Setting up…')
                  : tk('tailscale_setup_btn',   'SET UP TAILSCALE')}
          </button>
        </div>
      )}

      {/* 4. When up: show the IP, ready to use */}
      {up && ip && (
        <div className="rounded border border-emerald-400/30 bg-emerald-400/5 p-2 text-[11px] text-emerald-300 space-y-1">
          <p>{tk('tailscale_ready', '🟢 Tailscale is up. Share this IP with your friends:')}</p>
          <code className="font-mono text-emerald-200 text-sm select-all">{ip}</code>
          <p className="text-ark-cyan/45">
            {tk('tailscale_ready_friend',
              'Friends install Tailscale, you add them on your tailnet, they connect to <100.x.x.x> on UDP 7777.')}
          </p>
        </div>
      )}

      {/* 5. Manual refresh + last note */}
      <div className="flex items-center gap-2 pt-1">
        <button
          onClick={refresh}
          className="text-ark-cyan/60 hover:text-ark-cyan text-[10px] tracking-widest uppercase"
        >
          ↻ {tk('btn_refresh', 'REFRESH')}
        </button>
        <button
          onClick={() => openExternal('https://login.tailscale.com/admin/settings/keys')}
          className="text-ark-cyan/60 hover:text-ark-cyan text-[10px] tracking-widest uppercase"
        >
          → {tk('tailscale_get_key', 'GET AUTH KEY')}
        </button>
      </div>
      {msg && (
        <p className="text-ark-cyan/55 text-[10px] mt-1 font-mono whitespace-pre-wrap leading-tight">
          {msg}
        </p>
      )}
    </div>
  )
}
