import React from 'react'
import NumericInput from './NumericInput'
import ToggleButton from './ToggleButton'
import TruncatedText from './TruncatedText'
import Tooltip from './Tooltip'
import { useI18n } from '../i18n/useI18n'

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
  tooltip?: string  // explicit override; auto-looked-up from i18n/tooltips if absent
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
  tooltip,
}: SettingRowProps) {
  const { t, tip } = useI18n()
  const translatedLabel = t(label)
  const description = tooltip ?? tip(label)

  const labelNode = (
    <div className="flex items-center gap-1.5 min-w-0 flex-1 pr-2">
      <TruncatedText
        text={translatedLabel}
        className="text-ark-cyan/80 text-sm tracking-wide"
      />
      {description && (
        <span
          className="flex-shrink-0 text-[9px] text-ark-cyan/30 hover:text-ark-cyan/70 transition-colors cursor-default select-none"
          style={{ lineHeight: 1 }}
        >
          ⓘ
        </span>
      )}
    </div>
  )

  return (
    <div
      data-testid={testId}
      className="flex items-center justify-between py-2 px-4 border-b border-ark-cyan/10 hover:bg-ark-secondary/20 transition"
    >
      {description ? (
        <Tooltip content={description}>
          {labelNode}
        </Tooltip>
      ) : (
        labelNode
      )}

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
