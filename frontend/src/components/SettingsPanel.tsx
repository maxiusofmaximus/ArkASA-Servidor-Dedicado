import React from 'react'

interface SettingsPanelProps {
  children: React.ReactNode
  title?: string  // kept for backwards compat but not rendered
  wide?: boolean  // use max-w-4xl instead of default max-w-lg
}

export default function SettingsPanel({ children, wide }: SettingsPanelProps) {
  return (
    <div className={`${wide ? 'max-w-4xl' : 'max-w-lg'} mx-auto px-8 py-6`}>
      <div className="ark-panel rounded-lg overflow-hidden">
        <div className="ark-scroll overflow-y-auto max-h-[calc(100vh-300px)]">{children}</div>
      </div>
    </div>
  )
}
