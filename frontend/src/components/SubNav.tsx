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

function subTabClass(active: boolean): string {
  return active
    ? 'px-5 py-1.5 text-xs font-bold tracking-widest uppercase text-white bg-ark-secondary border border-ark-cyan/60 transition'
    : 'px-5 py-1.5 text-xs font-bold tracking-widest uppercase text-ark-cyan/55 hover:text-ark-cyan/80 transition'
}

function SubNavBar({ children }: { children: React.ReactNode }) {
  return (
    <div className="bg-ark-dark/80 border-b border-ark-cyan/20 px-8 py-2">
      <div className="flex gap-1">{children}</div>
    </div>
  )
}

export default function SubNav({ primaryTab }: SubNavProps) {
  const { gameRulesSubTab, setGameRulesSubTab, advancedSubTab, setAdvancedSubTab, modSettingsSubTab, setModSettingsSubTab } = useUiStore()

  if (primaryTab === 'game_rules') {
    return (
      <SubNavBar>
        {gameRulesTabs.map((tab) => (
          <button key={tab.id} onClick={() => setGameRulesSubTab(tab.id)} className={subTabClass(gameRulesSubTab === tab.id)}>
            {tab.label}
          </button>
        ))}
      </SubNavBar>
    )
  }

  if (primaryTab === 'advanced') {
    return (
      <SubNavBar>
        {advancedTabs.map((tab) => (
          <button key={tab.id} onClick={() => setAdvancedSubTab(tab.id)} className={subTabClass(advancedSubTab === tab.id)}>
            {tab.label}
          </button>
        ))}
      </SubNavBar>
    )
  }

  if (primaryTab === 'mod_settings') {
    return (
      <SubNavBar>
        {modSettingsTabs.map((tab) => (
          <button key={tab.id} onClick={() => setModSettingsSubTab(tab.id)} className={subTabClass(modSettingsSubTab === tab.id)}>
            {tab.label}
          </button>
        ))}
      </SubNavBar>
    )
  }

  return null
}
