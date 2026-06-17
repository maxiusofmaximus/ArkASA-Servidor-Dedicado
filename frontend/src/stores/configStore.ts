import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { ServerConfig, ValidationResult } from '../types'

const MAX_HISTORY = 50
const RAPID_MS    = 800  // group changes within this window into one undo step

interface ConfigStore {
  config: ServerConfig | null
  savedConfig: ServerConfig | null  // last successfully saved snapshot
  history: ServerConfig[]
  historyIndex: number
  lastHistoryTime: number

  validationResult: ValidationResult | null
  isSaving: boolean

  setConfig: (config: ServerConfig) => void           // load / reset — clears history
  updateConfig: (updates: Partial<ServerConfig>) => void  // user edit — pushes history
  setSavedConfig: (config: ServerConfig) => void
  undo: () => void
  redo: () => void

  setValidationResult: (result: ValidationResult | null) => void
  setSaving: (saving: boolean) => void
  reset: () => void
}

export const useConfigStore = create<ConfigStore>()(
  persist(
    (set) => ({
      config: null,
      savedConfig: null,
      history: [],
      historyIndex: -1,
      lastHistoryTime: 0,
      validationResult: null,
      isSaving: false,

      // Replace config wholesale (load / reset / import) — fresh history
      setConfig: (config) =>
        set({ config, history: [config], historyIndex: 0, lastHistoryTime: 0 }),

      // User-driven edit — push to history with rapid-change debouncing
      updateConfig: (updates) =>
        set((state) => {
          if (!state.config) return {}
          const now = Date.now()
          const newConfig = { ...state.config, ...updates }
          const isRapid = now - state.lastHistoryTime < RAPID_MS

          // Truncate any "redo" future
          const base = state.history.slice(0, state.historyIndex + 1)

          let history: ServerConfig[]
          let historyIndex: number

          if (isRapid && base.length > 0) {
            // Replace the current entry — debounce rapid keystrokes
            history = [...base.slice(0, -1), newConfig]
            historyIndex = history.length - 1
          } else {
            history = [...base, newConfig]
            // Keep within max
            if (history.length > MAX_HISTORY) history = history.slice(history.length - MAX_HISTORY)
            historyIndex = history.length - 1
          }

          return { config: newConfig, history, historyIndex, lastHistoryTime: now }
        }),

      setSavedConfig: (config) => set({ savedConfig: config }),

      undo: () =>
        set((state) => {
          if (state.historyIndex <= 0) return {}
          const historyIndex = state.historyIndex - 1
          return { config: state.history[historyIndex], historyIndex }
        }),

      redo: () =>
        set((state) => {
          if (state.historyIndex >= state.history.length - 1) return {}
          const historyIndex = state.historyIndex + 1
          return { config: state.history[historyIndex], historyIndex }
        }),

      setValidationResult: (result) => set({ validationResult: result }),
      setSaving: (saving) => set({ isSaving: saving }),

      reset: () =>
        set({
          config: null,
          savedConfig: null,
          history: [],
          historyIndex: -1,
          lastHistoryTime: 0,
          validationResult: null,
          isSaving: false,
        }),
    }),
    {
      name: 'ark-asa-config',
      // Only persist config; skip transient undo stack and UI state
      partialize: (state) => ({ config: state.config }),
    }
  )
)
