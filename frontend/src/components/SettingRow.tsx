import React from 'react'
import NumericInput from './NumericInput'
import ToggleButton from './ToggleButton'

interface SettingRowProps {
  label: string
  value: string | number | boolean
  type: 'number' | 'boolean' | 'text'
  onChange: (value: any) => void
  step?: number
  min?: number
  max?: number
  disabled?: boolean
  testId?: string
}

const SettingRow = React.memo(function SettingRow({
  label,
  value,
  type,
  onChange,
  step,
  min,
  max,
  disabled = false,
  testId,
}: SettingRowProps) {
  return (
    <div
      data-testid={testId}
      className="flex items-center justify-between py-2 px-4 border-b border-ark-cyan/10 hover:bg-ark-secondary/20 transition"
    >
      <span className="text-ark-cyan/80 text-sm tracking-wide">{label}</span>

      {type === 'boolean' && typeof value === 'boolean' && (
        <ToggleButton value={value} onChange={onChange} disabled={disabled} />
      )}

      {type === 'number' && typeof value === 'number' && (
        <NumericInput
          value={value}
          onChange={onChange}
          step={step}
          min={min}
          max={max}
          disabled={disabled}
        />
      )}

      {type === 'text' && typeof value === 'string' && (
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          disabled={disabled}
          className="bg-transparent border-b border-ark-cyan text-ark-cyan text-right w-40 focus:outline-none disabled:opacity-50"
        />
      )}
    </div>
  )
})

export default SettingRow
