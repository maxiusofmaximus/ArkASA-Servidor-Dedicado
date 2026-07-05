import { useState, useEffect, useCallback } from 'react'
import { useI18n } from '../../i18n/useI18n'
import { Section } from '../ui/OptionsUI'
import { invoke } from '../../services/tauri'

interface PluginCatalogView {
  id: string
  label: string
  channel: string
  capabilities: string[]
  required_secrets: string[]
  oauth_url: string | null
  enabled: boolean
  has_required_secrets: boolean
}

interface ConnectionPluginView {
  id: string
  label: string
  description: string
  free_tier: boolean
  requires_cli: string[]
  requires_credentials: boolean
  docs_url: string
}

interface ModelPluginView {
  id: string
  label: string
  description: string
  defaultBaseUrl: string
  defaultModel: string
  requiresApiKey: boolean
  isLocal: boolean
  installHint: string
  docsUrl: string
}

export default function PluginsTab() {
  const { tk } = useI18n()

  // 1. Chat plugin catalog (toggleable runtime: convex / vercel)
  const [plugins, setPlugins] = useState<PluginCatalogView[]>([])
  const [pluginBusy, setPluginBusy] = useState<string | null>(null)

  // 2. Connection catalog (read-only declarative metadata)
  const [conns, setConns] = useState<ConnectionPluginView[]>([])

  // 3. AI model plugins
  const [models, setModels] = useState<ModelPluginView[]>([])

  const refreshPlugins = useCallback(async () => {
    try { setPlugins(await invoke<PluginCatalogView[]>('list_plugin_catalog')) } catch { setPlugins([]) }
  }, [])

  const refreshConns = useCallback(async () => {
    try { setConns(await invoke<ConnectionPluginView[]>('list_connection_plugins')) } catch { setConns([]) }
  }, [])

  const refreshModels = useCallback(async () => {
    try { setModels(await invoke<ModelPluginView[]>('list_model_plugins')) } catch { setModels([]) }
  }, [])

  useEffect(() => { void refreshPlugins() }, [refreshPlugins])
  useEffect(() => { void refreshConns() }, [refreshConns])
  useEffect(() => { void refreshModels() }, [refreshModels])

  const togglePlugin = async (id: string, currentlyOn: boolean) => {
    setPluginBusy(id)
    try {
      if (currentlyOn) {
        await invoke('disable_plugin', { id })
      } else {
        await invoke('enable_plugin', { id })
      }
      await refreshPlugins()
    } catch (e) {
      console.warn('toggle failed', e)
    } finally {
      setPluginBusy(null)
    }
  }

  return (
    <>
      {/* ── Chat / backend plugins ────────────────────────────────── */}
      <Section title={tk('section_plugin_hub', 'Plugin Hub')}>
        <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
          {tk('plugin_hub_intro',
            'Toggle built-in backend plugins (Convex, Vercel). Changes apply without restarting the app.')}
        </p>
        {plugins.length === 0 && (
          <p className="text-ark-cyan/30 text-xs italic mt-2">{tk('loading', 'Loading…')}</p>
        )}
        <ul className="space-y-2 mt-2">
          {plugins.map((p) => (
            <li
              key={p.id}
              className="flex items-center justify-between rounded-md px-3 py-2 transition-colors"
              style={{
                background: p.enabled ? 'rgba(74,222,128,0.05)' : 'rgba(255,255,255,0.02)',
                border: p.enabled ? '1px solid rgba(74,222,128,0.25)' : '1px solid rgba(255,255,255,0.06)',
              }}
            >
              <div className="flex-1 min-w-0 pr-3">
                <p className="font-semibold text-ark-cyan/90 text-sm">{p.label}</p>
                <p className="text-ark-cyan/40 text-[10px] font-mono">{p.id}</p>
                {p.required_secrets.length > 0 && !p.has_required_secrets && (
                  <p className="text-amber-400 text-[10px] mt-1">
                    ⚠ {tk('plugin_missing_secrets', 'Missing required secrets: ')}
                    {p.required_secrets.join(', ')}
                  </p>
                )}
              </div>
              <button
                disabled={pluginBusy === p.id}
                onClick={() => togglePlugin(p.id, p.enabled)}
                className="ark-action-btn px-3 py-1 text-[10px] tracking-widest disabled:opacity-40"
                style={p.enabled ? { borderColor: 'rgba(74,222,128,0.4)' } : {}}
              >
                {p.enabled ? tk('plugin_enabled', '● ON')
                            : tk('plugin_disabled', '○ OFF')}
              </button>
            </li>
          ))}
        </ul>
      </Section>

      {/* ── Connection providers ─────────────────────────────────────── */}
      <Section title={tk('section_connection_providers', 'Connection providers')}>
        <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
          {tk('connection_providers_intro',
            'VPS providers the desktop app recognises. The operator runs the official CLI (hcloud/doctl/aws/az/gcloud/oci); the app just builds the runner scripts.')}
        </p>
        <ul className="space-y-2 mt-2">
          {conns.map((c) => (
            <li key={c.id} className="rounded-md px-3 py-2 border border-ark-cyan/15">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-semibold text-ark-cyan/90 text-sm">{c.label}</p>
                  <p className="text-ark-cyan/40 text-[10px] font-mono">{c.id}</p>
                </div>
                {c.free_tier && (
                  <span className="text-[9px] tracking-widest px-2 py-0.5 rounded"
                    style={{ color: 'rgba(74,222,128,0.9)', background: 'rgba(74,222,128,0.08)',
                             border: '1px solid rgba(74,222,128,0.3)' }}>
                    FREE TIER
                  </span>
                )}
              </div>
              <p className="text-ark-cyan/55 text-[10px] mt-1">{c.description}</p>
              <div className="text-ark-cyan/45 text-[10px] mt-1">
                {c.requires_cli.length > 0 && (
                  <span>CLI: <code className="font-mono text-ark-accent">{c.requires_cli.join(', ')}</code></span>
                )}
                {c.requires_cli.length > 0 && c.requires_credentials && <span> · </span>}
                {c.requires_credentials && <span>Needs credentials</span>}
                {(c.requires_cli.length === 0 && !c.requires_credentials) && (
                  <span className="text-emerald-400">No CLI / no credentials needed.</span>
                )}
              </div>
            </li>
          ))}
        </ul>
      </Section>

      {/* ── AI model plugins ─────────────────────────────────────────── */}
      <Section title={tk('section_ai_models', 'AI model plugins')}>
        <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
          {tk('ai_models_intro',
            'OpenAI-API-compatible adapters the desktop recognises. Pick one and the app fills AI_API_URL/AI_MODEL defaults. Locals run on your workstation without an API key.')}
        </p>
        <ul className="space-y-2 mt-2">
          {models.map((m) => (
            <li key={m.id} className="rounded-md px-3 py-2 border border-ark-cyan/15">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-semibold text-ark-cyan/90 text-sm">{m.label}</p>
                  <p className="text-ark-cyan/40 text-[10px] font-mono">{m.id}</p>
                </div>
                {m.isLocal
                  ? (<span className="text-[9px] tracking-widest px-2 py-0.5 rounded"
                       style={{ color: 'rgba(74,222,128,0.9)', background: 'rgba(74,222,128,0.08)',
                                border: '1px solid rgba(74,222,128,0.3)' }}>LOCAL</span>)
                  : m.requiresApiKey
                    ? (<span className="text-[9px] tracking-widest px-2 py-0.5 rounded"
                         style={{ color: 'rgba(0,200,255,0.9)', background: 'rgba(0,200,255,0.08)',
                                  border: '1px solid rgba(0,200,255,0.3)' }}>API KEY</span>)
                    : null
                }
              </div>
              <p className="text-ark-cyan/55 text-[10px] mt-1">{m.description}</p>
              <p className="text-ark-cyan/40 text-[10px] mt-1 font-mono">
                {m.defaultBaseUrl}<br />
                default model: <span className="text-ark-accent">{m.defaultModel}</span>
              </p>
              <p className="text-ark-cyan/40 text-[10px] mt-1 italic">
                {m.installHint}
              </p>
            </li>
          ))}
        </ul>
      </Section>
    </>
  )
}
