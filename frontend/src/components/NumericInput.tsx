import React, { useState } from 'react'

interface NumericInputProps {
  value: number
  onChange: (value: number) => void
  step?: number
  min?: number
  max?: number
  disabled?: boolean
}

export default function NumericInput({
  value,
  onChange,
  step = 0.1,
  min,
  max,
  disabled = false,
}: NumericInputProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [tempValue, setTempValue] = useState(value.toString())

  const handleBlur = () => {
    const num = parseFloat(tempValue)
    if (!isNaN(num)) {
      let finalValue = num
      if (min !== undefined && finalValue < min) finalValue = min
      if (max !== undefined && finalValue > max) finalValue = max
      onChange(finalValue)
    }
    setIsEditing(false)
    setTempValue(value.toString())
  }

  if (isEditing) {
    return (
      <input
        autoFocus
        type="number"
        value={tempValue}
        onChange={(e) => setTempValue(e.target.value)}
        onBlur={handleBlur}
        onKeyDown={(e) => {
          if (e.key === 'Enter') handleBlur()
          if (e.key === 'Escape') {
            setIsEditing(false)
            setTempValue(value.toString())
          }
        }}
        step={step}
        min={min}
        max={max}
        disabled={disabled}
        className="bg-transparent border-b border-ark-cyan text-ark-cyan text-right w-24 focus:outline-none"
      />
    )
  }

  return (
    <div
      onClick={() => !disabled && setIsEditing(true)}
      className={`text-right text-ark-cyan cursor-pointer ${disabled ? 'opacity-50' : 'hover:text-ark-cyan/80'}`}
    >
      {value.toFixed(step < 1 ? 2 : 0)}
    </div>
  )
}
