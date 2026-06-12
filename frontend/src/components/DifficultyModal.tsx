import React from 'react'

interface Preset {
  label: string
  value: number
  maxLevel: number
  description: string
}

const PRESETS: Preset[] = [
  { label: 'CASUAL',   value: 1.0,  maxLevel: 30,  description: 'Dinos nivel max ~30. Para recién comenzar.' },
  { label: 'NORMAL',   value: 4.0,  maxLevel: 120, description: 'Dinos nivel max ~120. Oficial estándar.' },
  { label: 'HARD',     value: 5.0,  maxLevel: 150, description: 'Dinos nivel max ~150. Oficial difícil.' },
  { label: 'EXTREME',  value: 8.0,  maxLevel: 240, description: 'Dinos nivel max ~240. Alta dificultad.' },
  { label: 'LEGENDARY',value: 10.0, maxLevel: 300, description: 'Dinos nivel max ~300. Máxima dificultad.' },
]

interface DifficultyModalProps {
  currentValue: number
  onSelect: (value: number) => void
  onClose: () => void
}

export default function DifficultyModal({ currentValue, onSelect, onClose }: DifficultyModalProps) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: 'rgba(0,0,0,0.75)' }}
      onClick={e => { if (e.target === e.currentTarget) onClose() }}
    >
      <div className="ark-panel rounded-lg w-full max-w-md mx-4 overflow-hidden">
        {/* Header */}
        <div className="px-5 py-3 border-b border-ark-cyan/30 flex items-center justify-between">
          <span className="text-ark-cyan text-xs font-bold tracking-widest uppercase">Choose Difficulty</span>
          <button onClick={onClose} className="text-ark-cyan/50 hover:text-ark-cyan/90 text-sm transition">✕</button>
        </div>

        {/* Presets */}
        <div className="p-4 space-y-2">
          {PRESETS.map(p => {
            const isActive = Math.abs(currentValue - p.value) < 0.001
            return (
              <button
                key={p.label}
                onClick={() => { onSelect(p.value); onClose() }}
                className={`w-full flex items-center gap-4 px-4 py-3 rounded border transition text-left ${
                  isActive
                    ? 'border-ark-cyan/60 bg-ark-cyan/10'
                    : 'border-ark-cyan/15 hover:border-ark-cyan/40 hover:bg-ark-secondary/20'
                }`}
              >
                <div className="flex-shrink-0 w-24">
                  <div className={`text-xs font-bold tracking-widest ${isActive ? 'text-ark-cyan' : 'text-ark-cyan/70'}`}>
                    {p.label}
                  </div>
                  <div className="text-ark-cyan/40 text-[10px] font-mono mt-0.5">×{p.value.toFixed(1)}</div>
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-ark-cyan/55 text-xs leading-relaxed">{p.description}</div>
                </div>
                <div className="flex-shrink-0 text-right">
                  <div className="text-ark-cyan/60 text-[10px] font-mono">lvl {p.maxLevel}</div>
                  {isActive && <div className="text-ark-cyan text-[9px] font-bold tracking-widest mt-0.5">✓ ACTUAL</div>}
                </div>
              </button>
            )
          })}

          {/* Custom value hint */}
          <p className="text-ark-cyan/25 text-[10px] pt-1 text-center">
            O ajusta <span className="font-mono text-ark-cyan/40">Override Official Difficulty</span> manualmente en Game Rules → World.
          </p>
        </div>
      </div>
    </div>
  )
}
