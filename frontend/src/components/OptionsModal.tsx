import React, { useState, useCallback } from 'react'
import { invoke } from '../services/tauri'
import { useBackupStore, type CloudProvider, type BackupScope } from '../stores/backupStore'
import { useConfigStore } from '../stores/configStore'

interface BackupListEntry {
  key: string
  name: string
  size_bytes: number
  created_at: string
}

interface OptionsModalProps {
  onClose: () => void
}

type OptionsTab = 'general' | 'backup'

const SCOPE_OPTIONS: { value: BackupScope; label: string; desc: string }[] = [
  { value: 'map', label: 'Solo mapa', desc: 'Archivo .ark del mundo (más rápido)' },
  { value: 'map_players_tribes', label: 'Mapa + Jugadores + Tribus', desc: 'Backup completo de partida (recomendado)' },
  { value: 'full', label: 'Directorio Saved/ completo', desc: 'Todo incluyendo configs del servidor' },
]

const PROVIDERS: { value: CloudProvider; label: string; color: string }[] = [
  { value: 'none', label: 'Sin proveedor', color: '#6b7280' },
  { value: 's3', label: 'S3-compatible', color: '#f97316' },
  { value: 'gdrive', label: 'Google Drive', color: '#4ade80' },
  { value: 'onedrive', label: 'OneDrive', color: '#3b82f6' },
  { value: 'icloud', label: 'iCloud', color: '#a78bfa' },
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
  const [restoring, setRestoring] = useState<string | null>(null) // key being restored
  const [restoreStatus, setRestoreStatus] = useState<string | null>(null)
  const [restoreError, setRestoreError] = useState<string | null>(null)

  const store = useBackupStore()
  const { config } = useConfigStore()

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
  }), [store])

  // ── List cloud backups ───────────────────────────────────
  const handleListBackups = async () => {
    setIsListing(true)
    setListError(null)
    setRestoreStatus(null)
    setRestoreError(null)
    try {
      const entries = await invoke<BackupListEntry[]>('list_cloud_backups', providerArgs())
      setCloudList(entries)
      if (entries.length === 0) setListError('No se encontraron backups en este proveedor.')
    } catch (e) {
      setListError(`❌ ${String(e)}`)
    } finally {
      setIsListing(false)
    }
  }

  // ── Restore a backup ─────────────────────────────────────
  const handleRestore = async (entry: BackupListEntry) => {
    if (!config) return
    const confirmed = window.confirm(
      `¿Restaurar "${entry.name}"?\n\n` +
      `Los saves actuales se renombrarán a SavedArks_preRestore_* como snapshot de seguridad antes de extraer el backup.\n\n` +
      `Asegúrate de que el servidor NO esté en ejecución.`
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

  // ── Backup now ───────────────────────────────────────────
  const handleBackupNow = async () => {
    if (!config) return
    setIsBacking(true)
    setBackupStatus(null)
    setBackupError(null)
    try {
      const token = store.provider === 'gdrive' ? store.gdriveAccessToken
        : store.provider === 'onedrive' ? store.onedriveAccessToken
        : undefined

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
      })

      store.addBackupEntry({
        filename,
        size_bytes: 0,
        created_at: new Date().toISOString(),
        provider: store.provider,
      })
      setBackupStatus(`✅ Backup completado: ${filename}`)
    } catch (e) {
      setBackupError(`❌ ${String(e)}`)
    } finally {
      setIsBacking(false)
    }
  }

  // ── Test S3 connection ───────────────────────────────────
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

  // ── OAuth flows ──────────────────────────────────────────
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
      setAuthStatus('✅ Google Drive autorizado correctamente')
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
      setAuthStatus('✅ OneDrive autorizado correctamente')
    } catch (e) {
      setAuthStatus(`❌ ${String(e)}`)
    } finally {
      setIsAuthing(false)
    }
  }

  const isConfigured = () => {
    switch (store.provider) {
      case 's3': return !!(store.s3Endpoint && store.s3Bucket && store.s3AccessKey && store.s3SecretKey)
      case 'gdrive': return !!store.gdriveAccessToken
      case 'onedrive': return !!store.onedriveAccessToken
      case 'icloud': return !!store.icloudPath
      default: return false
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: 'rgba(0,0,0,0.75)', backdropFilter: 'blur(4px)' }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        className="ark-panel rounded-xl w-full max-w-2xl max-h-[90vh] flex flex-col"
        style={{ border: '1px solid rgba(0,200,255,0.3)', boxShadow: '0 0 40px rgba(0,200,255,0.1)' }}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 pt-5 pb-4 border-b border-ark-cyan/20 flex-shrink-0">
          <span className="text-ark-cyan font-bold tracking-widest text-sm uppercase">⚙ Opciones</span>
          <button
            onClick={onClose}
            className="text-ark-cyan/40 hover:text-ark-cyan/80 text-xs tracking-widest transition-colors"
          >
            ESC / CERRAR
          </button>
        </div>

        {/* Tab bar */}
        <div className="flex border-b border-ark-cyan/20 flex-shrink-0">
          {(['general', 'backup'] as OptionsTab[]).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className="px-6 py-3 text-xs font-bold tracking-widest uppercase transition-colors"
              style={{
                color: tab === t ? 'rgba(0,200,255,0.9)' : 'rgba(0,200,255,0.35)',
                borderBottom: tab === t ? '2px solid rgba(0,200,255,0.8)' : '2px solid transparent',
              }}
            >
              {t === 'general' ? 'General' : 'Backup en la Nube'}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-5">

          {tab === 'general' && (
            <>
              {/* On-demand servers */}
              <Section title="Servidor Bajo Demanda">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">Activar modo dormido</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">
                      Permite que mapas individuales aparezcan en el browser de ARK sin que el servidor esté corriendo.
                      ARK arranca solo cuando alguien intenta conectarse. Útil para servidores locales con recursos limitados.
                      Desactívalo si usas un servidor en la nube 24/7.
                    </p>
                  </div>
                  <Toggle value={store.onDemandEnabled} onChange={store.setOnDemandEnabled} />
                </div>
              </Section>

              {/* Minimize to tray */}
              <Section title="Comportamiento al Cerrar">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">Minimizar a la bandeja del sistema</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">
                      Al cerrar la ventana, el manager se oculta en la barra de tareas (como Discord).
                      Haz clic en el icono de bandeja para volver a abrirlo o usa "Salir" para cerrarlo del todo.
                    </p>
                  </div>
                  <Toggle value={store.minimizeToTray} onChange={store.setMinimizeToTray} />
                </div>
              </Section>

              {/* Manual save */}
              <Section title="Guardado">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">Guardado manual</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">
                      Por defecto los cambios se guardan automáticamente. Activa esto para guardar solo al pulsar <span className="text-ark-cyan/60 font-mono">SAVE SETTINGS</span>.
                    </p>
                  </div>
                  <Toggle value={store.manualSave} onChange={store.setManualSave} />
                </div>
              </Section>

              {/* Logs toggle */}
              <Section title="Visor de Logs">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-ark-cyan/80 text-sm">Botón de logs del servidor</p>
                    <p className="text-ark-cyan/40 text-xs mt-0.5">Muestra el botón LOGS en la barra inferior para ver ShooterGame.log en tiempo real</p>
                  </div>
                  <Toggle value={store.logsEnabled} onChange={store.setLogsEnabled} />
                </div>
              </Section>

              {/* Backup scope */}
              <Section title="Scope del Backup">
                <div className="space-y-2">
                  {SCOPE_OPTIONS.map((opt) => (
                    <label
                      key={opt.value}
                      className="flex items-center gap-3 cursor-pointer p-2.5 rounded-md transition-colors"
                      style={{
                        background: store.backupScope === opt.value ? 'rgba(0,200,255,0.08)' : 'transparent',
                        border: `1px solid ${store.backupScope === opt.value ? 'rgba(0,200,255,0.4)' : 'rgba(255,255,255,0.06)'}`,
                      }}
                    >
                      <input
                        type="radio"
                        name="scope"
                        value={opt.value}
                        checked={store.backupScope === opt.value}
                        onChange={() => store.setBackupScope(opt.value)}
                        className="accent-ark-cyan"
                      />
                      <div>
                        <p className="text-ark-cyan/80 text-sm font-semibold">{opt.label}</p>
                        <p className="text-ark-cyan/40 text-xs">{opt.desc}</p>
                      </div>
                    </label>
                  ))}
                </div>
              </Section>

              {/* Max saves */}
              <Section title="Saves a conservar">
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
                    {store.maxSaves === 1 ? 'Solo el último' : `Últimos ${store.maxSaves}`}
                  </span>
                </div>
              </Section>
            </>
          )}

          {tab === 'backup' && (
            <>
              {/* Provider selector */}
              <Section title="Proveedor de la Nube">
                <div className="grid grid-cols-2 gap-2">
                  {PROVIDERS.map((p) => (
                    <button
                      key={p.value}
                      onClick={() => store.setProvider(p.value)}
                      className="rounded-md p-3 text-left text-sm font-semibold transition-all"
                      style={{
                        background: store.provider === p.value ? `${p.color}18` : 'rgba(255,255,255,0.03)',
                        border: `1px solid ${store.provider === p.value ? `${p.color}60` : 'rgba(255,255,255,0.08)'}`,
                        color: store.provider === p.value ? p.color : 'rgba(255,255,255,0.45)',
                        boxShadow: store.provider === p.value ? `0 0 12px ${p.color}20` : 'none',
                      }}
                    >
                      {p.label}
                    </button>
                  ))}
                </div>
              </Section>

              {/* S3 config */}
              {store.provider === 's3' && (
                <Section title="Configuración S3-compatible">
                  <p className="text-ark-cyan/40 text-xs mb-3">
                    Compatible con AWS S3, Contabo Object Storage, Seenode, Cloudflare R2, Backblaze B2, Wasabi, MinIO, etc.
                  </p>
                  <div className="space-y-2">
                    <Field label="Endpoint URL" value={store.s3Endpoint} onChange={(v) => store.setS3Field('s3Endpoint', v)} placeholder="https://eu2.contabostorage.com" />
                    <Field label="Bucket" value={store.s3Bucket} onChange={(v) => store.setS3Field('s3Bucket', v)} placeholder="ark-backups" />
                    <Field label="Access Key" value={store.s3AccessKey} onChange={(v) => store.setS3Field('s3AccessKey', v)} placeholder="AKIAIOSFODNN7EXAMPLE" />
                    <Field label="Secret Key" value={store.s3SecretKey} onChange={(v) => store.setS3Field('s3SecretKey', v)} placeholder="wJalrXUtnFEMI/K7MDENG/..." type="password" />
                    <Field label="Región" value={store.s3Region} onChange={(v) => store.setS3Field('s3Region', v)} placeholder="us-east-1" />
                  </div>
                  <div className="mt-3 flex items-center gap-3">
                    <button
                      onClick={handleTestS3}
                      disabled={isTesting}
                      className="ark-action-btn text-xs px-4 py-1.5"
                    >
                      {isTesting ? '⏳ Probando...' : '🔌 Probar conexión'}
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
                    <p className="text-green-400/70 text-xs mt-2">✅ Autorizado — token activo</p>
                  )}
                  <div className="mt-3 flex items-center gap-3">
                    <button
                      onClick={handleGDriveAuth}
                      disabled={isAuthing || !store.gdriveClientId || !store.gdriveClientSecret}
                      className="ark-action-btn text-xs px-4 py-1.5"
                    >
                      {isAuthing ? '⏳ Esperando autorización...' : store.gdriveAccessToken ? '🔄 Re-autorizar' : '🔑 Autorizar con Google'}
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
                    <p className="text-green-400/70 text-xs mt-2">✅ Autorizado — token activo</p>
                  )}
                  <div className="mt-3 flex items-center gap-3">
                    <button
                      onClick={handleOneDriveAuth}
                      disabled={isAuthing || !store.onedriveClientId}
                      className="ark-action-btn text-xs px-4 py-1.5"
                    >
                      {isAuthing ? '⏳ Esperando autorización...' : store.onedriveAccessToken ? '🔄 Re-autorizar' : '🔑 Autorizar con Microsoft'}
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
                <Section title="iCloud Drive (carpeta local)">
                  <p className="text-ark-cyan/40 text-xs mb-3">
                    Requiere la app de iCloud para Windows instalada. Los archivos se copian al directorio local de iCloud Drive y se sincronizan automáticamente. Ejemplo: <span className="font-mono text-ark-cyan/60">C:\Users\Max\iCloudDrive</span>
                  </p>
                  <Field label="Ruta de iCloud Drive" value={store.icloudPath} onChange={store.setICloudPath} placeholder="C:\Users\Max\iCloudDrive" />
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
                      {isBacking ? '⏳ Creando backup...' : '💾 BACKUP AHORA'}
                    </button>
                    {!isConfigured() && (
                      <span className="text-ark-cyan/40 text-xs">Completa la configuración del proveedor</span>
                    )}
                  </div>

                  {backupStatus && (
                    <p className="text-green-400/80 text-xs">{backupStatus}</p>
                  )}
                  {backupError && (
                    <p className="text-red-400/80 text-xs">{backupError}</p>
                  )}

                  {store.backupHistory.length > 0 && (
                    <Section title="Historial de Backups">
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

                  {/* ── Restore from cloud ── */}
                  <Section title="Restaurar desde la Nube">
                    <p className="text-ark-cyan/40 text-xs">
                      Lista los backups disponibles en tu proveedor y restaura cualquiera de ellos. Los saves actuales se guardan automáticamente como snapshot de seguridad antes de restaurar.
                    </p>

                    <div className="flex items-center gap-3 pt-1">
                      <button
                        onClick={handleListBackups}
                        disabled={isListing || !isConfigured()}
                        className="ark-action-btn text-xs px-4 py-1.5"
                        style={{ opacity: isConfigured() ? 1 : 0.4 }}
                      >
                        {isListing ? '⏳ Buscando...' : '🔍 Listar backups disponibles'}
                      </button>
                    </div>

                    {listError && (
                      <p className="text-red-400/80 text-xs">{listError}</p>
                    )}

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
                              </div>
                              <button
                                onClick={() => handleRestore(entry)}
                                disabled={!!restoring}
                                className="flex-shrink-0 ark-action-btn text-[10px] px-3 py-1"
                                style={{ opacity: restoring ? 0.4 : 1 }}
                              >
                                {isRestoring ? '⏳' : '⏪ Restaurar'}
                              </button>
                            </div>
                          )
                        })}
                      </div>
                    )}

                    {restoreStatus && (
                      <p className="text-green-400/80 text-xs">{restoreStatus}</p>
                    )}
                    {restoreError && (
                      <p className="text-red-400/80 text-xs">{restoreError}</p>
                    )}
                  </Section>
                </>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-ark-cyan/20 flex justify-end flex-shrink-0">
          <button onClick={onClose} className="ark-action-btn px-6 py-2 text-xs tracking-widest">
            CERRAR
          </button>
        </div>
      </div>
    </div>
  )
}

// ─── Small helpers ───────────────────────────────────────────

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
