import React from 'react'

interface ArkLayoutProps {
  children: React.ReactNode
}

export default function ArkLayout({ children }: ArkLayoutProps) {
  return (
    <div className="min-h-screen relative bg-ark-dark">
      {/* Space background with image or fallback gradient */}
      <div className="fixed inset-0 z-0 ark-bg ark-bg-fallback" />

      {/* ARK Logo - top center, decorative */}
      <div className="fixed top-0 left-1/2 transform -translate-x-1/2 -translate-y-1/4 z-0 pointer-events-none">
        <img
          src="/assets/ark-logo.png"
          alt="ARK Survival Ascended"
          className="w-64 h-auto opacity-30 drop-shadow-2xl"
          style={{
            filter: 'drop-shadow(0 0 40px rgba(0, 212, 255, 0.3)) drop-shadow(0 0 80px rgba(157, 78, 221, 0.2))'
          }}
        />
      </div>

      {/* Content on top of background */}
      <div className="relative z-10">{children}</div>
    </div>
  )
}
