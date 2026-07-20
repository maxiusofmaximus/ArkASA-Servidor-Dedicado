import { useState } from 'react'
import { useBackupStore, type BackupScope } from '../../stores/backupStore'
import { useConfigStore, type ConfigStore } from '../../stores/configStore'
import { useShallow } from 'zustand/react/shallow'
import { useI18n } from '../../i18n/useI18n'
import { Section, Toggle } from '../ui/OptionsUI'
import { useServerVersion } from '../../hooks/useServerVersion'
import { versionStatus } from '../../utils/versionStatus'
import { invoke } from '@tauri-apps/api/core'

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
      network:  s.config?.network,
      advanced: s.config?.advanced,
      config:   s.config,
      updateNetwork: (patch: Partial<NonNullable<typeof s.config>['network']>) => {
        if (!s.config) return
        s.setConfig({ ...s.config, network: { ...s.config.network, ...patch } })
      },
      updateAdvanced: (patch: Partial<NonNullable<typeof s.config>['advanced']>) => {
        if (!s.config) return
        s.setConfig({ ...s.config, advanced: { ...s.config.advanced, ...patch } })
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

      {/* ── Mundo / Inventario (v2.1) ───────────────────────────────── */}
      {/* Promoted to the top of the tab so the item-stack-size multiplier
          (X1..X10) and the per-resource overrides panel are the first
          controls the operator sees — most users want to bump stack
          sizes immediately after a fresh install. */}
      <Section title={tk('section_world', 'Mundo & Inventario')}>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex-1 pr-3">
              <p className="text-ark-cyan/80 text-sm">{tk('auto_save_title', 'Auto-guardado del mundo (minutos)')}</p>
              <p className="text-ark-cyan/40 text-xs mt-0.5">
                {tk('auto_save_desc',
                  'Frecuencia con que el server hace save al mundo entero. ARK ASA por defecto 15 min. Sube a 30–60 si tienes cluster grande y ves bajones tras cada saveworld; baja a 5 si quieres recuperación más granular tras caídas. 0 = guarda constantemente (no recomendable, mucho I/O).')
                }
              </p>
              <p className="text-ark-cyan/30 text-[10px] mt-0.5 italic">
                {tk('auto_save_hint',
                  'También se guarda al pulsar Stop (saveworld → doexit). Razón principal de los FPS drops durante juego es el GC del cluster + el coste del save — ajustar este valor es lo que más afecta la fluidez general.')
                }
              </p>
            </div>
            <input
              type="number"
              min={0}
              max={120}
              step={1}
              value={cfg.advanced?.auto_save_period_minutes ?? 15}
              onChange={(e) => {
                const v = Number(e.target.value);
                if (!Number.isFinite(v)) return;
                cfg.updateAdvanced({ auto_save_period_minutes: Math.max(0, Math.min(120, Math.round(v))) });
              }}
              className="w-24 bg-black/40 border border-ark-cyan/40 text-ark-cyan text-sm px-2 py-1 rounded font-mono"
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="flex-1 pr-3">
              <p className="text-ark-cyan/80 text-sm">{tk('global_stack_title', 'Apilable global (multiplicador de stacks)')}</p>
              <p className="text-ark-cyan/40 text-xs mt-0.5">
                {tk('global_stack_desc',
                  'Multiplica el stack base de cada ítem apilable. 1 = oficial ARK (carne 100, primemeat 40, stone 100, etc.). 2 duplica todo. Para tocar un recurso concreto usa la tabla de abajo.')}
              </p>
            </div>
            <input
              type="number"
              min={1}
              max={10}
              step={1}
              value={cfg.advanced?.item_stack_size_multiplier ?? 1}
              onChange={(e) => {
                const v = Number(e.target.value);
                if (!Number.isFinite(v)) return;
                cfg.updateAdvanced({ item_stack_size_multiplier: Math.max(1, Math.min(10, Math.round(v))) });
              }}
              className="w-20 bg-black/40 border border-ark-cyan/40 text-ark-cyan text-sm px-2 py-1 rounded font-mono"
            />
          </div>

          <StackOverridesRow />
        </div>
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
              <div className="flex-1 pr-3">
                <p className="text-ark-cyan/80 text-sm">{tk('fixed_ports_title', 'Consecutive cluster ports (per ARK ASA guide)')}</p>
                <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('fixed_ports_desc', 'When ON, each cluster map advances by 2 ports on Game (7777/7779/7781), +1 on Peer (always Game+1), +1 on Query (27015/27016/27017), +1 on RCON (27020/27021/27022). Recommended for the official ARK ASA cluster guide.')}</p>
                <p className="text-ark-cyan/30 text-[10px] mt-0.5 italic">{tk('fixed_ports_when_off', 'When OFF, ports are pinned to each map by FNV-1a hash of its name — each map gets the same port on every boot, but ports are NOT consecutive (e.g. Ragnarok may land on 8221/8219).')}</p>
              </div>
              <Toggle
                value={cfg.network?.fixed_port_assignment_per_map ?? false}
                onChange={(v) => cfg.updateNetwork({ fixed_port_assignment_per_map: v })}
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-ark-cyan/80 text-sm">{tk('cluster_failover_title', 'Allow secondary map to reclaim primary slot')}</p>
                <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('cluster_failover_desc', 'If the primary map fails to bind its UDP port in time, the next map launches on slot 0 instead. Runtime-only.')}</p>
              </div>
              <Toggle
                value={cfg.network?.cluster_failover_enabled ?? false}
                onChange={(v) => cfg.updateNetwork({ cluster_failover_enabled: v })}
              />
            </div>
            <div className="flex items-center justify-between pl-4">
              <div>
                <p className="text-ark-cyan/70 text-xs">{tk('cluster_failover_timeout_label', 'Primary-bind timeout (seconds)')}</p>
              </div>
              <input
                type="number"
                min={5}
                max={600}
                step={5}
                className="w-24 bg-black/40 border border-ark-cyan/40 text-ark-cyan text-sm px-2 py-1 rounded font-mono"
                value={cfg.network?.cluster_failover_timeout_sec ?? 60}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  if (!Number.isFinite(v)) return;
                  cfg.updateNetwork({ cluster_failover_timeout_sec: Math.max(5, Math.min(600, Math.round(v))) });
                }}
                disabled={!cfg.network?.cluster_failover_enabled}
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
      {/* ── Version sync (v2.1) ───────────────────────────────────────── */}
      <Section title={tk('section_version_sync', 'Version Sync (Steam buildid)')}>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-ark-cyan/80 text-sm">{tk('auto_update_title', 'Auto Update before Start')}</p>
              <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('auto_update_desc', 'When the local buildid is behind Steam\u2019s, update automatically before the server boots.')}</p>
            </div>
            <Toggle
              value={cfg.network?.auto_update_before_start ?? true}
              onChange={(v) => cfg.updateNetwork({ auto_update_before_start: v })}
            />
          </div>
          <UpdateNowCard />
        </div>
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

      <Section title={tk('section_diag_repair', 'Diagnostics & repair — in-game server list')}>
        <DiagRepairCard />
      </Section>
    </>
  )
}

/**
 * Version Sync card — surfaces the same `useServerVersion` data the
 * top-bar badge uses, but here we expose an explicit `[ Update Now ]`
 * button so the operator can force a SteamCMD update even when auto
 * update is off.
 */
function UpdateNowCard() {
  const { tk } = useI18n()
  const cfg = useConfigStore(
    useShallow((s: ConfigStore) => ({
      config: s.config,
    })),
  )
  const version = useServerVersion(cfg.config)
  const status = versionStatus(version.info)
  const local  = version.info?.local_buildid  ?? null
  const latest = version.info?.latest_buildid ?? null

  const [busy, setBusy]   = useState(false)
  const [msg,  setMsg]    = useState<string | null>(null)

  const runUpdate = async () => {
    if (busy || version.updating) return
    setBusy(true); setMsg(null)
    try {
      // Delegate to the same hook used by the top-bar badge — that path
      // shares the in-flight single-flight lock + 30s debounce window so
      // we can never fork two steamcmd subprocesses for the same click.
      const out = await version.runUpdate()
      setMsg(typeof out === 'string' ? out : 'Update OK')
    } catch (e: unknown) {
      const text = e instanceof Error ? e.message : String(e)
      setMsg(text)
    } finally {
      setBusy(false)
    }
  }

  const { color, label, dot } = ((): { color: string; label: string; dot: string } => {
    switch (status) {
      case 'current':  return { color: 'rgb(74,222,128)',  label: 'Up to date',  dot: '🟢' }
      case 'outdated': return { color: 'rgb(248,113,113)', label: 'Outdated',    dot: '🔴' }
      default:         return { color: 'rgb(148,163,184)', label: 'No manifest', dot: '⚪' }
    }
  })()

  return (
    <div className="rounded-md p-3 border border-ark-cyan/15 space-y-2">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-ark-cyan/85 font-semibold">
            <span className="mr-1">{dot}</span>
            {tk('version_status', label)}
          </p>
          <p className="text-ark-cyan/40 text-xs font-mono">
            {local  && <>local <span style={{ color }}>v{local}</span>&nbsp;·&nbsp;</>}
            {latest && <>latest <span style={{ color }}>v{latest}</span></>}
            {!local && !latest && (tk('version_unknown_desc', 'SteamCMD did not report a buildid. Is SteamCMD installed at the configured path?'))}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            disabled={busy || version.loading || version.updating}
            onClick={() => void version.refresh()}
            className="text-ark-cyan/70 hover:text-ark-cyan text-[10px] tracking-widest uppercase disabled:opacity-40"
          >
            ↻ {tk('btn_refresh', 'REFRESH')}
          </button>
          <button
            type="button"
            disabled={busy || version.updating || !cfg.config}
            onClick={runUpdate}
            className="ark-action-btn px-4 py-1.5 text-xs disabled:opacity-40"
            style={{ borderColor: color + '60' }}
          >
            {busy || version.updating ? tk('updating', 'Updating…')
                  : tk('btn_update_now', 'UPDATE NOW')}
          </button>
        </div>
      </div>
      {msg && (
        <p className="text-ark-cyan/55 text-[10px] mt-1 font-mono whitespace-pre-wrap leading-tight">
          {msg}
        </p>
      )}
    </div>
  )
}

/**
 * Stack overrides - operator add/remove per-item stack sizes.
 * Each row maps an ARK item class string (`PrimalItemConsumable_…`)
 * to an absolute stack size. Persister emits
 * `ConfigOverrideItemMaxQuantity=(ItemClassString="…",Quantity=(MaxItemQuantity=N,bIgnoreMultiplier=True))`
 * into Game.ini via the format documented on ark.wiki.gg Server
 * configuration → Game.ini → Item related.
 */
function StackOverridesRow() {
  const { tk } = useI18n()
  const cfg = useConfigStore(
    useShallow((s: ConfigStore) => ({
      overrides: s.config?.advanced?.item_stack_overrides ?? {},
      updateAdvanced: (patch: Partial<NonNullable<typeof s.config>['advanced']>) => {
        if (!s.config) return
        s.setConfig({ ...s.config, advanced: { ...s.config.advanced, ...patch } })
      },
    })),
  )

  const PRESETS: { key: string; label: string; default_qty: number }[] = [
    { key: 'PrimalItemConsumable_RawMeat_C',        label: 'Raw Meat',          default_qty: 200 },
    { key: 'PrimalItemConsumable_RawPrimeMeat_C',   label: 'Raw Prime Meat',    default_qty: 600 },
    { key: 'PrimalItemConsumable_CookedMeat_C',     label: 'Cooked Meat',       default_qty: 200 },
    { key: 'PrimalItemConsumable_CookedPrimeMeat_C',label: 'Cooked Prime Meat', default_qty: 600 },
    { key: 'PrimalItemConsumable_Hide_C',           label: 'Hide',              default_qty: 500 },
    { key: 'PrimalItemResource_Stone_C',            label: 'Stone',             default_qty: 500 },
    { key: 'PrimalItemResource_Wood_C',             label: 'Wood',              default_qty: 500 },
    { key: 'PrimalItemResource_Thatch_C',           label: 'Thatch',            default_qty: 500 },
    { key: 'PrimalItemResource_Metal_C',            label: 'Metal',             default_qty: 500 },
    { key: 'PrimalItemResource_Obsidian_C',         label: 'Obsidian',          default_qty: 300 },
    { key: 'PrimalItemResource_Crystal_C',          label: 'Crystal',           default_qty: 300 },
    { key: 'PrimalItemResource_Flint_C',            label: 'Flint',             default_qty: 500 },
    { key: 'PrimalItemConsumable_Berry_Base_C',     label: 'Berries',           default_qty: 500 },
    { key: 'PrimalItemConsumable_Veggie_Base_C',    label: 'Vegetables',        default_qty: 500 },
    { key: 'PrimalItemConsumable_Meat_Base_C',      label: 'Meat (any)',        default_qty: 200 },
  ]

  const updateQty = (key: string, value: number) => {
    const next = { ...cfg.overrides }
    if (!Number.isFinite(value) || value <= 0) {
      delete next[key]
    } else {
      next[key] = Math.max(1, Math.min(9999, Math.round(value)))
    }
    cfg.updateAdvanced({ item_stack_overrides: next })
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-ark-cyan/80 text-sm">{tk('stack_overrides_title', 'Apilables custom por recurso')}</p>
          <p className="text-ark-cyan/40 text-xs mt-0.5">
            {tk('stack_overrides_desc',
              'Override absoluto (no multiplicador) para un ítem concreto. Útil para evitar que carne, primemeat, piel o piedra te llenen el inventario a media recolección.')}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-1.5">
        {PRESETS.map((preset) => {
          const qty = cfg.overrides[preset.key]
          const enabled = typeof qty === 'number'
          return (
            <button
              key={preset.key}
              onClick={() => updateQty(preset.key, enabled ? 0 : preset.default_qty)}
              className="rounded px-2 py-1 text-left transition-colors"
              style={{
                background: enabled ? 'rgba(0,200,255,0.12)' : 'rgba(255,255,255,0.03)',
                border: `1px solid ${enabled ? 'rgba(0,200,255,0.4)' : 'rgba(255,255,255,0.07)'}`,
              }}
              title={preset.key}
            >
              <span className="text-ark-cyan/85 text-[11px] font-semibold">{preset.label}</span>
              {enabled && (
                <span className="text-[9px] font-mono ml-1" style={{ color: 'rgba(0,200,255,0.9)' }}>
                  ×{qty}
                </span>
              )}
            </button>
          )
        })}
      </div>

      {Object.keys(cfg.overrides).length > 0 && (
        <div className="mt-2 rounded-md p-2" style={{ border: '1px solid rgba(0,200,255,0.15)' }}>
          <p className="text-ark-cyan/55 text-[10px] font-bold tracking-widest uppercase mb-1">
            {Object.keys(cfg.overrides).length} override{Object.keys(cfg.overrides).length === 1 ? '' : 's'} activas
          </p>
          <div className="space-y-1">
            {Object.entries(cfg.overrides)
              .sort(([a], [b]) => a.localeCompare(b))
              .map(([cls, qty]) => {
                const label = PRESETS.find((p) => p.key === cls)?.label ?? cls
                return (
                  <div key={cls} className="flex items-center gap-2 text-[11px] font-mono">
                    <span className="flex-1 truncate text-ark-cyan/70">
                      {label}
                      {label !== cls && <span className="text-ark-cyan/30"> · {cls}</span>}
                    </span>
                    <input
                      type="number"
                      min={1}
                      max={9999}
                      step={50}
                      value={qty}
                      onChange={(e) => updateQty(cls, Number(e.target.value))}
                      className="w-20 bg-black/40 border border-ark-cyan/30 text-ark-cyan text-[11px] px-2 py-0.5 rounded font-mono"
                    />
                    <button
                      onClick={() => updateQty(cls, 0)}
                      className="text-ark-cyan/45 hover:text-rose-400 text-[11px]"
                      title="Quitar override"
                    >×</button>
                  </div>
                )
              })}
          </div>
        </div>
      )}
    </div>
  )
}

/**
 * Diagnóstico y reparación de la lista in-game — cubre las tres causas
 * documentadas en `docs/TROUBLESHOOTING.md` "El servidor no aparece en la
 * lista in-game tras una actualización de ARK":
 *   1. Falta `[Internationalization] Culture=en` en GameUserSettings.ini
 *   2. Certificado EOS `Amazon RSA 2048 M02` no instalado en Trusted Root
 *   3. Steam build-id desactualizado (sólo informativo, no se auto-ejecuta)
 *
 * Bound to the Tauri command `diagnose_server_list` in
 * `src-tauri/src/ark/diagnostics.rs`. When `repair=true`, the command
 * fixes what it can (#1 and #2). Build-mismatch is always informational
 * — running SteamCMD `validate` mid-session would risk wiping running
 * saves, so we surface the command for the operator to run manually.
 */
interface DiagCheck {
  key: string
  label: string
  status: 'ok' | 'missing' | 'stale' | 'fixed' | 'error' | 'skipped'
  detail: string
  repaired: boolean
}
interface DiagReport {
  checks: DiagCheck[]
  summary: string
  overall_ok: boolean
}

const STATUS_ICON: Record<DiagCheck['status'], string> = {
  ok: '✓',
  missing: '⚠',
  stale: '⚠',
  fixed: '✓',
  error: '✗',
  skipped: '–',
}

/**
 * Build the actionable summary line in the project's runtime UI language,
 * using the canonical English fallback + dictionary override per the
 * i18n convention (`tk(key, English)`).
 *
 * The Rust `diagnose_server_list` returns an English `summary`; we don't
 * display that string directly because it bypasses the operator's UI
 * language. Instead we count the fixable vs. OK markers ourselves and
 * surface a localized summary via dictionary keys with {{n}} interpolation.
 */
function DiagRepairCard() {
  const { tk } = useI18n()
  const cfg = useConfigStore(
    useShallow((s: ConfigStore) => ({ config: s.config })),
  )
  const [busyDiag, setBusyDiag] = useState(false)
  const [busyRepair, setBusyRepair] = useState(false)
  const [report, setReport] = useState<DiagReport | null>(null)
  const [errMsg, setErrMsg] = useState<string>('')

  const run = async (repair: boolean) => {
    const c = cfg.config
    if (!c) return
    setErrMsg('')
    if (repair) setBusyRepair(true); else setBusyDiag(true)
    try {
      const r = await invoke<DiagReport>('diagnose_server_list', {
        serverDir:    c.paths.server_dir,
        steamCmdDir:  c.paths.steam_cmd_dir,
        repair,
      })
      setReport(r)
    } catch (e: any) {
      setErrMsg(typeof e === 'string' ? e : (e?.message ?? String(e)))
    } finally {
      setBusyDiag(false); setBusyRepair(false)
    }
  }

  // Show a brief inline hint next to the buttons when a known-blocking
  // issue is detected on the read-only diagnostic pass.
  const hasCultureIssue = report?.checks.some(c => c.key === 'culture_en' && c.status !== 'ok')
  const hasCertIssue    = report?.checks.some(c => c.key === 'eos_cert'    && c.status !== 'ok' && c.status !== 'fixed')

  return (
    <div className="rounded-md p-3 border border-ark-cyan/15 space-y-3">
      <div className="text-ark-cyan/55 text-[11px] leading-snug">
        {tk('diag_help',
          'If your server works by direct IP but does not show up in the in-game browser after an ARK update, this detects and auto-fixes the most common causes: missing [Internationalization] Culture=en block, expired EOS trust-root certificate, and out-of-date Steam build-id.')}
      </div>

      <div className="flex items-center gap-2">
        <button
          disabled={busyDiag || busyRepair || !cfg.config}
          onClick={() => void run(false)}
          className="ark-action-btn px-3 py-1.5 text-[10px] tracking-widest disabled:opacity-40"
        >
          {busyDiag ? tk('diag_running', 'Diagnosing…') : tk('btn_diag', 'DIAGNOSE')}
        </button>
        <button
          disabled={busyDiag || busyRepair || !cfg.config}
          onClick={() => void run(true)}
          className="ark-action-btn px-3 py-1.5 text-[10px] tracking-widest disabled:opacity-40"
          style={{ borderColor: 'rgba(0,212,255,0.6)' }}
        >
          {busyRepair ? tk('repair_running', 'Repairing…') : tk('btn_repair', 'REPAIR ALL')}
        </button>
        {report && report.overall_ok && !busyRepair && (
          <span className="text-emerald-400 text-[10px] tracking-widest">
            ✓ {tk('diag_all_ok', 'All OK')}
          </span>
        )}
      </div>

      {(hasCultureIssue || hasCertIssue) && !busyRepair && (
        <div className="text-amber-400/90 text-[11px] leading-snug">
          {tk('diag_advice_repair',
            'A known issue preventing the server from appearing in the in-game list was detected. Click "REPAIR ALL", then restart the server.')}
        </div>
      )}

      {errMsg && (
        <p className="text-red-400/80 text-[11px] flex items-center gap-1">
          <span>⚠</span> {errMsg}
        </p>
      )}

      {report && (
        <>
          <p className="text-ark-cyan/85 text-[11px] font-semibold tracking-wide">
            {(() => {
              const okCount  = report.checks.filter(c => c.status === 'ok').length
              const badCount  = report.checks.length - okCount
              const fixed     = report.checks.filter(c => c.repaired).length
              if (report.overall_ok) {
                return tk('diag_summary_ok', '{{n}} check(s) OK.')
                        .replace('{{n}}', String(okCount))
              }
              if (fixed > 0) {
                return tk('diag_summary_partial',
                          '{{fixed}} fix(es) applied, {{bad}} still need attention. Restart the server and re-run diagnostics.')
                        .replace('{{fixed}}', String(fixed))
                        .replace('{{bad}}', String(badCount))
              }
              return tk('diag_summary_bad', '{{ok}} check(s) OK, {{bad}} need repair.')
                      .replace('{{ok}}', String(okCount))
                      .replace('{{bad}}', String(badCount))
            })()}
          </p>
          <pre className="text-[10px] bg-black/40 text-ark-cyan/70 p-3 rounded font-mono max-h-72 overflow-y-auto whitespace-pre-wrap leading-tight border border-ark-cyan/5">
{report.checks.map(c => {
  const icon = STATUS_ICON[c.status] ?? '?'
  const repairedLabel = c.repaired ? `     ↳ ${tk('diag_fixed_marker', '(fixed)')}\n` : ''
  const labelKey = `diag_label_${c.key}`
  const labelTxt = tk(labelKey, c.label)
  return `  ${icon}  [${c.status.toUpperCase().padEnd(7)}]  ${labelTxt}\n` +
         `     ${c.detail}\n` +
         repairedLabel +
         `\n`
}).join('')}
{report.overall_ok
  ? `\n✓ ${tk('diag_report_ok', 'Diagnostics OK — the server should appear in the list.')}`
  : `\n⚠ ${tk('diag_report_pending', 'Pending verification — click "REPAIR ALL" and restart the server.')}`}
</pre>

          {!report.overall_ok && (
            <p className="text-ark-cyan/40 text-[10px] leading-snug">
              {tk('diag_repair_hint',
                'After "REPAIR ALL": stop the server (START SERVER → STOP SERVER) and start it again so ARK re-reads the INI and re-registers with EOS.')}
            </p>
          )}
        </>
      )}
    </div>
  )
}
