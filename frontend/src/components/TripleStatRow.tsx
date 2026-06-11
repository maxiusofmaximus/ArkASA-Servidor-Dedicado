import React from 'react'
import NumericInput from './NumericInput'

interface TripleStatRowProps {
  label: string
  perLevel: number
  addPerLevel: number
  affinity: number
  onPerLevelChange: (value: number) => void
  onAddPerLevelChange: (value: number) => void
  onAffinityChange: (value: number) => void
}

const TripleStatRow = React.memo(function TripleStatRow({
  label,
  perLevel,
  addPerLevel,
  affinity,
  onPerLevelChange,
  onAddPerLevelChange,
  onAffinityChange,
}: TripleStatRowProps) {
  return (
    <div className="flex items-center justify-between py-2 px-4 border-b border-ark-cyan/10 hover:bg-ark-secondary/20 transition">
      <span className="text-ark-cyan/80 text-sm tracking-wide w-32">{label}</span>

      <div className="flex gap-8 flex-1 justify-end">
        <div className="flex flex-col items-end">
          <span className="text-ark-cyan/50 text-xs mb-1">Per Level</span>
          <NumericInput value={perLevel} onChange={onPerLevelChange} step={0.01} />
        </div>

        <div className="flex flex-col items-end">
          <span className="text-ark-cyan/50 text-xs mb-1">Add Per Level</span>
          <NumericInput value={addPerLevel} onChange={onAddPerLevelChange} step={0.01} />
        </div>

        <div className="flex flex-col items-end">
          <span className="text-ark-cyan/50 text-xs mb-1">Affinity</span>
          <NumericInput value={affinity} onChange={onAffinityChange} step={0.01} />
        </div>
      </div>
    </div>
  )
})

export default TripleStatRow
