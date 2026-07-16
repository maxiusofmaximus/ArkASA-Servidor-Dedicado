import React, { useEffect, useRef, useState } from 'react'
import { useConfigStore, type ConfigStore } from '../stores/configStore'
import { useUiStore } from '../stores/uiStore'
import { useShallow } from 'zustand/react/shallow'
import ArkMenuScene from './ArkMenuScene'

interface ArkLayoutProps {
  children: React.ReactNode
}

export default function ArkLayout({ children }: ArkLayoutProps) {
  const config = useConfigStore(useShallow((s: ConfigStore) => s.config))
  const serverNameVisible = useUiStore((s) => s.serverNameVisible)
  const serverName = config?.identification?.session_name || 'ARK SERVER'
  const audioRef = useRef<HTMLAudioElement>(null)
  const [musicEnabled, setMusicEnabled] = useState(false)

  useEffect(() => {
    const storedPreference = window.localStorage.getItem('ark-menu-music')
    setMusicEnabled(storedPreference === 'on')
  }, [])

  useEffect(() => {
    const audio = audioRef.current
    if (!audio) return

    audio.muted = !musicEnabled
    audio.volume = 0.35

    if (musicEnabled) {
      void audio.play().catch(() => {
        // The next click on the control will retry playback inside a user gesture.
      })
    } else {
      audio.pause()
    }
  }, [musicEnabled])

  const toggleMusic = () => {
    const nextEnabled = !musicEnabled
    setMusicEnabled(nextEnabled)
    window.localStorage.setItem('ark-menu-music', nextEnabled ? 'on' : 'off')
  }

  return (
    <div className="min-h-screen relative bg-ark-dark">
      <div className="fixed inset-0 z-0 ark-bg" aria-hidden="true">
        <ArkMenuScene fallbackImage="/assets/ark-background.png" />
        <div className="ark-bg-vignette" />
        <div className="ark-bg-grain" />
      </div>

      <audio ref={audioRef} src="/assets/ark-menu-music.m4a" loop preload="metadata" />

      {/* ARK logo — sits between header strip and nav, centered */}
      <div className="fixed left-1/2 transform -translate-x-1/2 z-20 pointer-events-none" style={{ top: '4px' }}>
        <img
          src="/assets/ark-logo.png"
          alt="ARK Survival Ascended"
          className="w-28 h-auto drop-shadow-2xl"
          style={{ filter: 'drop-shadow(0 0 16px rgba(0,212,255,1)) drop-shadow(0 0 32px rgba(157,78,221,0.7))' }}
        />
      </div>

      {/* Header strip — server name left */}
      <div className="fixed top-0 left-0 right-0 z-30 flex items-center px-5 py-1 pointer-events-none">
        <span
          className="text-ark-cyan/80 text-xs font-bold tracking-widest uppercase italic pointer-events-auto select-none"
          style={{ fontStyle: 'italic', letterSpacing: '0.12em', filter: serverNameVisible ? 'none' : 'blur(6px)' }}
        >
          {serverName}
        </span>
      </div>

      <button
        type="button"
        className={`ark-audio-toggle fixed top-3 right-5 z-30 ${musicEnabled ? 'is-enabled' : ''}`}
        onClick={toggleMusic}
        aria-pressed={musicEnabled}
        aria-label={musicEnabled ? 'Desactivar música del menú' : 'Activar música del menú'}
        title={musicEnabled ? 'Desactivar música del menú' : 'Activar música del menú'}
      >
        <span className="ark-audio-toggle-icon" aria-hidden="true">{musicEnabled ? '♫' : '♪̸'}</span>
        {musicEnabled ? 'MUSIC ON' : 'MUSIC OFF'}
      </button>

      {/* Content — pt-8 to clear header strip; pb-16 for ActionBar */}
      <div className="relative z-10 pt-8">{children}</div>
    </div>
  )
}
