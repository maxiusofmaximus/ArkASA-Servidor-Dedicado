import React from 'react'

interface ActionBarProps {
  onSave?: () => void
  onReset?: () => void
  isSaving?: boolean
}

export default function ActionBar({ onSave, onReset, isSaving = false }: ActionBarProps) {
  return (
    <div className="fixed bottom-0 left-0 right-0 ark-panel border-t border-ark-cyan/40 px-8 py-4 flex justify-end gap-4 z-20">
      {onReset && (
        <button
          onClick={onReset}
          disabled={isSaving}
          className="px-6 py-2 border border-ark-cyan/50 text-ark-cyan/70 hover:border-ark-cyan hover:text-ark-cyan rounded transition disabled:opacity-50"
        >
          RESTORE DEFAULTS
        </button>
      )}

      {onSave && (
        <button
          onClick={onSave}
          disabled={isSaving}
          className="px-6 py-2 bg-ark-cyan text-ark-dark font-bold rounded hover:bg-ark-cyan/90 transition disabled:opacity-50 uppercase tracking-wider"
        >
          {isSaving ? 'SAVING...' : 'SAVE SETTINGS'}
        </button>
      )}
    </div>
  )
}
