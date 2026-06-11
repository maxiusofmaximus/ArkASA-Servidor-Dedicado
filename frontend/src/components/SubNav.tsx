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

  if (primaryTab === 'arks' || primaryTab === 'engrams') {
    return null
  }

  const getTabs = () => {
    switch (primaryTab) {
      case 'game_rules':
        return { tabs: gameRulesTabs, active: gameRulesSubTab, setActive: setGameRulesSubTab }
      case 'advanced':
        return { tabs: advancedTabs, active: advancedSubTab, setActive: setAdvancedSubTab }
      case 'mod_settings':
        return { tabs: modSettingsTabs, active: modSettingsSubTab, setActive: setModSettingsSubTab }
      default:
        return { tabs: [], active: null, setActive: null }
    }
  }

  const { tabs, active, setActive } = getTabs()

  if (!setActive) return null

  return (
    <div className="border-b border-ark-cyan/20 bg-ark-dark/50 px-8 py-2">
      <div className="flex gap-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActive(tab.id)}
            className={`px-4 py-1 text-xs font-semibold tracking-widest uppercase transition ${
              active === tab.id ? 'text-white bg-ark-secondary/50' : 'text-ark-cyan/60 hover:text-ark-cyan/80'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>
    </div>
  )
}
