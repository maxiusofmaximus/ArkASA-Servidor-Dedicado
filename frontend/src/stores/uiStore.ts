import { create } from 'zustand'
import type { PrimaryTab, GameRulesSubTab, AdvancedSubTab, ModSettingsSubTab } from '../types'

interface NavSnapshot {
  primaryTab: PrimaryTab
  gameRulesSubTab: GameRulesSubTab
  advancedSubTab: AdvancedSubTab
  modSettingsSubTab: ModSettingsSubTab
}

interface UiStore {
  primaryTab: PrimaryTab
  gameRulesSubTab: GameRulesSubTab
  advancedSubTab: AdvancedSubTab
  modSettingsSubTab: ModSettingsSubTab
  navHistory: NavSnapshot[]

  setPrimaryTab: (tab: PrimaryTab) => void
  setGameRulesSubTab: (tab: GameRulesSubTab) => void
  setAdvancedSubTab: (tab: AdvancedSubTab) => void
  setModSettingsSubTab: (tab: ModSettingsSubTab) => void
  goBack: () => void
}

const snapshot = (s: UiStore): NavSnapshot => ({
  primaryTab: s.primaryTab,
  gameRulesSubTab: s.gameRulesSubTab,
  advancedSubTab: s.advancedSubTab,
  modSettingsSubTab: s.modSettingsSubTab,
})

export const useUiStore = create<UiStore>((set, get) => ({
  primaryTab: 'arks',
  gameRulesSubTab: 'player',
  advancedSubTab: 'pve',
  modSettingsSubTab: 'active_mods',
  navHistory: [],

  setPrimaryTab: (tab) => set((s) => ({
    navHistory: [...s.navHistory, snapshot(s)].slice(-20),
    primaryTab: tab,
  })),

  setGameRulesSubTab: (tab) => set((s) => ({
    navHistory: [...s.navHistory, snapshot(s)].slice(-20),
    gameRulesSubTab: tab,
  })),

  setAdvancedSubTab: (tab) => set((s) => ({
    navHistory: [...s.navHistory, snapshot(s)].slice(-20),
    advancedSubTab: tab,
  })),

  setModSettingsSubTab: (tab) => set((s) => ({
    navHistory: [...s.navHistory, snapshot(s)].slice(-20),
    modSettingsSubTab: tab,
  })),

  goBack: () => set((s) => {
    if (s.navHistory.length === 0) return s
    const prev = s.navHistory[s.navHistory.length - 1]
    return {
      ...prev,
      navHistory: s.navHistory.slice(0, -1),
    }
  }),
}))
