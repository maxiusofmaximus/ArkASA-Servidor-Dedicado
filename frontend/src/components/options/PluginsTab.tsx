import { useState, useEffect, useCallback, useMemo } from 'react'
import type { ServerConfig } from '../../types'
import { useConfigStore, type ConfigStore } from '../../stores/configStore'
import { useShallow } from 'zustand/react/shallow'
import { useI18n } from '../../i18n/useI18n'
import { Section } from '../ui/OptionsUI'
import { invoke } from '../../services/tauri'
import {
  loadPluginCatalog,
  type CatalogCard,
  type ConnectionPluginView,
  type ModelPluginView,
  type PluginCatalogView,
} from '../../utils/pluginCatalog'

// ─────────────────────────────────────────────────────────────────────────────
// TS interfaces — MUST mirror the Rust serde output exactly.
//
//   CatalogEntryView        (pluginhub.rs:51)                has `rename_all =
//   "camelCase"`, so fields arrive as `requiredSecrets`, `oauthUrl`,
//   `hasRequiredSecrets`.
//
//   ConnectionPluginJson    (connection.rs:239)              has `rename_all =
//   "camelCase"` — `freeTier`, `requiresCli`, `requiresCredentials`, `docsUrl`.
//
//   ModelPluginJson         (model.rs:174)                   has `rename_all =
//   "camelCase"` — `defaultBaseUrl`, `defaultModel`, `requiresApiKey`,
//   `isLocal`, `installHint`, `docsUrl`.
//
// Optional `?:` + `?.`/`??` defense so a missing backend field never crashes
// the grid (the previous crash was here on `requiresCli.length`).
// ─────────────────────────────────────────────────────────────────────────────

type FilterChip = 'all' | 'installed' | 'available'

interface PluginsTabProps {
  config: ServerConfig | null
}

// ─────────────────────────────────────────────────────────────────────────────

export default function PluginsTab({ config }: PluginsTabProps) {
  const { tk } = useI18n()
  const { updateConfig } = useConfigStore(useShallow((s: ConfigStore) => ({ updateConfig: s.updateConfig })))

  const [cards,    setCards]    = useState<CatalogCard[]>([])
  const [loading,  setLoading]  = useState(true)
  const [filter,   setFilter]   = useState<FilterChip>('all')
  const [busy,     setBusy]     = useState<string | null>(null)
  const [error,    setError]    = useState<string | null>(null)

  // ── Parallel fetch of the three catalogs with partial-failure reporting ─
  const refresh = useCallback(async () => {
    if (!config) {
      setCards([])
      setError(null)
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      const result = await loadPluginCatalog(invoke, config)
      setCards(result.cards)
      if (result.errors.length > 0) {
        setError(result.errors.map((item) => `${item.command}: ${item.message}`).join(' · '))
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }, [config])

  useEffect(() => { void refresh() }, [refresh])

  // ── Handlers ──────────────────────────────────────────────────────────
  // Single source of truth: mutate `config.installed_plugins` array, pipe
  // through `updateConfig` -> `useAutoSave` (1.5s) -> `save_config`. We do
  // NOT invoke `set_plugin_installed`; that backend command is the
  // fallback API for external integrations and CLI.
  const installPlugin = useCallback((id: string) => {
    if (!config) return
    const next = Array.from(new Set([...config.installed_plugins, id])).sort()
    updateConfig({ installed_plugins: next })
    // optimistic local flip so the card re-paints immediately
    setCards((prev) => prev.map((c) =>
      c.kind === 'chat' && c.data.id === id ? { kind: 'chat', data: { ...c.data, installed: true } } : c
    ))
  }, [config, updateConfig])

  const uninstallPlugin = useCallback(async (id: string) => {
    if (!config) return
    const next = config.installed_plugins.filter((x) => x !== id).sort()
    updateConfig({ installed_plugins: next })
    // Chat plugins have a runtime registry; connection plugins such as
    // Tailscale are configured by their owning connection screen.
    const existing = cards.find((card) => card.data.id === id)
    if (existing?.kind === 'chat') {
      try { await invoke('disable_plugin', { id }) } catch { /* swallow */ }
    }
    setCards((prev) => prev.map((c) =>
      c.kind === 'chat' && c.data.id === id
        ? { kind: 'chat', data: { ...c.data, installed: false, enabled: false } }
        : c
    ))
  }, [cards, config, updateConfig])

  const toggleEnabled = useCallback(async (id: string, currentlyOn: boolean) => {
    setBusy(id)
    try {
      if (currentlyOn) {
        await invoke('disable_plugin', { id })
      } else {
        await invoke('enable_plugin', { id })
      }
      setCards((prev) => prev.map((c) =>
        c.kind === 'chat' && c.data.id === id
          ? { kind: 'chat', data: { ...c.data, enabled: !currentlyOn } }
          : c
      ))
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }, [])

  // ── Filtering ─────────────────────────────────────────────────────────
  const visibleCards = useMemo(() => {
    if (filter === 'all') return cards
    if (filter === 'installed') {
      return cards.filter((c) => isCardInstalled(c, config))
    }
    // 'available'
    return cards.filter((c) => !isCardInstalled(c, config))
  }, [cards, config, filter])

  const installedCount = useMemo(
    () => cards.filter((c) => isCardInstalled(c, config)).length,
    [cards, config],
  )
  const availableCount = useMemo(
    () => cards.filter((c) => !isCardInstalled(c, config)).length,
    [cards, config],
  )

  // ── Render helpers ────────────────────────────────────────────────────
  const chipStyle = (active: boolean) => ({
    background: active ? 'rgba(0,200,255,0.14)' : 'rgba(255,255,255,0.03)',
    border: `1px solid ${active ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.08)'}`,
    color: active ? 'rgba(0,200,255,0.9)' : 'rgba(255,255,255,0.45)',
  })

  return (
    <>
      <Section title={tk('section_plugin_hub', 'Plugin Hub')}>
        <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
          {tk('plugin_marketplace_intro',
            'Browse chat backends, connection providers, and AI model adapters. Install the ones you want in your setup; configure them at runtime.')}
        </p>

        {/* Filter chips */}
        <div className="flex gap-1.5 flex-wrap items-center mt-3">
          <button
            onClick={() => setFilter('all')}
            className="px-3 py-1.5 text-[10px] font-bold tracking-widest uppercase rounded-md transition-colors"
            style={chipStyle(filter === 'all')}
          >
            {tk('plugin_filter_all', 'Todos')} ({cards.length})
          </button>
          <button
            onClick={() => setFilter('installed')}
            className="px-3 py-1.5 text-[10px] font-bold tracking-widest uppercase rounded-md transition-colors"
            style={chipStyle(filter === 'installed')}
          >
            {tk('plugin_filter_installed', 'Instalados')} ({installedCount})
          </button>
          <button
            onClick={() => setFilter('available')}
            className="px-3 py-1.5 text-[10px] font-bold tracking-widest uppercase rounded-md transition-colors"
            style={chipStyle(filter === 'available')}
          >
            {tk('plugin_filter_available', 'Disponibles')} ({availableCount})
          </button>
        </div>

        {error && (
          <p className="text-red-400/80 text-xs mt-3">{error}</p>
        )}

        {loading && (
          <p className="text-ark-cyan/30 text-xs italic mt-3">{tk('loading', 'Loading…')}</p>
        )}

        {/* Unified grid — anti-overflow minmax(0, 1fr) per-row your batch pattern */}
        <div
          className="mt-3"
          style={{
            display: 'grid',
            // min(max(280px, 100%)) protects narrow viewports from overflow.
            gridTemplateColumns: 'repeat(auto-fill, minmax(min(280px, 100%), 1fr))',
            gap: '12px',
          }}
        >
          {visibleCards.map((card) => (
            <Card
              key={`${card.kind}-${card.data.id}`}
              card={card}
              busy={busy}
              installed={isCardInstalled(card, config)}
              onInstall={installPlugin}
              onUninstall={uninstallPlugin}
              onToggle={toggleEnabled}
              tk={tk}
            />
          ))}
        </div>

        {!loading && visibleCards.length === 0 && (
          <p className="text-ark-cyan/30 text-xs italic mt-3">
            {tk('plugin_no_results', 'No hay plugins para este filtro.')}
          </p>
        )}
      </Section>
    </>
  )
}

function isCardInstalled(card: CatalogCard, config: ServerConfig | null): boolean {
  if (card.kind === 'chat') return card.data.installed
  if (card.kind === 'conn') return config?.installed_plugins.includes(card.data.id) ?? false
  return false
}

// ─────────────────────────────────────────────────────────────────────────────
// Card — one per plugin; renders only the actions/badges relevant to its kind.
// ─────────────────────────────────────────────────────────────────────────────

interface CardProps {
  card: CatalogCard
  busy: string | null
  installed: boolean
  onInstall: (id: string) => void
  onUninstall: (id: string) => Promise<void>
  onToggle: (id: string, currentlyOn: boolean) => Promise<void>
  tk: (key: string, fallback?: string) => string
}

function Card({ card, busy, installed, onInstall, onUninstall, onToggle, tk }: CardProps) {
  const { data } = card
  const isBusy = busy === data.id

  // Kind chip
  const kindLabel =
    card.kind === 'chat'  ? tk('plugin_kind_chat',  'CHAT BACKEND')
    : card.kind === 'conn' ? tk('plugin_kind_conn',  'CONNECTION')
    : tk('plugin_kind_model', 'AI MODEL')

  return (
    <div
      className="rounded-md px-3 py-2.5 transition-colors flex flex-col gap-2 min-w-0"
      style={{
        background: card.kind === 'chat' && card.data.installed
          ? 'rgba(74,222,128,0.06)'
          : 'rgba(255,255,255,0.02)',
        border: card.kind === 'chat' && card.data.installed
          ? '1px solid rgba(74,222,128,0.25)'
          : '1px solid rgba(255,255,255,0.08)',
      }}
    >
      {/* Header: label + kind chip */}
      <div className="flex items-start justify-between gap-2 min-w-0">
        <div className="flex-1 min-w-0">
          <p
            className="font-semibold text-ark-cyan/90 text-sm truncate"
            title={data.label}
          >
            {data.label}
          </p>
          <p className="text-ark-cyan/40 text-[10px] font-mono truncate" title={data.id}>
            {data.id}
          </p>
        </div>
        <span
          className="text-[9px] tracking-widest px-2 py-0.5 rounded flex-shrink-0"
          style={{
            color: 'rgba(0,200,255,0.7)',
            background: 'rgba(0,200,255,0.06)',
            border: '1px solid rgba(0,200,255,0.25)',
          }}
        >
          {kindLabel}
        </span>
      </div>

      {/* Per-kind body */}
      {card.kind === 'chat' && (
        <ChatBody card={card} busy={isBusy} onInstall={onInstall} onUninstall={onUninstall} onToggle={onToggle} tk={tk} />
      )}
      {card.kind === 'conn' && (
        <ConnBody
          card={card}
          installed={installed}
          busy={isBusy}
          onInstall={onInstall}
          onUninstall={onUninstall}
          tk={tk}
        />
      )}
      {card.kind === 'model' && <ModelBody card={card} />}
    </div>
  )
}

// ── Chat backend body — Install/Uninstall + ON/OFF + secrets warning ────
function ChatBody({ card, busy, onInstall, onUninstall, onToggle, tk }: {
  card: { kind: 'chat'; data: PluginCatalogView }
  busy: boolean
  onInstall: (id: string) => void
  onUninstall: (id: string) => Promise<void>
  onToggle: (id: string, currentlyOn: boolean) => Promise<void>
  tk: (key: string, fallback?: string) => string
}) {
  const { data: p } = card
  const missingSecrets =
    (p.requiredSecrets?.length ?? 0) > 0 && !p.hasRequiredSecrets

  return (
    <>
      <p className="text-ark-cyan/55 text-[10px] leading-relaxed">
        {p.channel} · {(p.capabilities ?? []).join(', ') || '—'}
      </p>

      {missingSecrets && (
        <p className="text-amber-400 text-[10px] mt-0.5">
          ⚠ {tk('plugin_missing_secrets', 'Missing required secrets: ') +
            (p.requiredSecrets ?? []).join(', ')}
        </p>
      )}

      {/* State chips: INSTALADO + ENCENDIDO/APAGADO */}
      <div className="flex gap-1.5 flex-wrap">
        {p.installed && (
          <span
            className="text-[9px] tracking-widest px-2 py-0.5 rounded"
            style={{
              color: 'rgba(74,222,128,0.9)',
              background: 'rgba(74,222,128,0.08)',
              border: '1px solid rgba(74,222,128,0.3)',
            }}
          >
            {tk('plugin_installed_chip', 'INSTALADO')}
          </span>
        )}
        <span
          className="text-[9px] tracking-widest px-2 py-0.5 rounded"
          style={
            p.enabled
              ? { color: 'rgba(74,222,128,0.9)', background: 'rgba(74,222,128,0.08)', border: '1px solid rgba(74,222,128,0.3)' }
              : { color: 'rgba(255,255,255,0.4)', background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.1)' }
          }
        >
          {p.enabled ? tk('plugin_enabled', '● ON') : tk('plugin_disabled', '○ OFF')}
        </span>
      </div>

      {/* Actions row */}
      <div className="flex gap-1.5 mt-1">
        {p.installed ? (
          <button
            onClick={() => void onUninstall(p.id)}
            disabled={busy}
            className="ark-action-btn text-[10px] px-2.5 py-1 disabled:opacity-40"
            style={{ borderColor: 'rgba(255,120,120,0.4)' }}
          >
            {busy ? tk('saving', 'Saving...') : tk('plugin_uninstall', 'UNINSTALL')}
          </button>
        ) : (
          <button
            onClick={() => onInstall(p.id)}
            disabled={busy}
            className="ark-action-btn text-[10px] px-2.5 py-1 disabled:opacity-40"
          >
            {busy ? tk('saving', 'Saving...') : tk('plugin_install', 'INSTALL')}
          </button>
        )}
        <button
          onClick={() => void onToggle(p.id, p.enabled)}
          disabled={!p.installed || busy}
          className="ark-action-btn text-[10px] px-2.5 py-1 disabled:opacity-25"
          title={!p.installed ? tk('plugin_enable_blocked', 'Instala primero el plugin para poder encenderlo.') : ''}
          style={p.enabled ? { borderColor: 'rgba(74,222,128,0.4)' } : {}}
        >
          {p.enabled ? tk('plugin_turn_off', 'Apagar') : tk('plugin_turn_on', 'Encender')}
        </button>
      </div>
    </>
  )
}

// ── Connection provider body — install marker + badges ─────────────────
function ConnBody({ card, installed, busy, onInstall, onUninstall, tk }: {
  card: { kind: 'conn'; data: ConnectionPluginView }
  installed: boolean
  busy: boolean
  onInstall: (id: string) => void
  onUninstall: (id: string) => Promise<void>
  tk: (key: string, fallback?: string) => string
}) {
  const { data: c } = card
  return (
    <>
      <p className="text-ark-cyan/55 text-[10px] leading-relaxed">{c.description}</p>
      {(c.requiresCli?.length ?? 0) > 0 && (
        <p className="text-ark-cyan/45 text-[10px]">
          {tk('plugin_cli_required', 'CLI: ')}
          <code className="font-mono text-ark-accent">
            {(c.requiresCli ?? []).join(', ')}
          </code>
        </p>
      )}
      <div className="flex gap-1.5 flex-wrap mt-0.5">
        {c.freeTier && (
          <span className="text-[9px] tracking-widest px-2 py-0.5 rounded"
            style={{
              color: 'rgba(74,222,128,0.9)', background: 'rgba(74,222,128,0.08)',
              border: '1px solid rgba(74,222,128,0.3)',
            }}>FREE TIER</span>
        )}
        {c.requiresCredentials && (
          <span className="text-[9px] tracking-widest px-2 py-0.5 rounded"
            style={{
              color: 'rgba(0,200,255,0.9)', background: 'rgba(0,200,255,0.08)',
              border: '1px solid rgba(0,200,255,0.3)',
            }}>{tk('plugin_needs_creds', 'NEEDS CREDENTIALS')}</span>
        )}
      </div>
      {c.id === 'tailscale' && (
        <div className="flex items-center gap-2 mt-1.5">
          <span className="text-ark-cyan/45 text-[10px]">
            {installed ? '● selected' : '○ not selected'}
          </span>
          {installed ? (
            <button
              onClick={() => void onUninstall(c.id)}
              disabled={busy}
              className="ark-action-btn text-[10px] px-2.5 py-1 disabled:opacity-40"
              style={{ borderColor: 'rgba(255,120,120,0.4)' }}
            >
              {busy ? tk('saving', 'Saving...') : tk('plugin_uninstall', 'UNSELECT')}
            </button>
          ) : (
            <button
              onClick={() => onInstall(c.id)}
              disabled={busy}
              className="ark-action-btn text-[10px] px-2.5 py-1 disabled:opacity-40"
            >
              {busy ? tk('saving', 'Saving...') : tk('plugin_install', 'SELECT PLUGIN')}
            </button>
          )}
        </div>
      )}
    </>
  )
}

// ── AI model body — read-only, local/API badges + default URL ───────────
function ModelBody({ card }: {
  card: { kind: 'model'; data: ModelPluginView }
}) {
  const { data: m } = card
  return (
    <>
      <p className="text-ark-cyan/55 text-[10px] leading-relaxed">{m.description}</p>
      <p className="text-ark-cyan/40 text-[10px] mt-0.5 font-mono truncate" title={m.defaultBaseUrl}>
        {m.defaultBaseUrl}
      </p>
      <div className="flex gap-1.5 flex-wrap items-center mt-0.5">
        {m.isLocal ? (
          <span className="text-[9px] tracking-widest px-2 py-0.5 rounded"
            style={{
              color: 'rgba(74,222,128,0.9)', background: 'rgba(74,222,128,0.08)',
              border: '1px solid rgba(74,222,128,0.3)',
            }}>LOCAL</span>
        ) : m.requiresApiKey ? (
          <span className="text-[9px] tracking-widest px-2 py-0.5 rounded"
            style={{
              color: 'rgba(0,200,255,0.9)', background: 'rgba(0,200,255,0.08)',
              border: '1px solid rgba(0,200,255,0.3)',
            }}>API KEY</span>
        ) : null}
        {m.defaultModel && (
          <span className="text-[10px] text-ark-cyan/45 font-mono">
            {m.defaultModel}
          </span>
        )}
      </div>
    </>
  )
}
