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

const leftTabs  = tabs.slice(0, 2)  // ARKS, MOD SETTINGS
const rightTabs = tabs.slice(2)     // GAME RULES, ADVANCED, ENGRAMS

export default function PrimaryNav({ activeTab, onTabChange }: PrimaryNavProps) {
  const tabClass = (id: PrimaryTab) =>
    `px-5 py-2 text-sm tracking-widest uppercase transition ${
      activeTab === id
        ? 'font-extrabold text-white underline underline-offset-4 decoration-ark-cyan decoration-2 drop-shadow-[0_0_8px_rgba(0,212,255,0.8)]'
        : 'font-semibold text-ark-cyan/55 hover:text-ark-cyan/80'
    }`

  return (
    <nav className="ark-panel border-b border-ark-cyan/40 sticky top-0 z-20">
      {/* pt-9 creates space for the floating ARK logo (w-28) above */}
      <div className="flex items-center justify-center pt-9 pb-2 px-6">
        {/* Left tabs */}
        <div className="flex gap-1 flex-1 justify-end pr-12">
          {leftTabs.map((tab) => (
            <button key={tab.id} onClick={() => onTabChange(tab.id)} className={tabClass(tab.id)}>
              {tab.label}
            </button>
          ))}
        </div>

        {/* Center spacer matching the w-28 logo */}
        <div className="w-28 flex-shrink-0" />

        {/* Right tabs */}
        <div className="flex gap-1 flex-1 justify-start pl-12">
          {rightTabs.map((tab) => (
            <button key={tab.id} onClick={() => onTabChange(tab.id)} className={tabClass(tab.id)}>
              {tab.label}
            </button>
          ))}
        </div>
      </div>
    </nav>
  )
}
