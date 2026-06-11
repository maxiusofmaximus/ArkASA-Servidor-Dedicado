import React from 'react'
import { useUiStore } from '../stores/uiStore'
import type { GameRulesSubTab, AdvancedSubTab, ModSettingsSubTab, PrimaryTab } from '../types'

interface SubNavProps {
  primaryTab: PrimaryTab
}

const gameRulesTabs: { id: GameRulesSubTab; label: string }[] = [
  { id: 'player', label: 'PLAYER' },
  { id: 'creature', label: 'CREATURE' },
  { id: 'structure', label: 'STRUCTURE' },
  { id: 'world', label: 'WORLD' },
  { id: 'rules', label: 'RULES' },
]

const advancedTabs: { id: AdvancedSubTab; label: string }[] = [
  { id: 'pve', label: 'PVE' },
  { id: 'pvp', label: 'PVP' },
  { id: 'world', label: 'WORLD' },
  { id: 'wild_dino', label: 'WILD DINO' },
  { id: 'tamed_dino', label: 'TAMED DINO' },
  { id: 'player', label: 'PLAYER' },
  { id: 'xp_multipliers', label: 'XP MULTIPLIERS' },
  { id: 'misc', label: 'MISC' },
]

const modSettingsTabs: { id: ModSettingsSubTab; label: string }[] = [
  { id: 'active_mods', label: 'ACTIVE MODS' },
  { id: 'available_mods', label: 'AVAILABLE MODS' },
]

export default function SubNav({ primaryTab }: SubNavProps) {
  const { gameRulesSubTab, setGameRulesSubTab, advancedSubTab, setAdvancedSubTab, modSettingsSubTab, setModSettingsSubTab } = useUiStore()

  if (primaryTab !== 'game_rules' && primaryTab !== 'advanced' && primaryTab !== 'mod_settings') {
    return null
  }

  if (primaryTab === 'game_rules') {
    return (
      <div className="border-b border-ark-cyan/20 bg-ark-dark/50 px-8 py-2">
        <div className="flex gap-1">
          {gameRulesTabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setGameRulesSubTab(tab.id)}
              className={`px-4 py-1 text-xs font-semibold tracking-widest uppercase transition ${
                gameRulesSubTab === tab.id ? 'text-white bg-ark-secondary/50' : 'text-ark-cyan/60 hover:text-ark-cyan/80'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>
    )
  }

  if (primaryTab === 'advanced') {
    return (
      <div className="border-b border-ark-cyan/20 bg-ark-dark/50 px-8 py-2">
        <div className="flex gap-1">
          {advancedTabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setAdvancedSubTab(tab.id)}
              className={`px-4 py-1 text-xs font-semibold tracking-widest uppercase transition ${
                advancedSubTab === tab.id ? 'text-white bg-ark-secondary/50' : 'text-ark-cyan/60 hover:text-ark-cyan/80'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>
    )
  }

  if (primaryTab === 'mod_settings') {
    return (
      <div className="border-b border-ark-cyan/20 bg-ark-dark/50 px-8 py-2">
        <div className="flex gap-1">
          {modSettingsTabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setModSettingsSubTab(tab.id)}
              className={`px-4 py-1 text-xs font-semibold tracking-widest uppercase transition ${
                modSettingsSubTab === tab.id ? 'text-white bg-ark-secondary/50' : 'text-ark-cyan/60 hover:text-ark-cyan/80'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>
    )
  }

  return null
}
