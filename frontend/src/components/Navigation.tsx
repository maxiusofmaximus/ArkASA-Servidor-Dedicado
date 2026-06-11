import type { PrimaryTab } from '../types'

interface NavigationProps {
  activeTab: PrimaryTab
  onTabChange: (tab: PrimaryTab) => void
}

export default function Navigation({ activeTab, onTabChange }: NavigationProps) {
  const tabs: { id: PrimaryTab; label: string }[] = [
    { id: 'arks', label: 'ARKS' },
    { id: 'mod_settings', label: 'MOD SETTINGS' },
    { id: 'game_rules', label: 'GAME RULES' },
    { id: 'advanced', label: 'ADVANCED' },
    { id: 'engrams', label: 'ENGRAMS' },
  ]

  return (
    <nav className="bg-ark-secondary border-b-2 border-ark-cyan shadow-ark">
      <div className="px-8 py-4">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-3xl font-bold bg-gradient-to-r from-ark-cyan to-ark-purple bg-clip-text text-transparent">
            ARK ASA Configuration Manager
          </h1>
          <div className="text-sm text-ark-cyan">v2.0.0</div>
        </div>

        <div className="flex gap-2">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`px-6 py-3 font-semibold transition-all duration-200 ${
                activeTab === tab.id
                  ? 'bg-ark-cyan text-ark-dark shadow-ark'
                  : 'text-ark-cyan hover:bg-ark-cyan/20 border-b-2 border-transparent hover:border-ark-cyan'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>
    </nav>
  )
}
