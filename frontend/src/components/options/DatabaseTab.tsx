import { useState, useEffect } from 'react'
import { useI18n } from '../../i18n/useI18n'
import { Section, Field, Select } from '../ui/OptionsUI'
import { invoke } from '../../services/tauri'

interface DbBackendView {
  key: string
  label: string
}

interface DbConfig {
  backend: string
  url: string
  api_key: string
  schema: string
  table: string
}

const DEFAULT_CONFIG: DbConfig = {
  backend: 'sqlite',
  url: 'ark-config.db',
  api_key: '',
  schema: 'public',
  table: 'command_log',
}

export default function DatabaseTab() {
  const { tk } = useI18n()
  const [backends, setBackends] = useState<DbBackendView[]>([])
  const [cfg, setCfg] = useState<DbConfig>(DEFAULT_CONFIG)
  const [status, setStatus] = useState<'idle' | 'ok' | 'fail'>('idle')
  const [error, setError] = useState('')
  const [validating, setValidating] = useState(false)
  const [deploying, setDeploying] = useState(false)
  const [deployStatus, setDeployStatus] = useState<'idle' | 'ok' | 'fail'>('idle')
  const [deployError, setDeployError] = useState('')
  const [deployLog, setDeployLog] = useState('')

  useEffect(() => {
    invoke<DbBackendView[]>('list_database_backends')
      .then(setBackends)
      .catch(() => setBackends([]))
  }, [])

  const validate = async () => {
    setValidating(true)
    setStatus('idle')
    setError('')
    try {
      await invoke<void>('validate_database_config', {
        cfg: {
          backend: cfg.backend,
          url: cfg.url,
          api_key: cfg.api_key,
          schema: cfg.schema,
          table: cfg.table,
        },
      })
      setStatus('ok')
    } catch (e: unknown) {
      setStatus('fail')
      setError(String(e))
    } finally {
      setValidating(false)
    }
  }

  const handleConvexDeploy = async () => {
    setDeploying(true)
    setDeployStatus('idle')
    setDeployError('')
    setDeployLog('')
    try {
      const out = await invoke<string>('convex_deploy', {
        deploymentUrl: cfg.url,
        deployKey: cfg.api_key,
      })
      setDeployStatus('ok')
      setDeployLog(out)
    } catch (e: unknown) {
      setDeployStatus('fail')
      const errText = String(e)
      setDeployError(errText)
      setDeployLog(errText)
    } finally {
      setDeploying(false)
    }
  }

  const backendOptions = backends.map((b) => ({ value: b.key, label: b.label }))

  return (
    <>
      <Section title={tk('section_db_backend', 'Audit-log database')}>
        <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
          {tk('db_intro',
            'Every command issued by any chat bot is appended here. Default is local SQLite (no network). Choose Convex / Supabase / InsForge / PostgreSQL / MongoDB for multi-host setups.')}
        </p>
        <Select
          label={tk('db_backend_label', 'Backend')}
          value={cfg.backend}
          onChange={(v) => {
            setCfg({ ...cfg, backend: v })
            setStatus('idle')
          }}
          options={backendOptions}
          placeholder="—"
        />
        <Field
          label={cfg.backend === 'sqlite' ? 'File path' : 'URL'}
          value={cfg.url}
          onChange={(v) => {
            setCfg({ ...cfg, url: v })
            setStatus('idle')
          }}
          placeholder={cfg.backend.startsWith('ibm') ? 'postgres://…/…'
            : cfg.backend === 'sqlite'   ? 'ark-config.db'
            : 'https://xyz.supabase.co'}
        />
        {cfg.backend !== 'sqlite' && (
          <Field
            label={tk('db_key_label', 'API key / token')}
            value={cfg.api_key}
            onChange={(v) => {
              setCfg({ ...cfg, api_key: v })
              setStatus('idle')
            }}
            placeholder="paste from BaaS dashboard"
            type="password"
          />
        )}
        {cfg.backend !== 'sqlite' && (
          <Field
            label={tk('db_schema_label', 'Schema')}
            value={cfg.schema}
            onChange={(v) => setCfg({ ...cfg, schema: v })}
          />
        )}
        <Field
          label={tk('db_table_label', 'Table / collection')}
          value={cfg.table}
          onChange={(v) => setCfg({ ...cfg, table: v })}
        />
        <div className="flex items-center gap-3">
          <button
            onClick={validate}
            disabled={validating || !cfg.url}
            className="ark-action-btn px-4 py-2 text-xs tracking-widest disabled:opacity-40"
          >
            {validating ? tk('validating', 'Validating…') : tk('btn_validate', 'VALIDATE')}
          </button>
          {status === 'ok' && (
            <span className="text-emerald-400 text-xs tracking-widest">✓ {tk('db_valid', 'Config valid')}</span>
          )}
          {status === 'fail' && (
            <span className="text-red-400 text-xs">⚠ {error}</span>
          )}
        </div>
      </Section>

      <Section title={tk('section_db_notes', 'Behaviour')}>
        <ul className="text-ark-cyan/45 text-[11px] space-y-1.5 list-disc pl-5">
          <li>{tk('db_note_1', 'SQLite is the default and persists in APPDATA/ARK ASA Config Manager.')}</li>
          <li>{tk('db_note_2', 'Supabase & InsForge both expose RLS-friendly PostgREST; reuse the same URL for the Web Admin.')}</li>
          <li>{tk('db_note_3', 'Postgres backend accepts libpq or PostgREST URLs.')}</li>
          <li>{tk('db_note_4', 'MongoDB uses Atlas Data API — drop the raw connection string, use the API base URL.')}</li>
        </ul>
      </Section>

      {cfg.backend === 'convex' && (
        <Section title={tk('convex_one_click_title', 'Convex One-Click Deploy')}>
          <div className="space-y-4">
            <p className="text-ark-cyan/40 text-[10px] leading-relaxed">
              {tk('convex_one_click_desc',
                'Push the schema and database functions directly to your Convex deployment. Make sure you entered your Convex URL and Deploy Key above, then click DEPLOY.')}
            </p>
            <div className="flex items-center gap-3 flex-wrap">
              <button
                onClick={handleConvexDeploy}
                disabled={deploying || !cfg.url || !cfg.api_key}
                className="ark-action-btn px-4 py-2 text-xs tracking-widest disabled:opacity-40"
                style={{ borderColor: 'rgba(0, 200, 255, 0.4)' }}
              >
                {deploying ? tk('deploying', 'Deploying…') : tk('btn_deploy', 'DEPLOY TO CONVEX')}
              </button>
              {deployStatus === 'ok' && (
                <span className="text-emerald-400 text-xs tracking-widest">{tk('deploy_success', 'Deployed successfully')}</span>
              )}
              {deployStatus === 'fail' && (
                <span className="text-red-400 text-xs">⚠ {deployError}</span>
              )}
            </div>
            {deployLog && (
              <pre className="text-[10px] bg-black/40 text-ark-cyan/60 p-3 rounded font-mono max-h-40 overflow-y-auto whitespace-pre-wrap leading-tight border border-ark-cyan/5">
                {deployLog}
              </pre>
            )}
          </div>
        </Section>
      )}
    </>
  )
}
