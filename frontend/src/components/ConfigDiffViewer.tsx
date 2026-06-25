import React, { useMemo, useRef } from 'react'
import type { DiffEntry } from '../utils/configDiff'
import { formatValue } from '../utils/configDiff'
import { useI18n } from '../i18n/useI18n'

interface Props {
  title?: string
  entries: DiffEntry[]
  onApply: () => void
  onCancel: () => void
  applyLabel?: string
}

function groupByCategory(entries: DiffEntry[]): Array<{ category: string; items: DiffEntry[] }> {
  const map = new Map<string, DiffEntry[]>()
  for (const e of entries) {
    if (!map.has(e.category)) map.set(e.category, [])
    map.get(e.category)!.push(e)
  }
  return Array.from(map.entries()).map(([category, items]) => ({ category, items }))
}

function DiffRow({ entry }: { entry: DiffEntry }) {
  const oldText = formatValue(entry.oldValue, entry.sensitive)
  const newText = formatValue(entry.newValue, entry.sensitive)

  return (
    <div className="grid grid-cols-[1fr_220px_220px] text-[11px] border-b border-white/5 hover:bg-white/3 group">
      <div className="px-3 py-1.5 text-ark-cyan/70 group-hover:text-ark-cyan/90 font-medium truncate">
        {entry.label}
      </div>
      <div
        className="px-3 py-1.5 font-mono truncate"
        style={{
          background: 'rgba(239,68,68,0.07)',
          color: 'rgba(239,68,68,0.85)',
          borderLeft: '1px solid rgba(239,68,68,0.15)',
        }}
        title={oldText}
      >
        <span className="opacity-50 mr-1">–</span>{oldText}
      </div>
      <div
        className="px-3 py-1.5 font-mono truncate"
        style={{
          background: 'rgba(74,222,128,0.07)',
          color: 'rgba(74,222,128,0.85)',
          borderLeft: '1px solid rgba(74,222,128,0.15)',
        }}
        title={newText}
      >
        <span className="opacity-50 mr-1">+</span>{newText}
      </div>
    </div>
  )
}

export default function ConfigDiffViewer({ title, entries, onApply, onCancel, applyLabel }: Props) {
  const groups = useMemo(() => groupByCategory(entries), [entries])
  const scrollRef = useRef<HTMLDivElement>(null)
  const { tk } = useI18n()

  const resolvedApplyLabel = applyLabel ?? tk('apply_changes', 'Apply Changes')
  const resolvedTitle = title ?? tk('config_changes_title', 'Configuration Changes')
  const fieldWord = entries.length === 1 ? tk('save_count_singular', 'change') : tk('save_count_plural', 'changes')

  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center"
      style={{ background: 'rgba(0,0,0,0.75)', backdropFilter: 'blur(4px)' }}
    >
      <div
        className="ark-panel flex flex-col"
        style={{
          width: '780px',
          maxWidth: '95vw',
          maxHeight: '82vh',
          border: '1px solid rgba(0,200,255,0.3)',
          boxShadow: '0 0 40px rgba(0,200,255,0.12)',
          borderRadius: '4px',
        }}
      >
        {/* Header */}
        <div
          className="flex items-center justify-between px-5 py-3 flex-shrink-0"
          style={{ borderBottom: '1px solid rgba(0,200,255,0.2)' }}
        >
          <div>
            <span className="text-ark-cyan font-bold tracking-wider text-sm uppercase">
              {resolvedTitle}
            </span>
            <span
              className="ml-3 text-[10px] px-2 py-0.5 rounded font-mono"
              style={{
                background: 'rgba(0,200,255,0.12)',
                color: 'rgba(0,200,255,0.7)',
                border: '1px solid rgba(0,200,255,0.25)',
              }}
            >
              {entries.length} {fieldWord}
            </span>
          </div>
          <button
            onClick={onCancel}
            className="text-ark-cyan/40 hover:text-ark-cyan/80 text-lg leading-none"
          >
            ×
          </button>
        </div>

        {/* Column headers */}
        <div
          className="grid grid-cols-[1fr_220px_220px] text-[10px] tracking-widest uppercase flex-shrink-0"
          style={{ borderBottom: '1px solid rgba(0,200,255,0.15)' }}
        >
          <div className="px-3 py-2 text-ark-cyan/40">{tk('field_col', 'Field')}</div>
          <div
            className="px-3 py-2 text-red-400/60"
            style={{ borderLeft: '1px solid rgba(0,200,255,0.15)' }}
          >
            {tk('before_col', 'Before')}
          </div>
          <div
            className="px-3 py-2 text-green-400/60"
            style={{ borderLeft: '1px solid rgba(0,200,255,0.15)' }}
          >
            {tk('after_col', 'After')}
          </div>
        </div>

        {/* Diff rows */}
        <div ref={scrollRef} className="overflow-y-auto flex-1 min-h-0">
          {groups.map(({ category, items }) => (
            <div key={category}>
              <div
                className="px-3 py-1 text-[9px] tracking-[0.2em] uppercase font-bold sticky top-0"
                style={{
                  background: 'rgba(0,12,24,0.95)',
                  color: 'rgba(0,200,255,0.4)',
                  borderBottom: '1px solid rgba(0,200,255,0.08)',
                }}
              >
                {category}
              </div>
              {items.map((e) => (
                <DiffRow key={e.path} entry={e} />
              ))}
            </div>
          ))}
        </div>

        {/* Footer */}
        <div
          className="flex items-center justify-end gap-3 px-5 py-3 flex-shrink-0"
          style={{ borderTop: '1px solid rgba(0,200,255,0.2)' }}
        >
          <button onClick={onCancel} className="ark-action-btn text-xs px-4">
            {tk('cancel', 'Cancel')}
          </button>
          <button
            onClick={onApply}
            className="ark-action-btn text-xs px-5"
            style={{
              color: 'rgba(74,222,128,0.9)',
              outlineColor: 'rgba(74,222,128,0.4)',
              background: 'rgba(74,222,128,0.08)',
            }}
          >
            {resolvedApplyLabel}
          </button>
        </div>
      </div>
    </div>
  )
}
