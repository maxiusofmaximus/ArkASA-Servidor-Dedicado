import React from 'react'

interface ToggleButtonProps {
  value: boolean
  onChange: (value: boolean) => void
  disabled?: boolean
}

export default function ToggleButton({ value, onChange, disabled = false }: ToggleButtonProps) {
  return (
    <div className="flex gap-2">
      <button
        onClick={() => !disabled && onChange(true)}
        className={`px-4 py-1 text-sm font-semibold transition ${
          value
            ? 'bg-ark-cyan text-ark-dark'
            : 'border border-ark-cyan/30 text-ark-cyan/50 hover:border-ark-cyan/60'
        } ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
      >
        ON
      </button>
      <button
        onClick={() => !disabled && onChange(false)}
        className={`px-4 py-1 text-sm font-semibold transition ${
          !value
            ? 'bg-ark-cyan text-ark-dark'
            : 'border border-ark-cyan/30 text-ark-cyan/50 hover:border-ark-cyan/60'
        } ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
      >
        OFF
      </button>
    </div>
  )
}
