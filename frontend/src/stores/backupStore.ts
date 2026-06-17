import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { FriendContact } from '../types'

export type BackupScope = 'map' | 'map_players_tribes' | 'full'
export type CloudProvider = 'none' | 's3' | 'gdrive' | 'onedrive' | 'icloud' | 'local_folder'

export interface BackupEntry {
  filename: string
  size_bytes: number
  created_at: string
  provider: CloudProvider
}

export interface ModUsageRecord {
  firstSeen: string      // ISO date — when the mod was first added to the active list
  lastActive: string     // ISO date — last time server started with this mod
  serverLaunches: number // how many successful server starts included this mod
}

interface BackupStore {
  // General options
  language: string          // 'en' | 'es' | 'fr' | 'zh' | 'ja' | 'ko' | 'pt' | 'de' | 'it' | 'ru'
  logsEnabled: boolean
  minimizeToTray: boolean
  manualSave: boolean
  backupScope: BackupScope
  maxSaves: number

  // On-demand server
  onDemandEnabled: boolean
  onDemandMaps: string[]
  autoShutdownMin: number
  clusterStartDelaySec: number

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

  // Local folder
  localFolderPath: string

  // History
  backupHistory: BackupEntry[]

  // Friend Contacts
  friendContacts: FriendContact[]
  activePingContactId: string | null

  // Mod usage tracking (persisted — survives restarts)
  modUsageHistory: Record<string, ModUsageRecord>

  // Actions
  setLanguage: (v: string) => void
  setLogsEnabled: (v: boolean) => void
  setMinimizeToTray: (v: boolean) => void
  setManualSave: (v: boolean) => void
  setOnDemandEnabled: (v: boolean) => void
  setOnDemandMaps: (v: string[]) => void
  toggleOnDemandMap: (mapId: string) => void
  setAutoShutdownMin: (v: number) => void
  setClusterStartDelaySec: (v: number) => void
  setBackupScope: (v: BackupScope) => void
  setMaxSaves: (v: number) => void
  setProvider: (v: CloudProvider) => void
  setS3Field: (field: keyof Pick<BackupStore, 's3Endpoint' | 's3Bucket' | 's3AccessKey' | 's3SecretKey' | 's3Region'>, value: string) => void
  setGDriveField: (field: keyof Pick<BackupStore, 'gdriveClientId' | 'gdriveClientSecret' | 'gdriveAccessToken' | 'gdriveRefreshToken'>, value: string) => void
  setOneDriveField: (field: keyof Pick<BackupStore, 'onedriveClientId' | 'onedriveAccessToken' | 'onedriveRefreshToken'>, value: string) => void
  setICloudPath: (v: string) => void
  setLocalFolderPath: (v: string) => void
  addBackupEntry: (entry: BackupEntry) => void

  // Friend Contact actions
  addFriendContact: (contact: Omit<FriendContact, 'id'>) => void
  updateFriendContact: (id: string, patch: Partial<Omit<FriendContact, 'id'>>) => void
  removeFriendContact: (id: string) => void
  setActivePingContactId: (id: string | null) => void

  // Mod usage actions
  recordModsActive: (modIds: string[]) => void
}

export const useBackupStore = create<BackupStore>()(
  persist(
    (set) => ({
      language: 'es',
      logsEnabled: false,
      minimizeToTray: true,
      manualSave: false,
      backupScope: 'map_players_tribes',
      maxSaves: 3,
      onDemandEnabled: true,
      onDemandMaps: [],
      autoShutdownMin: 30,
      clusterStartDelaySec: 60,
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
      localFolderPath: '',
      backupHistory: [],
      friendContacts: [],
      activePingContactId: null,
      modUsageHistory: {},

      setLanguage: (v) => set({ language: v }),
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
      setClusterStartDelaySec: (v) => set({ clusterStartDelaySec: v }),
      setBackupScope: (v) => set({ backupScope: v }),
      setMaxSaves: (v) => set({ maxSaves: v }),
      setProvider: (v) => set({ provider: v }),
      setS3Field: (field, value) => set({ [field]: value } as any),
      setGDriveField: (field, value) => set({ [field]: value } as any),
      setOneDriveField: (field, value) => set({ [field]: value } as any),
      setICloudPath: (v) => set({ icloudPath: v }),
      setLocalFolderPath: (v) => set({ localFolderPath: v }),
      addBackupEntry: (entry) =>
        set((s) => ({
          backupHistory: [entry, ...s.backupHistory].slice(0, 50),
        })),

      addFriendContact: (contact) =>
        set((s) => ({
          friendContacts: [
            ...s.friendContacts,
            { ...contact, id: crypto.randomUUID() },
          ],
        })),
      updateFriendContact: (id, patch) =>
        set((s) => ({
          friendContacts: s.friendContacts.map((c) =>
            c.id === id ? { ...c, ...patch } : c
          ),
        })),
      removeFriendContact: (id) =>
        set((s) => ({
          friendContacts: s.friendContacts.filter((c) => c.id !== id),
        })),
      setActivePingContactId: (id) => set({ activePingContactId: id }),

      recordModsActive: (modIds) => set((s) => {
        const now = new Date().toISOString()
        const next = { ...s.modUsageHistory }
        for (const id of modIds) {
          const prev = next[id]
          next[id] = {
            firstSeen:     prev?.firstSeen ?? now,
            lastActive:    now,
            serverLaunches: (prev?.serverLaunches ?? 0) + 1,
          }
        }
        return { modUsageHistory: next }
      }),
    }),
    { name: 'ark-backup-config' }
  )
)
