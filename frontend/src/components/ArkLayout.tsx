import React from 'react'
import { useConfigStore, type ConfigStore } from '../stores/configStore'
import { useShallow } from 'zustand/react/shallow'

interface ArkLayoutProps {
  children: React.ReactNode
}

export default function ArkLayout({ children }: ArkLayoutProps) {
  const config = useConfigStore(useShallow((s: ConfigStore) => s.config))
  const serverName = config?.identification?.session_name || 'ARK SERVER'

  return (
    <div className="min-h-screen relative bg-ark-dark">
      <div className="fixed inset-0 z-0 ark-bg" />

      {/* ARK logo — sits between header strip and nav, centered */}
      <div className="fixed left-1/2 transform -translate-x-1/2 z-20 pointer-events-none" style={{ top: '4px' }}>
        <img
          src="/assets/ark-logo.png"
          alt="ARK Survival Ascended"
          className="w-28 h-auto drop-shadow-2xl"
          style={{ filter: 'drop-shadow(0 0 16px rgba(0,212,255,1)) drop-shadow(0 0 32px rgba(157,78,221,0.7))' }}
        />
      </div>

      {/* Header strip — server name left, ESC right */}
      <div className="fixed top-0 left-0 right-0 z-30 flex items-center justify-between px-5 py-1 pointer-events-none">
        <span className="text-ark-cyan/80 text-xs font-bold tracking-widest uppercase italic pointer-events-auto"
              style={{ fontStyle: 'italic', letterSpacing: '0.12em' }}>
          {serverName}
        </span>
        <button
          className="pointer-events-auto ark-action-btn text-[10px] px-3 py-0.5"
          title="Exit"
          onClick={async () => {
            try {
              const { getCurrentWindow } = await import('@tauri-apps/api/window')
              await getCurrentWindow().close()
            } catch {
              window.close()
            }
          }}
        >
          ESC
        </button>
      </div>

      {/* Content — pt-8 to clear header strip; pb-16 for ActionBar */}
      <div className="relative z-10 pt-8">{children}</div>
    </div>
  )
}
