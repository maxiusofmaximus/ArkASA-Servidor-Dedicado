import { create } from 'zustand'
import type { ServerConfig, ValidationResult } from '../types'

interface ConfigStore {
  config: ServerConfig | null
  validationResult: ValidationResult | null
  isSaving: boolean
  setConfig: (config: ServerConfig) => void
  updateConfig: (updates: Partial<ServerConfig>) => void
  setValidationResult: (result: ValidationResult | null) => void
  setSaving: (saving: boolean) => void
  reset: () => void
}

export const useConfigStore = create<ConfigStore>((set) => ({
  config: null,
  validationResult: null,
  isSaving: false,

  setConfig: (config) => set({ config }),

  updateConfig: (updates) =>
    set((state) => ({
      config: state.config ? { ...state.config, ...updates } : null,
    })),

  setValidationResult: (result) => set({ validationResult: result }),

  setSaving: (saving) => set({ isSaving: saving }),

  reset: () =>
    set({
      config: null,
      validationResult: null,
      isSaving: false,
    }),
}))
