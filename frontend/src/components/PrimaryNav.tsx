import React from 'react'
import type { PrimaryTab } from '../types'

interface PrimaryNavProps {
  activeTab: PrimaryTab
  onTabChange: (tab: PrimaryTab) => void
}

const tabs: { id: PrimaryTab; label: string }[] = [
  { id: 'arks', label: 'ARKS' },
  { id: 'mod_settings', label: 'MOD SETTINGS' },
  { id: 'game_rules', label: 'GAME RULES' },
  { id: 'advanced', label: 'ADVANCED' },
  { id: 'engrams', label: 'ENGRAMS' },
]

export default function PrimaryNav({ activeTab, onTabChange }: PrimaryNavProps) {
  return (
    <nav className="ark-panel border-b border-ark-cyan/40 sticky top-0 z-20">
      <div className="flex gap-1 px-8 py-4">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={`px-6 py-2 text-sm font-semibold tracking-widest uppercase transition ${
              activeTab === tab.id
                ? 'text-ark-cyan border-b-2 border-ark-cyan'
                : 'text-ark-cyan/50 hover:text-ark-cyan/80'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>
    </nav>
  )
}
