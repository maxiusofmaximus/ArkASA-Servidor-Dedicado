import { useState, useCallback } from 'react'
import { invoke } from '../services/tauri'
import { useBackupStore } from '../stores/backupStore'
import type { ServerConfig } from '../types'
import { useI18n } from '../i18n/useI18n'

interface BackupListEntry {
  key: string
  name: string
  size_bytes: number
  created_at: string
}

export interface UseBackupActionsReturn {
  handleBackupNow: () => Promise<void>
  handleTestS3: () => Promise<void>
  handleGDriveAuth: () => Promise<void>
  handleOneDriveAuth: () => Promise<void>
  handleListBackups: () => Promise<void>
  handleRestore: (entry: BackupListEntry) => Promise<void>
  isBacking: boolean
  isTesting: boolean
  isAuthing: boolean
  isListing: boolean
  backupStatus: string | null
  backupError: string | null
  testStatus: string | null
  authStatus: string | null
  listError: string | null
  cloudList: BackupListEntry[]
  backupMeta: Record<string, { mod_ids?: string[]; server_name?: string; note?: string } | null>
  restoring: string | null
  restoreStatus: string | null
  restoreError: string | null
  isConfigured: () => boolean
}

export function useBackupActions(config: ServerConfig | null): UseBackupActionsReturn {
  const store = useBackupStore()
  const { tk } = useI18n()

  const [backupStatus, setBackupStatus] = useState<string | null>(null)
  const [backupError, setBackupError] = useState<string | null>(null)
  const [testStatus, setTestStatus] = useState<string | null>(null)
  const [authStatus, setAuthStatus] = useState<string | null>(null)
  const [isBacking, setIsBacking] = useState(false)
  const [isTesting, setIsTesting] = useState(false)
  const [isAuthing, setIsAuthing] = useState(false)
  const [cloudList, setCloudList] = useState<BackupListEntry[]>([])
  const [isListing, setIsListing] = useState(false)
  const [listError, setListError] = useState<string | null>(null)
  const [restoring, setRestoring] = useState<string | null>(null)
  const [restoreStatus, setRestoreStatus] = useState<string | null>(null)
  const [restoreError, setRestoreError] = useState<string | null>(null)
  const [backupMeta, setBackupMeta] = useState<Record<string, { mod_ids?: string[]; server_name?: string; note?: string } | null>>({})

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

  const buildMetadataJson = useCallback(() => {
    if (!config) return null
    return JSON.stringify({
      server_name: config.identification.session_name,
      mod_ids: config.mods.active_mods,
      scope: store.backupScope,
      backed_up_at: new Date().toISOString(),
    })
  }, [config, store.backupScope])

  const handleListBackups = async () => {
    setIsListing(true)
    setListError(null)
    setRestoreStatus(null)
    setRestoreError(null)
    try {
      const entries = await invoke<BackupListEntry[]>('list_cloud_backups', providerArgs())
      setCloudList(entries)
      if (entries.length === 0) setListError(tk('no_backups_found', 'No backups found for this provider.'))
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

  const handleBackupNow = async () => {
    if (!config) return
    setIsBacking(true)
    setBackupStatus(null)
    setBackupError(null)
    try {
      const token = store.provider === 'gdrive' ? store.gdriveAccessToken
        : store.provider === 'onedrive' ? store.onedriveAccessToken
        : undefined
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

  const isConfigured = useCallback((): boolean => {
    switch (store.provider) {
      case 's3':           return !!(store.s3Endpoint && store.s3Bucket && store.s3AccessKey && store.s3SecretKey)
      case 'gdrive':       return !!store.gdriveAccessToken
      case 'onedrive':     return !!store.onedriveAccessToken
      case 'icloud':       return !!store.icloudPath
      case 'local_folder': return !!store.localFolderPath
      default:             return false
    }
  }, [store])

  return {
    handleBackupNow,
    handleTestS3,
    handleGDriveAuth,
    handleOneDriveAuth,
    handleListBackups,
    handleRestore,
    isBacking,
    isTesting,
    isAuthing,
    isListing,
    backupStatus,
    backupError,
    testStatus,
    authStatus,
    listError,
    cloudList,
    backupMeta,
    restoring,
    restoreStatus,
    restoreError,
    isConfigured,
  }
}
