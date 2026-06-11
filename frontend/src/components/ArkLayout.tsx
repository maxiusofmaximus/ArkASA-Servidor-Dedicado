import React from 'react'

interface ArkLayoutProps {
  children: React.ReactNode
}

export default function ArkLayout({ children }: ArkLayoutProps) {
  return (
    <div className="min-h-screen relative bg-ark-dark">
      {/* Space background with image or fallback gradient */}
      <div className="fixed inset-0 z-0 ark-bg ark-bg-fallback" />

      {/* Content on top of background */}
      <div className="relative z-10">{children}</div>
    </div>
  )
}
