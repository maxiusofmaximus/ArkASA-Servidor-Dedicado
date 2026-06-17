import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { ServerConfig, ValidationResult } from '../types'

interface ConfigStore {
  config: ServerConfig | null
  savedConfig: ServerConfig | null  // snapshot of last successfully saved config
  validationResult: ValidationResult | null
  isSaving: boolean
  setConfig: (config: ServerConfig) => void
  updateConfig: (updates: Partial<ServerConfig>) => void
  setSavedConfig: (config: ServerConfig) => void
  setValidationResult: (result: ValidationResult | null) => void
  setSaving: (saving: boolean) => void
  reset: () => void
}

export const useConfigStore = create<ConfigStore>()(
  persist(
    (set) => ({
      config: null,
      savedConfig: null,
      validationResult: null,
      isSaving: false,

      setConfig: (config) => set({ config }),

      updateConfig: (updates) =>
        set((state) => ({
          config: state.config ? { ...state.config, ...updates } : null,
        })),

      setSavedConfig: (config) => set({ savedConfig: config }),

      setValidationResult: (result) => set({ validationResult: result }),

      setSaving: (saving) => set({ isSaving: saving }),

      reset: () =>
        set({
          config: null,
          savedConfig: null,
          validationResult: null,
          isSaving: false,
        }),
    }),
    {
      name: 'ark-asa-config',
      // Only persist config; skip transient UI state
      partialize: (state) => ({ config: state.config }),
    }
  )
)
