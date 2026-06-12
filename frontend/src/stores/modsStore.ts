import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export interface ModInfo {
  name: string
  summary: string
  logoUrl: string | null
  slug: string
  downloadCount: number
  categories: string[]
  clientOnly?: boolean
}

interface ModsStore {
  modCache: Record<string, ModInfo>
  apiKey: string  // persisted so AvailableModsTab doesn't re-prompt on every tab switch
  setModInfo: (id: string, info: ModInfo) => void
  getModInfo: (id: string) => ModInfo | undefined
  getModName: (id: string) => string | undefined
  setApiKey: (key: string) => void
}

export const useModsStore = create<ModsStore>()(
  persist(
    (set, get) => ({
      modCache: {},
      apiKey: '',

      setModInfo: (id, info) =>
        set((state) => ({
          modCache: { ...state.modCache, [id]: info },
        })),

      getModInfo: (id) => get().modCache[id],

      getModName: (id) => get().modCache[id]?.name,

      setApiKey: (key) => set({ apiKey: key }),
    }),
    { name: 'ark-mods-name-cache' }
  )
)
