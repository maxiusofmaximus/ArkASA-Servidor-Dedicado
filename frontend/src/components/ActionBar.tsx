import React, { useState } from 'react'

interface ActionBarProps {
  onSave?: () => Promise<void> | void
  onReset?: () => void
  onBack?: () => void
  onChooseDifficulty?: () => void
  onStartServer?: () => void
  onStopServer?: () => void
  isSaving?: boolean
  isServerRunning?: boolean
  isServerStarting?: boolean
  isServerStopping?: boolean
  variant?: 'default' | 'mod_settings'
}

export default function ActionBar({
  onSave,
  onReset,
  onBack,
  onChooseDifficulty,
  onStartServer,
  onStopServer,
  isSaving = false,
  isServerRunning = false,
  isServerStarting = false,
  isServerStopping = false,
  variant = 'default',
}: ActionBarProps) {
  const [saveOk, setSaveOk] = useState(false)

  const handleSave = async () => {
    if (!onSave) return
    try {
      await onSave()
      setSaveOk(true)
      setTimeout(() => setSaveOk(false), 1800)
    } catch { /* error shown elsewhere */ }
  }

  // Start button: disabled while starting OR already running
  const startDisabled = isServerStarting || isServerRunning
  // Stop button: disabled while stopping
  const stopDisabled = isServerStopping

  return (
    <div className="fixed bottom-0 left-0 right-0 z-20 ark-panel border-t border-ark-cyan/40 px-6 py-3 flex items-center justify-between">
      {/* Left: action buttons */}
      <div className="flex items-center gap-2">
        <button onClick={onBack} className="ark-action-btn">BACK</button>

        {variant === 'mod_settings' ? (
          <>
            <button className="ark-action-btn">UPDATE</button>
            <button className="ark-action-btn">GET MODS</button>
            <button className="ark-action-btn">DEACTIVATE MOD</button>
            <button className="ark-action-btn">MOD INFO</button>
          </>
        ) : (
          <>
            {onReset && (
              <button onClick={onReset} disabled={isSaving} className="ark-action-btn disabled:opacity-40">
                RESTORE DEFAULTS
              </button>
            )}
            <button onClick={onChooseDifficulty} className="ark-action-btn">CHOOSE DIFFICULTY</button>
            {onSave && (
              <button onClick={handleSave} disabled={isSaving} className="ark-action-btn disabled:opacity-40"
                style={saveOk ? { color: 'rgba(74,222,128,0.9)', outlineColor: 'rgba(74,222,128,0.4)' } : undefined}>
                {isSaving ? 'SAVING...' : saveOk ? 'GUARDADO ✓' : 'SAVE SETTINGS'}
              </button>
            )}
          </>
        )}
      </div>

      {/* Right: server control */}
      <div className="flex items-center gap-3">
        <span className="text-ark-cyan/45 text-[9px] font-bold tracking-widest uppercase italic">
          - DEDICATED SERVER -
        </span>

        {isServerRunning || isServerStopping ? (
          /* STOP SERVER button — red when server is running */
          <button
            onClick={onStopServer}
            disabled={stopDisabled}
            className="ark-action-btn px-5 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
            style={{
              background: stopDisabled ? 'rgba(239,68,68,0.07)' : 'rgba(239,68,68,0.15)',
              color: 'rgba(239,68,68,0.9)',
              outlineColor: 'rgba(239,68,68,0.6)',
              boxShadow: stopDisabled ? 'none' : '0 0 12px rgba(239,68,68,0.3)',
            }}
          >
            {isServerStopping ? '■ DETENIENDO...' : '■ STOP SERVER'}
          </button>
        ) : (
          /* DEDICATED SERVER / START button — amber/gold */
          <button
            onClick={onStartServer}
            disabled={startDisabled}
            className="ark-action-btn ark-action-btn-amber-active px-5 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
            title="Save settings then launch the ARK dedicated server"
          >
            {isServerStarting ? '▶ INICIANDO...' : '▶ DEDICATED SERVER'}
          </button>
        )}
      </div>
    </div>
  )
}
