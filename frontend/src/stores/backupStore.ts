import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type BackupScope = 'map' | 'map_players_tribes' | 'full'
export type CloudProvider = 'none' | 's3' | 'gdrive' | 'onedrive' | 'icloud'

export interface BackupEntry {
  filename: string
  size_bytes: number
  created_at: string
  provider: CloudProvider
}

interface BackupStore {
  // General options
  logsEnabled: boolean
  minimizeToTray: boolean
  manualSave: boolean
  backupScope: BackupScope
  maxSaves: number // 1–10

  // On-demand server
  onDemandEnabled: boolean     // master switch — disables the whole dormant-stub feature
  onDemandMaps: string[]       // map IDs that should run as stubs instead of always-on
  autoShutdownMin: number      // minutes of empty server before auto-stop (0 = never)

  // Provider
  provider: CloudProvider

  // S3
  s3Endpoint: string
  s3Bucket: string
  s3AccessKey: string
  s3SecretKey: string
  s3Region: string

  // Google Drive
  gdriveClientId: string
  gdriveClientSecret: string
  gdriveAccessToken: string
  gdriveRefreshToken: string

  // OneDrive
  onedriveClientId: string
  onedriveAccessToken: string
  onedriveRefreshToken: string

  // iCloud
  icloudPath: string

  // History
  backupHistory: BackupEntry[]

  // Actions
  setLogsEnabled: (v: boolean) => void
  setMinimizeToTray: (v: boolean) => void
  setManualSave: (v: boolean) => void
  setOnDemandEnabled: (v: boolean) => void
  setOnDemandMaps: (v: string[]) => void
  toggleOnDemandMap: (mapId: string) => void
  setAutoShutdownMin: (v: number) => void
  setBackupScope: (v: BackupScope) => void
  setMaxSaves: (v: number) => void
  setProvider: (v: CloudProvider) => void
  setS3Field: (field: keyof Pick<BackupStore, 's3Endpoint' | 's3Bucket' | 's3AccessKey' | 's3SecretKey' | 's3Region'>, value: string) => void
  setGDriveField: (field: keyof Pick<BackupStore, 'gdriveClientId' | 'gdriveClientSecret' | 'gdriveAccessToken' | 'gdriveRefreshToken'>, value: string) => void
  setOneDriveField: (field: keyof Pick<BackupStore, 'onedriveClientId' | 'onedriveAccessToken' | 'onedriveRefreshToken'>, value: string) => void
  setICloudPath: (v: string) => void
  addBackupEntry: (entry: BackupEntry) => void
}

export const useBackupStore = create<BackupStore>()(
  persist(
    (set) => ({
      logsEnabled: false,
      minimizeToTray: true,
      manualSave: false,
      backupScope: 'map_players_tribes',
      maxSaves: 3,
      onDemandEnabled: true,
      onDemandMaps: [],
      autoShutdownMin: 30,
      provider: 'none',
      s3Endpoint: '',
      s3Bucket: '',
      s3AccessKey: '',
      s3SecretKey: '',
      s3Region: 'us-east-1',
      gdriveClientId: '',
      gdriveClientSecret: '',
      gdriveAccessToken: '',
      gdriveRefreshToken: '',
      onedriveClientId: '',
      onedriveAccessToken: '',
      onedriveRefreshToken: '',
      icloudPath: '',
      backupHistory: [],

      setLogsEnabled: (v) => set({ logsEnabled: v }),
      setMinimizeToTray: (v) => set({ minimizeToTray: v }),
      setManualSave: (v) => set({ manualSave: v }),
      setOnDemandEnabled: (v) => set({ onDemandEnabled: v }),
      setOnDemandMaps: (v) => set({ onDemandMaps: v }),
      toggleOnDemandMap: (mapId) => set((s) => ({
        onDemandMaps: s.onDemandMaps.includes(mapId)
          ? s.onDemandMaps.filter((m) => m !== mapId)
          : [...s.onDemandMaps, mapId],
      })),
      setAutoShutdownMin: (v) => set({ autoShutdownMin: v }),
      setBackupScope: (v) => set({ backupScope: v }),
      setMaxSaves: (v) => set({ maxSaves: v }),
      setProvider: (v) => set({ provider: v }),
      setS3Field: (field, value) => set({ [field]: value } as any),
      setGDriveField: (field, value) => set({ [field]: value } as any),
      setOneDriveField: (field, value) => set({ [field]: value } as any),
      setICloudPath: (v) => set({ icloudPath: v }),
      addBackupEntry: (entry) =>
        set((s) => ({
          backupHistory: [entry, ...s.backupHistory].slice(0, 50),
        })),
    }),
    { name: 'ark-backup-config' }
  )
)
