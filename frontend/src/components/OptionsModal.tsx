import React, { useState, useCallback } from 'react'
import { invoke } from '../services/tauri'
import { useBackupStore, type CloudProvider, type BackupScope } from '../stores/backupStore'
import { useConfigStore } from '../stores/configStore'
import RawConfigViewer from './RawConfigViewer'
import { useI18n } from '../i18n/useI18n'

interface BackupListEntry {
  key: string
  name: string
  size_bytes: number
  created_at: string
}

interface OptionsModalProps {
  onClose: () => void
}

type OptionsTab = 'general' | 'backup' | 'config'

// Scope options — labels resolved via i18n inside component
const SCOPE_VALUES: BackupScope[] = ['map', 'map_players_tribes', 'full']

// Provider colors stay static; labels/descs resolved via i18n inside component
const PROVIDER_COLORS: Record<CloudProvider, string> = {
  none:         '#6b7280',
  s3:           '#f97316',
  gdrive:       '#4ade80',
  onedrive:     '#3b82f6',
  icloud:       '#a78bfa',
  local_folder: '#22d3ee',
}

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
  { code: 'ru', label: 'Russian',            native: 'Русский' },
]

export default function OptionsModal({ onClose }: OptionsModalProps) {
  const [tab, setTab] = useState<OptionsTab>('general')
  const [backupStatus, setBackupStatus] = useState<string | null>(null)
  const [backupError, setBackupError] = useState<string | null>(null)
  const [testStatus, setTestStatus] = useState<string | null>(null)
  const [authStatus, setAuthStatus] = useState<string | null>(null)
  const [isBacking, setIsBacking] = useState(false)
  const [isTesting, setIsTesting] = useState(false)
  const [isAuthing, setIsAuthing] = useState(false)
  // Restore state
  const [cloudList, setCloudList] = useState<BackupListEntry[]>([])
  const [isListing, setIsListing] = useState(false)
  const [listError, setListError] = useState<string | null>(null)
  const [restoring, setRestoring] = useState<string | null>(null)
  const [restoreStatus, setRestoreStatus] = useState<string | null>(null)
  const [restoreError, setRestoreError] = useState<string | null>(null)
  // Metadata for local backups — keyed by backup key (abs path)
  const [backupMeta, setBackupMeta] = useState<Record<string, { mod_ids?: string[]; server_name?: string; note?: string } | null>>({})

  const store = useBackupStore()
  const { config, setConfig, setSavedConfig } = useConfigStore()
  const { tk } = useI18n()

  const providerArgs = useCallback(() => ({
    provider: store.provider,
    s3Endpoint: store.s3Endpoint || null,
    s3Bucket: store.s3Bucket || null,
    s3AccessKey: store.s3AccessKey || null,
    s3SecretKey: store.s3SecretKey || null,
    s3Region: store.s3Region || null,
    accessToken: (store.provider === 'gdrive' ? store.gdriveAccessToken
      : store.provider === 'onedrive' ? store.onedriveAccessToken
      : null) || null,
    icloudPath: store.icloudPath || null,
    localFolderPath: store.localFolderPath || null,
  }), [store])

  // Metadata JSON embedded inside every backup zip (for new-PC restore)
  const buildMetadataJson = useCallback(() => {
    if (!config) return null
    return JSON.stringify({
      server_name: config.identification.session_name,
      mod_ids: config.mods.active_mods,
      scope: store.backupScope,
      backed_up_at: new Date().toISOString(),
    })
  }, [config, store.backupScope])

  // ── List cloud backups ───────────────────────────────────────────────────────
  const handleListBackups = async () => {
    setIsListing(true)
    setListError(null)
    setRestoreStatus(null)
    setRestoreError(null)
    try {
      const entries = await invoke<BackupListEntry[]>('list_cloud_backups', providerArgs())
      setCloudList(entries)
      if (entries.length === 0) setListError(tk('no_backups_found', 'No backups found for this provider.'))

      // For local providers (icloud, local_folder), read metadata from each zip
      if (store.provider === 'local_folder' || store.provider === 'icloud') {
        const metaResults: Record<string, any> = {}
        await Promise.all(entries.map(async (entry) => {
          try {
            const raw = await invoke<string | null>('read_backup_metadata', { zipPath: entry.key })
            metaResults[entry.key] = raw ? JSON.parse(raw) : null
          } catch {
            metaResults[entry.key] = null
          }
        }))
        setBackupMeta(metaResults)
      }
    } catch (e) {
      setListError(`❌ ${String(e)}`)
    } finally {
      setIsListing(false)
    }
  }

  // ── Restore a backup ─────────────────────────────────────────────────────────
  const handleRestore = async (entry: BackupListEntry) => {
    if (!config) return
    const confirmed = window.confirm(
      `Restore "${entry.name}"?\n\n` +
      `Current saves will be renamed to SavedArks_preRestore_* as a safety snapshot.\n\n` +
      `Make sure the server is NOT running.`
    )
    if (!confirmed) return

    setRestoring(entry.key)
    setRestoreStatus(null)
    setRestoreError(null)
    try {
      const msg = await invoke<string>('restore_backup_from_cloud', {
        serverDir: config.paths.server_dir,
        backupKey: entry.key,
        backupName: entry.name,
        ...providerArgs(),
      })
      setRestoreStatus(`✅ ${msg}`)
    } catch (e) {
      setRestoreError(`❌ ${String(e)}`)
    } finally {
      setRestoring(null)
    }
  }

  // ── Backup now ───────────────────────────────────────────────────────────────
  const handleBackupNow = async () => {
    if (!config) return
    setIsBacking(true)
    setBackupStatus(null)
    setBackupError(null)
    try {
      const token = store.provider === 'gdrive' ? store.gdriveAccessToken
        : store.provider === 'onedrive' ? store.onedriveAccessToken
        : undefined

      // Include config.toml so the backup is self-contained and importable
      let configToml: string | null = null
      try { configToml = await invoke<string>('config_to_toml', { config }) } catch { /* non-fatal */ }

      const filename = await invoke<string>('backup_saves', {
        serverDir: config.paths.server_dir,
        map: config.cluster_maps?.[0] || 'TheIsland_WP',
        scope: store.backupScope,
        provider: store.provider,
        s3Endpoint: store.s3Endpoint || null,
        s3Bucket: store.s3Bucket || null,
        s3AccessKey: store.s3AccessKey || null,
        s3SecretKey: store.s3SecretKey || null,
        s3Region: store.s3Region || null,
        accessToken: token || null,
        icloudPath: store.icloudPath || null,
        localFolderPath: store.localFolderPath || null,
        metadataJson: buildMetadataJson(),
        configToml,
      })

      store.addBackupEntry({
        filename,
        size_bytes: 0,
        created_at: new Date().toISOString(),
        provider: store.provider,
      })
      setBackupStatus(`✅ ${tk('backup_complete', 'Backup complete')}: ${filename}`)
    } catch (e) {
      setBackupError(`❌ ${String(e)}`)
    } finally {
      setIsBacking(false)
    }
  }

  // ── Test S3 connection ───────────────────────────────────────────────────────
  const handleTestS3 = async () => {
    setIsTesting(true)
    setTestStatus(null)
    try {
      const result = await invoke<string>('test_s3_connection', {
        endpoint: store.s3Endpoint,
        bucket: store.s3Bucket,
        accessKey: store.s3AccessKey,
        secretKey: store.s3SecretKey,
        region: store.s3Region,
      })
      setTestStatus(`✅ ${result}`)
    } catch (e) {
      setTestStatus(`❌ ${String(e)}`)
    } finally {
      setIsTesting(false)
    }
  }

  // ── OAuth flows ──────────────────────────────────────────────────────────────
  const handleGDriveAuth = async () => {
    setIsAuthing(true)
    setAuthStatus(null)
    try {
      const tokens = await invoke<{ access_token: string; refresh_token: string }>('start_gdrive_oauth', {
        clientId: store.gdriveClientId,
        clientSecret: store.gdriveClientSecret,
      })
      store.setGDriveField('gdriveAccessToken', tokens.access_token)
      store.setGDriveField('gdriveRefreshToken', tokens.refresh_token)
      setAuthStatus(`✅ ${tk('gdrive_authorized', 'Google Drive authorized successfully')}`)
    } catch (e) {
      setAuthStatus(`❌ ${String(e)}`)
    } finally {
      setIsAuthing(false)
    }
  }

  const handleOneDriveAuth = async () => {
    setIsAuthing(true)
    setAuthStatus(null)
    try {
      const tokens = await invoke<{ access_token: string; refresh_token: string }>('start_onedrive_oauth', {
        clientId: store.onedriveClientId,
      })
      store.setOneDriveField('onedriveAccessToken', tokens.access_token)
      store.setOneDriveField('onedriveRefreshToken', tokens.refresh_token)
      setAuthStatus(`✅ ${tk('onedrive_authorized', 'OneDrive authorized successfully')}`)
    } catch (e) {
      setAuthStatus(`❌ ${String(e)}`)
    } finally {
      setIsAuthing(false)
    }
  }

  const isConfigured = () => {
    switch (store.provider) {
      case 's3':           return !!(store.s3Endpoint && store.s3Bucket && store.s3AccessKey && store.s3SecretKey)
      case 'gdrive':       return !!store.gdriveAccessToken
      case 'onedrive':     return !!store.onedriveAccessToken
      case 'icloud':       return !!store.icloudPath
      case 'local_folder': return !!store.localFolderPath
      default:             return false
    }
  }

  // icloud is synced silently by the OS; local_folder and cloud providers support explicit list/restore
  const isCloudProvider = !['none', 'icloud'].includes(store.provider)

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: 'rgba(0,0,0,0.75)', backdropFilter: 'blur(4px)' }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        className={`ark-panel rounded-xl w-full max-h-[90vh] flex flex-col transition-all ${tab === 'config' ? 'max-w-4xl' : 'max-w-2xl'}`}
        style={{ border: '1px solid rgba(0,200,255,0.3)', boxShadow: '0 0 40px rgba(0,200,255,0.1)' }}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 pt-5 pb-4 border-b border-ark-cyan/20 flex-shrink-0">
          <span className="text-ark-cyan font-bold tracking-widest text-sm uppercase">{tk('options_title', '⚙ Options')}</span>
          <button
            onClick={onClose}
            className="text-ark-cyan/40 hover:text-ark-cyan/80 text-xs tracking-widest transition-colors"
          >
            {tk('esc_close', 'ESC / CLOSE')}
          </button>
        </div>

        {/* Tab bar */}
        <div className="flex border-b border-ark-cyan/20 flex-shrink-0">
          {(['general', 'backup', 'config'] as OptionsTab[]).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className="px-6 py-3 text-xs font-bold tracking-widest uppercase transition-colors"
              style={{
                color: tab === t ? 'rgba(0,200,255,0.9)' : 'rgba(0,200,255,0.35)',
                borderBottom: tab === t ? '2px solid rgba(0,200,255,0.8)' : '2px solid transparent',
              }}
            >
              {t === 'general' ? tk('tab_general', 'General') : t === 'backup' ? tk('tab_backup', 'Backup') : tk('tab_config_ini', 'Config INI')}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-5">

          {/* ── GENERAL TAB ── */}
          {tab === 'general' && (
            <>
              {/* Language selector */}
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

              {/* On-demand servers */}
              <Section title={tk('section_on_demand', 'On-Demand Server')}>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">{tk('on_demand_title', 'Enable sleep mode')}</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('on_demand_desc', 'Lets individual maps appear in the ARK browser without the server running.')}</p>
                  </div>
                  <Toggle value={store.onDemandEnabled} onChange={store.setOnDemandEnabled} />
                </div>
              </Section>

              {/* Cluster start delay — only relevant in always-on (non-on-demand) mode */}
              {!store.onDemandEnabled && (
                <Section title={tk('section_cluster', 'Server Cluster')}>
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
                </Section>
              )}

              {/* Minimize to tray */}
              <Section title={tk('section_close_behavior', 'Close Behavior')}>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">{tk('minimize_tray_title', 'Minimize to system tray')}</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('minimize_tray_desc', 'When closing the window, the app minimizes to the system tray.')}</p>
                  </div>
                  <Toggle value={store.minimizeToTray} onChange={store.setMinimizeToTray} />
                </div>
              </Section>

              {/* Manual save */}
              <Section title={tk('section_save', 'Save Settings')}>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">{tk('manual_save_title', 'Manual save')}</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('manual_save_desc', 'By default changes are saved automatically. Enable this to save only when you press SAVE SETTINGS.')}</p>
                  </div>
                  <Toggle value={store.manualSave} onChange={store.setManualSave} />
                </div>
              </Section>

              {/* Logs toggle */}
              <Section title={tk('section_logs', 'Log Viewer')}>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">{tk('logs_btn_title', 'Server logs button')}</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('logs_btn_desc', 'Shows the LOGS button in the bottom bar to view ShooterGame.log in real time')}</p>
                  </div>
                  <Toggle value={store.logsEnabled} onChange={store.setLogsEnabled} />
                </div>
              </Section>

              {/* Backup scope */}
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

              {/* Max saves */}
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
          )}

          {/* ── BACKUP TAB ── */}
          {tab === 'backup' && (
            <>
              {/* Provider selector */}
              <Section title={tk('section_backup_dest', 'Backup Destination')}>
                <div className="grid grid-cols-2 gap-2">
                  {(Object.keys(PROVIDER_COLORS) as CloudProvider[]).map((pVal) => {
                    const color = PROVIDER_COLORS[pVal]
                    const label = pVal === 'none' ? tk('provider_none', 'No provider')
                               : pVal === 'local_folder' ? tk('provider_local_folder', 'Local folder')
                               : pVal === 'gdrive' ? 'Google Drive'
                               : pVal === 'onedrive' ? 'OneDrive'
                               : pVal === 'icloud' ? 'iCloud'
                               : 'S3-compatible'
                    const desc = pVal === 's3' ? 'AWS S3, Backblaze B2, Wasabi, R2…'
                               : pVal === 'gdrive' ? 'OAuth 2.0 — requires GCP app'
                               : pVal === 'onedrive' ? 'OAuth 2.0 — requires Azure app'
                               : pVal === 'icloud' ? 'iCloud for Windows local folder'
                               : pVal === 'local_folder' ? 'No account — sync with local OneDrive/GDrive'
                               : ''
                    return (
                      <button
                        key={pVal}
                        onClick={() => store.setProvider(pVal)}
                        className="rounded-md p-3 text-left transition-all"
                        style={{
                          background: store.provider === pVal ? `${color}18` : 'rgba(255,255,255,0.03)',
                          border: `1px solid ${store.provider === pVal ? `${color}60` : 'rgba(255,255,255,0.08)'}`,
                          boxShadow: store.provider === pVal ? `0 0 12px ${color}20` : 'none',
                        }}
                      >
                        <p className="text-sm font-semibold" style={{ color: store.provider === pVal ? color : 'rgba(255,255,255,0.45)' }}>
                          {label}
                        </p>
                        {desc && (
                          <p className="text-[10px] mt-0.5" style={{ color: store.provider === pVal ? `${color}99` : 'rgba(255,255,255,0.2)' }}>
                            {desc}
                          </p>
                        )}
                      </button>
                    )
                  })}
                </div>
              </Section>

              {/* Local folder config */}
              {store.provider === 'local_folder' && (
                <Section title={tk('section_local_folder', 'Local Folder')}>
                  <p className="text-ark-cyan/40 text-xs mb-3">
                    Copia el backup como .zip a esta carpeta. Ponla dentro de tu carpeta sincronizada de
                    <span className="text-ark-cyan/60"> OneDrive</span>,
                    <span className="text-ark-cyan/60"> Google Drive</span> o
                    <span className="text-ark-cyan/60"> Dropbox</span> para tener respaldo automático en la nube
                    sin configurar OAuth. Ejemplo: <span className="font-mono text-ark-cyan/60">C:\Users\Max\OneDrive\ARKBackups</span>
                  </p>
                  <Field
                    label={tk('destination_path', 'Destination path')}
                    value={store.localFolderPath}
                    onChange={store.setLocalFolderPath}
                    placeholder="C:\Users\Max\OneDrive\ARKBackups"
                  />
                </Section>
              )}

              {/* S3 config */}
              {store.provider === 's3' && (
                <Section title={tk('section_s3_config', 'S3-compatible Configuration')}>
                  <p className="text-ark-cyan/40 text-xs mb-3">
                    Compatible con AWS S3, Contabo Object Storage, Seenode, Cloudflare R2, Backblaze B2, Wasabi, MinIO, etc.
                  </p>
                  <div className="space-y-2">
                    <Field label="Endpoint URL" value={store.s3Endpoint} onChange={(v) => store.setS3Field('s3Endpoint', v)} placeholder="https://eu2.contabostorage.com" />
                    <Field label="Bucket" value={store.s3Bucket} onChange={(v) => store.setS3Field('s3Bucket', v)} placeholder="ark-backups" />
                    <Field label="Access Key" value={store.s3AccessKey} onChange={(v) => store.setS3Field('s3AccessKey', v)} placeholder="AKIAIOSFODNN7EXAMPLE" />
                    <Field label="Secret Key" value={store.s3SecretKey} onChange={(v) => store.setS3Field('s3SecretKey', v)} placeholder="wJalrXUtnFEMI/K7MDENG/..." type="password" />
                    <Field label={tk('region', 'Region')} value={store.s3Region} onChange={(v) => store.setS3Field('s3Region', v)} placeholder="us-east-1" />
                  </div>
                  <div className="mt-3 flex items-center gap-3">
                    <button
                      onClick={handleTestS3}
                      disabled={isTesting}
                      className="ark-action-btn text-xs px-4 py-1.5"
                    >
                      {isTesting ? tk('testing', '⏳ Testing...') : tk('test_connection', '🔌 Test connection')}
                    </button>
                    {testStatus && (
                      <span className={`text-xs ${testStatus.startsWith('✅') ? 'text-green-400' : 'text-red-400'}`}>
                        {testStatus}
                      </span>
                    )}
                  </div>
                </Section>
              )}

              {/* Google Drive config */}
              {store.provider === 'gdrive' && (
                <Section title="Google Drive">
                  <p className="text-ark-cyan/40 text-xs mb-3">
                    Crea una app en <span className="text-ark-cyan/60">console.cloud.google.com</span> → OAuth 2.0 → Desktop App. Agrega tu cuenta como usuario de prueba. Copia el Client ID y Client Secret aquí.
                  </p>
                  <div className="space-y-2">
                    <Field label="Client ID" value={store.gdriveClientId} onChange={(v) => store.setGDriveField('gdriveClientId', v)} placeholder="12345.apps.googleusercontent.com" />
                    <Field label="Client Secret" value={store.gdriveClientSecret} onChange={(v) => store.setGDriveField('gdriveClientSecret', v)} placeholder="GOCSPX-..." type="password" />
                  </div>
                  {store.gdriveAccessToken && (
                    <p className="text-green-400/70 text-xs mt-2">{tk('authorized', '✅ Authorized — token active')}</p>
                  )}
                  <div className="mt-3 flex items-center gap-3">
                    <button
                      onClick={handleGDriveAuth}
                      disabled={isAuthing || !store.gdriveClientId || !store.gdriveClientSecret}
                      className="ark-action-btn text-xs px-4 py-1.5"
                    >
                      {isAuthing ? tk('waiting_auth', '⏳ Waiting for authorization...') : store.gdriveAccessToken ? tk('reauth', '🔄 Re-authorize') : tk('auth_google', '🔑 Authorize with Google')}
                    </button>
                  </div>
                  {authStatus && (
                    <p className={`text-xs mt-2 ${authStatus.startsWith('✅') ? 'text-green-400' : 'text-red-400'}`}>
                      {authStatus}
                    </p>
                  )}
                </Section>
              )}

              {/* OneDrive config */}
              {store.provider === 'onedrive' && (
                <Section title="OneDrive (Microsoft)">
                  <p className="text-ark-cyan/40 text-xs mb-3">
                    Crea una app en <span className="text-ark-cyan/60">portal.azure.com</span> → Registros de App → Nueva. Tipo de cuenta: "Cualquier org + cuentas personales". Agrega redirect URI: <span className="text-ark-cyan/60 font-mono">http://localhost</span>. No necesitas client secret.
                  </p>
                  <div className="space-y-2">
                    <Field label="Client ID (Application ID)" value={store.onedriveClientId} onChange={(v) => store.setOneDriveField('onedriveClientId', v)} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" />
                  </div>
                  {store.onedriveAccessToken && (
                    <p className="text-green-400/70 text-xs mt-2">{tk('authorized', '✅ Authorized — token active')}</p>
                  )}
                  <div className="mt-3 flex items-center gap-3">
                    <button
                      onClick={handleOneDriveAuth}
                      disabled={isAuthing || !store.onedriveClientId}
                      className="ark-action-btn text-xs px-4 py-1.5"
                    >
                      {isAuthing ? tk('waiting_auth', '⏳ Waiting for authorization...') : store.onedriveAccessToken ? tk('reauth', '🔄 Re-authorize') : tk('auth_microsoft', '🔑 Authorize with Microsoft')}
                    </button>
                  </div>
                  {authStatus && (
                    <p className={`text-xs mt-2 ${authStatus.startsWith('✅') ? 'text-green-400' : 'text-red-400'}`}>
                      {authStatus}
                    </p>
                  )}
                </Section>
              )}

              {/* iCloud config */}
              {store.provider === 'icloud' && (
                <Section title="iCloud Drive">
                  <Field label={tk('icloud_path', 'iCloud Drive path')} value={store.icloudPath} onChange={store.setICloudPath} placeholder="C:\Users\Max\iCloudDrive" />
                </Section>
              )}

              {/* Backup now + history */}
              {store.provider !== 'none' && (
                <>
                  <div className="flex items-center gap-3 pt-2">
                    <button
                      onClick={handleBackupNow}
                      disabled={isBacking || !isConfigured()}
                      className="ark-action-btn px-5 py-2 text-xs font-bold tracking-widest"
                      style={{ opacity: isConfigured() ? 1 : 0.4 }}
                    >
                      {isBacking ? tk('backing_up', '⏳ Creating backup...') : tk('backup_now_btn', '💾 BACKUP NOW')}
                    </button>
                    {!isConfigured() && (
                      <span className="text-ark-cyan/40 text-xs">{tk('complete_provider_config', 'Complete the provider configuration')}</span>
                    )}
                  </div>

                  {backupStatus && <p className="text-green-400/80 text-xs">{backupStatus}</p>}
                  {backupError && <p className="text-red-400/80 text-xs">{backupError}</p>}

                  {store.backupHistory.length > 0 && (
                    <Section title={tk('section_backup_history', 'Backup History')}>
                      <div className="space-y-1.5 max-h-40 overflow-y-auto">
                        {store.backupHistory.map((b, i) => (
                          <div
                            key={i}
                            className="flex items-center justify-between px-3 py-2 rounded"
                            style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)' }}
                          >
                            <span className="text-ark-cyan/60 text-xs font-mono truncate max-w-xs">{b.filename}</span>
                            <span className="text-ark-cyan/30 text-[10px] ml-2 flex-shrink-0">
                              {new Date(b.created_at).toLocaleString()}
                            </span>
                          </div>
                        ))}
                      </div>
                    </Section>
                  )}

                  {/* Restore — only for remote cloud providers */}
                  {isCloudProvider && (
                    <Section title={tk('section_restore_cloud', 'Restore from Cloud')}>
                      <div className="flex items-center gap-3 pt-1">
                        <button
                          onClick={handleListBackups}
                          disabled={isListing || !isConfigured()}
                          className="ark-action-btn text-xs px-4 py-1.5"
                          style={{ opacity: isConfigured() ? 1 : 0.4 }}
                        >
                          {isListing ? tk('searching', '⏳ Searching...') : tk('list_backups_btn', '🔍 List available backups')}
                        </button>
                      </div>

                      {listError && <p className="text-red-400/80 text-xs">{listError}</p>}

                      {cloudList.length > 0 && (
                        <div className="space-y-1.5 max-h-52 overflow-y-auto mt-2">
                          {cloudList.map((entry) => {
                            const isRestoring = restoring === entry.key
                            const sizeMB = (entry.size_bytes / 1024 / 1024).toFixed(1)
                            const date = new Date(entry.created_at).toLocaleString()
                            return (
                              <div
                                key={entry.key}
                                className="flex items-center justify-between px-3 py-2 rounded gap-2"
                                style={{
                                  background: isRestoring ? 'rgba(0,200,255,0.06)' : 'rgba(255,255,255,0.03)',
                                  border: `1px solid ${isRestoring ? 'rgba(0,200,255,0.35)' : 'rgba(255,255,255,0.07)'}`,
                                }}
                              >
                                <div className="min-w-0 flex-1">
                                  <p className="text-ark-cyan/80 text-xs font-mono truncate">{entry.name}</p>
                                  <p className="text-ark-cyan/30 text-[10px]">{date} · {sizeMB} MB</p>
                                  {backupMeta[entry.key]?.mod_ids && backupMeta[entry.key]!.mod_ids!.length > 0 && (
                                    <p className="text-ark-cyan/40 text-[10px] mt-0.5 truncate">
                                      Mods: {backupMeta[entry.key]!.mod_ids!.slice(0, 5).join(', ')}
                                      {backupMeta[entry.key]!.mod_ids!.length > 5 && ` +${backupMeta[entry.key]!.mod_ids!.length - 5} más`}
                                    </p>
                                  )}
                                  {backupMeta[entry.key]?.note && (
                                    <p className="text-yellow-400/50 text-[10px] mt-0.5 truncate">
                                      ℹ {backupMeta[entry.key]!.note}
                                    </p>
                                  )}
                                </div>
                                <button
                                  onClick={() => handleRestore(entry)}
                                  disabled={!!restoring}
                                  className="flex-shrink-0 ark-action-btn text-[10px] px-3 py-1"
                                  style={{ opacity: restoring ? 0.4 : 1 }}
                                >
                                  {isRestoring ? tk('restoring', '⏳') : tk('restore_btn', '⏪ Restore')}
                                </button>
                              </div>
                            )
                          })}
                        </div>
                      )}

                      {restoreStatus && <p className="text-green-400/80 text-xs">{restoreStatus}</p>}
                      {restoreError && <p className="text-red-400/80 text-xs">{restoreError}</p>}
                    </Section>
                  )}
                </>
              )}
            </>
          )}
          {/* ── CONFIG TAB ── */}
          {tab === 'config' && config && (
            <RawConfigViewer
              config={config}
              onConfigSaved={(updated) => {
                setConfig(updated)
                setSavedConfig(updated)
              }}
            />
          )}
          {tab === 'config' && !config && (
            <p className="text-ark-cyan/40 text-sm text-center py-8">
              {tk('config_not_loaded', 'Load the server configuration first.')}
            </p>
          )}

        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-ark-cyan/20 flex justify-end flex-shrink-0">
          <button onClick={onClose} className="ark-action-btn px-6 py-2 text-xs tracking-widest">
            {tk('close', 'CLOSE')}
          </button>
        </div>
      </div>
    </div>
  )
}

// ─── Small helpers ────────────────────────────────────────────────────────────

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <p className="text-ark-cyan/60 text-[10px] font-bold tracking-widest uppercase">{title}</p>
      {children}
    </div>
  )
}

function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!value)}
      className="relative w-10 h-5 rounded-full transition-colors flex-shrink-0"
      style={{ background: value ? 'rgba(0,200,255,0.7)' : 'rgba(255,255,255,0.1)' }}
    >
      <span
        className="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform"
        style={{ left: value ? '1.3rem' : '0.125rem' }}
      />
    </button>
  )
}

function Field({
  label, value, onChange, placeholder, type = 'text'
}: {
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  type?: string
}) {
  return (
    <div className="flex items-center gap-3">
      <label className="text-ark-cyan/50 text-xs w-32 flex-shrink-0 text-right">{label}</label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="flex-1 bg-transparent border border-ark-cyan/25 text-ark-cyan/90 text-xs px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/20 font-mono"
      />
    </div>
  )
}
