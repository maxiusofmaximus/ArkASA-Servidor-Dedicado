import React, { useState, useEffect } from 'react'
import NumericInput from './NumericInput'
import ToggleButton from './ToggleButton'
import TruncatedText from './TruncatedText'
import Tooltip from './Tooltip'
import { useI18n } from '../i18n/useI18n'
import { useUiStore } from '../stores/uiStore'

interface SettingRowProps {
  label: string
  value: string | number | boolean
  type: 'number' | 'boolean' | 'text' | 'secret' | 'copyable'
  onChange: (value: any) => void
  step?: number
  min?: number
  max?: number
  disabled?: boolean
  testId?: string
  tooltip?: string
}

const INPUT_W = 'w-32'

const ActionBtn = ({ onClick, active, activeColor, children, title }: {
  onClick: () => void
  active: boolean
  activeColor: string
  children: React.ReactNode
  title: string
}) => (
  <button
    onClick={onClick}
    className="text-[10px] w-5 text-center py-0.5 rounded transition-colors"
    style={{ color: active ? activeColor : 'rgba(255,255,255,0.15)' }}
    title={title}
  >
    {children}
  </button>
)

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
  const { t, tip, tk } = useI18n()
  const translatedLabel = t(label)
  const description = tooltip ?? tip(label)

  const [visible, setVisible] = useState(false)
  const [copied, setCopied] = useState(false)
  const setServerNameVisible = useUiStore((s) => s.setServerNameVisible)
  const isServerName = label === 'Server Name'

  useEffect(() => {
    if (isServerName) setServerNameVisible(visible)
  }, [visible, isServerName, setServerNameVisible])

  const handleCopy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch { /* ignore */ }
  }

  const hasActions = type === 'secret' || type === 'copyable'

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

      <div className="flex items-center shrink-0">
        {/* Input area — same width for all types */}
        {type === 'boolean' && typeof value === 'boolean' && (
          <div className={`${INPUT_W} flex justify-end`}>
            <ToggleButton value={value} onChange={onChange} disabled={disabled} />
          </div>
        )}

        {type === 'number' && typeof value === 'number' && (
          <div className={`${INPUT_W} flex justify-end`}>
            <NumericInput
              value={value}
              onChange={onChange}
              step={step}
              min={min}
              max={max}
              disabled={disabled}
            />
          </div>
        )}

        {type === 'text' && typeof value === 'string' && (
          <input
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            disabled={disabled}
            className={`bg-transparent border-b border-ark-cyan text-ark-cyan text-right ${INPUT_W} focus:outline-none disabled:opacity-50`}
          />
        )}

        {type === 'secret' && typeof value === 'string' && (
          <input
            type={visible ? 'text' : 'password'}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            disabled={disabled}
            className={`bg-transparent border-b border-ark-cyan text-ark-cyan text-right ${INPUT_W} focus:outline-none disabled:opacity-50 font-mono`}
          />
        )}

        {type === 'copyable' && typeof value === 'string' && (
          <input
            type={visible ? 'text' : 'password'}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            disabled={disabled}
            className={`bg-transparent border-b border-ark-cyan text-ark-cyan text-right ${INPUT_W} focus:outline-none disabled:opacity-50`}
          />
        )}

        {/* Actions zone — separated by vertical line */}
        {hasActions && (
          <div
            className="flex items-center gap-0.5 ml-3 pl-3"
            style={{ borderLeft: '1px solid rgba(0,212,255,0.12)' }}
          >
            {(type === 'secret' || type === 'copyable') && (
              <ActionBtn
                onClick={() => setVisible(!visible)}
                active={visible}
                activeColor="rgba(0,212,255,0.9)"
                title={visible ? tk('hide', 'Hide') : tk('show', 'Show')}
              >
                {visible ? '◉' : '○'}
              </ActionBtn>
            )}
            <ActionBtn
              onClick={() => handleCopy(value as string)}
              active={copied}
              activeColor="rgba(74,222,128,0.8)"
              title={tk('copy', 'Copy')}
            >
              {copied ? '✓' : '⧉'}
            </ActionBtn>
          </div>
        )}
      </div>
    </div>
  )
})

export default SettingRow
