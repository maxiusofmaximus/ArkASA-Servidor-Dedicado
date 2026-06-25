import { useBackupStore, type CloudProvider } from '../../stores/backupStore'
import { useI18n } from '../../i18n/useI18n'
import { Section, Field } from '../ui/OptionsUI'
import type { UseBackupActionsReturn } from '../../hooks/useBackupActions'

const PROVIDER_COLORS: Record<CloudProvider, string> = {
  none:         '#6b7280',
  s3:           '#f97316',
  gdrive:       '#4ade80',
  onedrive:     '#3b82f6',
  icloud:       '#a78bfa',
  local_folder: '#22d3ee',
}

interface BackupTabProps {
  actions: UseBackupActionsReturn
}

export default function BackupTab({ actions }: BackupTabProps) {
  const store = useBackupStore()
  const { tk } = useI18n()

  const isCloudProvider = !['none', 'icloud'].includes(store.provider)

  return (
    <>
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
              onClick={actions.handleTestS3}
              disabled={actions.isTesting}
              className="ark-action-btn text-xs px-4 py-1.5"
            >
              {actions.isTesting ? tk('testing', '⏳ Testing...') : tk('test_connection', '🔌 Test connection')}
            </button>
            {actions.testStatus && (
              <span className={`text-xs ${actions.testStatus.startsWith('✅') ? 'text-green-400' : 'text-red-400'}`}>
                {actions.testStatus}
              </span>
            )}
          </div>
        </Section>
      )}

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
              onClick={actions.handleGDriveAuth}
              disabled={actions.isAuthing || !store.gdriveClientId || !store.gdriveClientSecret}
              className="ark-action-btn text-xs px-4 py-1.5"
            >
              {actions.isAuthing ? tk('waiting_auth', '⏳ Waiting for authorization...') : store.gdriveAccessToken ? tk('reauth', '🔄 Re-authorize') : tk('auth_google', '🔑 Authorize with Google')}
            </button>
          </div>
          {actions.authStatus && (
            <p className={`text-xs mt-2 ${actions.authStatus.startsWith('✅') ? 'text-green-400' : 'text-red-400'}`}>
              {actions.authStatus}
            </p>
          )}
        </Section>
      )}

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
              onClick={actions.handleOneDriveAuth}
              disabled={actions.isAuthing || !store.onedriveClientId}
              className="ark-action-btn text-xs px-4 py-1.5"
            >
              {actions.isAuthing ? tk('waiting_auth', '⏳ Waiting for authorization...') : store.onedriveAccessToken ? tk('reauth', '🔄 Re-authorize') : tk('auth_microsoft', '🔑 Authorize with Microsoft')}
            </button>
          </div>
          {actions.authStatus && (
            <p className={`text-xs mt-2 ${actions.authStatus.startsWith('✅') ? 'text-green-400' : 'text-red-400'}`}>
              {actions.authStatus}
            </p>
          )}
        </Section>
      )}

      {store.provider === 'icloud' && (
        <Section title="iCloud Drive">
          <Field label={tk('icloud_path', 'iCloud Drive path')} value={store.icloudPath} onChange={store.setICloudPath} placeholder="C:\Users\Max\iCloudDrive" />
        </Section>
      )}

      {store.provider !== 'none' && (
        <>
          <div className="flex items-center gap-3 pt-2">
            <button
              onClick={actions.handleBackupNow}
              disabled={actions.isBacking || !actions.isConfigured()}
              className="ark-action-btn px-5 py-2 text-xs font-bold tracking-widest"
              style={{ opacity: actions.isConfigured() ? 1 : 0.4 }}
            >
              {actions.isBacking ? tk('backing_up', '⏳ Creating backup...') : tk('backup_now_btn', '💾 BACKUP NOW')}
            </button>
            {!actions.isConfigured() && (
              <span className="text-ark-cyan/40 text-xs">{tk('complete_provider_config', 'Complete the provider configuration')}</span>
            )}
          </div>

          {actions.backupStatus && <p className="text-green-400/80 text-xs">{actions.backupStatus}</p>}
          {actions.backupError && <p className="text-red-400/80 text-xs">{actions.backupError}</p>}

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

          {isCloudProvider && (
            <Section title={tk('section_restore_cloud', 'Restore from Cloud')}>
              <div className="flex items-center gap-3 pt-1">
                <button
                  onClick={actions.handleListBackups}
                  disabled={actions.isListing || !actions.isConfigured()}
                  className="ark-action-btn text-xs px-4 py-1.5"
                  style={{ opacity: actions.isConfigured() ? 1 : 0.4 }}
                >
                  {actions.isListing ? tk('searching', '⏳ Searching...') : tk('list_backups_btn', '🔍 List available backups')}
                </button>
              </div>

              {actions.listError && <p className="text-red-400/80 text-xs">{actions.listError}</p>}

              {actions.cloudList.length > 0 && (
                <div className="space-y-1.5 max-h-52 overflow-y-auto mt-2">
                  {actions.cloudList.map((entry) => {
                    const isRestoring = actions.restoring === entry.key
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
                          {actions.backupMeta[entry.key]?.mod_ids && actions.backupMeta[entry.key]!.mod_ids!.length > 0 && (
                            <p className="text-ark-cyan/40 text-[10px] mt-0.5 truncate">
                              Mods: {actions.backupMeta[entry.key]!.mod_ids!.slice(0, 5).join(', ')}
                              {actions.backupMeta[entry.key]!.mod_ids!.length > 5 && ` +${actions.backupMeta[entry.key]!.mod_ids!.length - 5} más`}
                            </p>
                          )}
                          {actions.backupMeta[entry.key]?.note && (
                            <p className="text-yellow-400/50 text-[10px] mt-0.5 truncate">
                              ℹ {actions.backupMeta[entry.key]!.note}
                            </p>
                          )}
                        </div>
                        <button
                          onClick={() => actions.handleRestore(entry)}
                          disabled={!!actions.restoring}
                          className="flex-shrink-0 ark-action-btn text-[10px] px-3 py-1"
                          style={{ opacity: actions.restoring ? 0.4 : 1 }}
                        >
                          {isRestoring ? tk('restoring', '⏳') : tk('restore_btn', '⏪ Restore')}
                        </button>
                      </div>
                    )
                  })}
                </div>
              )}

              {actions.restoreStatus && <p className="text-green-400/80 text-xs">{actions.restoreStatus}</p>}
              {actions.restoreError && <p className="text-red-400/80 text-xs">{actions.restoreError}</p>}
            </Section>
          )}
        </>
      )}
    </>
  )
}
