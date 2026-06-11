import React from 'react'

interface SettingsPanelProps {
  children: React.ReactNode
  title?: string
}

export default function SettingsPanel({ children, title }: SettingsPanelProps) {
  return (
    <div className="max-w-4xl mx-auto px-8 py-6">
      {title && <h2 className="text-xl font-bold text-ark-cyan mb-4">{title}</h2>}
      <div className="ark-panel rounded-lg overflow-hidden">
        <div className="ark-scroll overflow-y-auto max-h-[calc(100vh-300px)]">{children}</div>
      </div>
    </div>
  )
}
