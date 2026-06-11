import { create } from 'zustand'
import type { PrimaryTab, GameRulesSubTab, AdvancedSubTab, ModSettingsSubTab } from '../types'

interface UiStore {
  primaryTab: PrimaryTab
  gameRulesSubTab: GameRulesSubTab
  advancedSubTab: AdvancedSubTab
  modSettingsSubTab: ModSettingsSubTab

  setPrimaryTab: (tab: PrimaryTab) => void
  setGameRulesSubTab: (tab: GameRulesSubTab) => void
  setAdvancedSubTab: (tab: AdvancedSubTab) => void
  setModSettingsSubTab: (tab: ModSettingsSubTab) => void
}

export const useUiStore = create<UiStore>((set) => ({
  primaryTab: 'arks',
  gameRulesSubTab: 'player',
  advancedSubTab: 'pve',
  modSettingsSubTab: 'active_mods',

  setPrimaryTab: (tab) => set({ primaryTab: tab }),
  setGameRulesSubTab: (tab) => set({ gameRulesSubTab: tab }),
  setAdvancedSubTab: (tab) => set({ advancedSubTab: tab }),
  setModSettingsSubTab: (tab) => set({ modSettingsSubTab: tab }),
}))
