import { useState, useMemo, useCallback } from 'react'
import { useI18n } from '../i18n/useI18n'

interface IniSection {
  name: string
  pairs: { key: string; value: string; comment?: boolean; line: string }[]
}

function parseIniSections(text: string): IniSection[] {
  const sections: IniSection[] = []
  let current: IniSection | null = null

  for (const raw of text.split('\n')) {
    const line = raw.trim()
    if (!line || line.startsWith(';') || line.startsWith('#')) {
      if (current) {
        current.pairs.push({ key: '', value: '', comment: true, line: raw })
      }
      continue
    }
    const sectionMatch = line.match(/^\[(.+)\]$/)
    if (sectionMatch) {
      current = { name: sectionMatch[1], pairs: [] }
      sections.push(current)
      continue
    }
    if (!current) {
      current = { name: '', pairs: [] }
      sections.push(current)
    }
    const eqIdx = line.indexOf('=')
    if (eqIdx !== -1) {
      current.pairs.push({
        key: line.slice(0, eqIdx).trim(),
        value: line.slice(eqIdx + 1).trim(),
        line: raw,
      })
    } else {
      current.pairs.push({ key: line, value: '', line: raw })
    }
  }

  return sections
}

function serializeSections(sections: IniSection[], edits: Record<string, Record<string, string>>, added: Record<string, Record<string, string>>, removed: Record<string, Set<string>>): string {
  const lines: string[] = []
  for (const sec of sections) {
    if (sec.name) lines.push(`[${sec.name}]`)
    const secEdits = edits[sec.name] ?? {}
    const secAdded = added[sec.name] ?? {}
    const secRemoved = removed[sec.name] ?? new Set()
    for (const pair of sec.pairs) {
      if (pair.comment) {
        lines.push(pair.line)
        continue
      }
      if (secRemoved.has(pair.key)) continue
      if (pair.key in secEdits) {
        lines.push(`${pair.key}=${secEdits[pair.key]}`)
      } else {
        lines.push(pair.key && pair.value !== undefined ? `${pair.key}=${pair.value}` : pair.line)
      }
    }
    const existingKeys = new Set(sec.pairs.filter((p) => !p.comment).map((p) => p.key))
    for (const [key, val] of Object.entries(secAdded)) {
      if (!existingKeys.has(key) && !secRemoved.has(key)) {
        lines.push(`${key}=${val}`)
      }
    }
    lines.push('')
  }
  return lines.join('\n')
}

interface ConfigFormEditorProps {
  content: string
  onSave: (newContent: string) => Promise<void>
  onCancel: () => void
}

export default function ConfigFormEditor({ content, onSave, onCancel }: ConfigFormEditorProps) {
  const { tk } = useI18n()
  const sections = useMemo(() => parseIniSections(content), [content])

  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({})
  const [edits, setEdits] = useState<Record<string, Record<string, string>>>({})
  const [added, setAdded] = useState<Record<string, Record<string, string>>>({})
  const [removed, setRemoved] = useState<Record<string, Set<string>>>({})
  const [newKey, setNewKey] = useState<Record<string, string>>({})
  const [newVal, setNewVal] = useState<Record<string, string>>({})
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const hasChanges = useMemo(() => {
    if (Object.values(edits).some((m) => Object.keys(m).length > 0)) return true
    if (Object.values(added).some((m) => Object.keys(m).length > 0)) return true
    if (Object.values(removed).some((s) => s.size > 0)) return true
    return false
  }, [edits, added, removed])

  const toggleSection = useCallback((name: string) => {
    setCollapsed((prev) => ({ ...prev, [name]: !prev[name] }))
  }, [])

  const setField = useCallback((section: string, key: string, value: string) => {
    setEdits((prev) => ({
      ...prev,
      [section]: { ...(prev[section] ?? {}), [key]: value },
    }))
  }, [])

  const resetField = useCallback((section: string, key: string) => {
    setEdits((prev) => {
      const next = { ...prev }
      const sec = { ...(next[section] ?? {}) }
      delete sec[key]
      next[section] = sec
      return next
    })
  }, [])

  const removeField = useCallback((section: string, key: string) => {
    setRemoved((prev) => {
      const next = new Set(prev[section] ?? [])
      next.add(key)
      return { ...prev, [section]: next }
    })
  }, [])

  const unremoveField = useCallback((section: string, key: string) => {
    setRemoved((prev) => {
      const next = new Set(prev[section] ?? [])
      next.delete(key)
      return { ...prev, [section]: next }
    })
  }, [])

  const addField = useCallback((section: string) => {
    const key = (newKey[section] ?? '').trim()
    const val = (newVal[section] ?? '').trim()
    if (!key) return
    setAdded((prev) => ({
      ...prev,
      [section]: { ...(prev[section] ?? {}), [key]: val },
    }))
    setNewKey((prev) => ({ ...prev, [section]: '' }))
    setNewVal((prev) => ({ ...prev, [section]: '' }))
  }, [newKey, newVal])

  const removeAddedField = useCallback((section: string, key: string) => {
    setAdded((prev) => {
      const sec = { ...(prev[section] ?? {}) }
      delete sec[key]
      return { ...prev, [section]: sec }
    })
  }, [])

  const setAddedValue = useCallback((section: string, key: string, value: string) => {
    setAdded((prev) => ({
      ...prev,
      [section]: { ...(prev[section] ?? {}), [key]: value },
    }))
  }, [])

  const handleSave = async () => {
    setSaving(true)
    setError(null)
    try {
      const newContent = serializeSections(sections, edits, added, removed)
      await onSave(newContent)
    } catch (e) {
      setError(String(e))
    } finally {
      setSaving(false)
    }
  }

  const isRemoved = (section: string, key: string) =>
    (removed[section] ?? new Set()).has(key)

  const getDisplayValue = (section: string, pair: { key: string; value: string }) => {
    return edits[section]?.[pair.key] ?? pair.value
  }

  const isEdited = (section: string, key: string, original: string) =>
    edits[section]?.[key] !== undefined && edits[section][key] !== original

  return (
    <div className="flex flex-col gap-2">
      {error && <p className="text-red-400/80 text-xs">{error}</p>}

      {sections.map((sec) => {
        const isHidden = collapsed[sec.name]
        const secRemoved = removed[sec.name] ?? new Set()
        const secAdded = added[sec.name] ?? {}
        const visiblePairs = sec.pairs.filter((p) => !p.comment && !secRemoved.has(p.key))

        return (
          <div
            key={sec.name || '__root__'}
            className="rounded-md overflow-hidden"
            style={{ border: '1px solid rgba(0,200,255,0.15)' }}
          >
            {/* Section header */}
            <button
              onClick={() => toggleSection(sec.name)}
              className="w-full flex items-center justify-between px-3 py-2 text-left"
              style={{ background: 'rgba(0,200,255,0.06)' }}
            >
              <span
                className="text-xs font-bold tracking-widest uppercase font-mono"
                style={{ color: sec.name ? 'rgba(0,200,255,0.8)' : 'rgba(255,255,255,0.4)' }}
              >
                {sec.name ? `[${sec.name}]` : tk('root_section', '(root)')}
              </span>
              <div className="flex items-center gap-2">
                <span className="text-[10px]" style={{ color: 'rgba(0,200,255,0.35)' }}>
                  {visiblePairs.length} {tk('keys', 'keys')}
                </span>
                <span
                  className="text-[10px] transition-transform"
                  style={{
                    color: 'rgba(0,200,255,0.4)',
                    transform: isHidden ? 'rotate(-90deg)' : 'rotate(0deg)',
                  }}
                >
                  ▼
                </span>
              </div>
            </button>

            {/* Section body */}
            {!isHidden && (
              <div className="px-3 py-2 space-y-1">
                {sec.pairs.map((pair, i) => {
                  if (pair.comment) {
                    return (
                      <div
                        key={`comment-${i}`}
                        className="text-[10px] italic"
                        style={{ color: 'rgba(100,160,100,0.55)' }}
                      >
                        {pair.line.trim()}
                      </div>
                    )
                  }

                  if (isRemoved(sec.name, pair.key)) {
                    return (
                      <div
                        key={pair.key}
                        className="flex items-center gap-2 px-2 py-1 rounded text-[11px]"
                        style={{
                          background: 'rgba(239,68,68,0.06)',
                          border: '1px dashed rgba(239,68,68,0.25)',
                        }}
                      >
                        <span className="line-through opacity-40">{pair.key}</span>
                        <span className="text-red-400/50">=</span>
                        <span className="line-through opacity-40">{pair.value}</span>
                        <button
                          onClick={() => unremoveField(sec.name, pair.key)}
                          className="ml-auto text-[9px] text-ark-cyan/60 hover:text-ark-cyan/90"
                        >
                          {tk('undo_remove', 'Undo')}
                        </button>
                      </div>
                    )
                  }

                  const edited = isEdited(sec.name, pair.key, pair.value)
                  const displayVal = getDisplayValue(sec.name, pair)

                  return (
                    <div
                      key={pair.key}
                      className="flex items-center gap-2 px-2 py-1.5 rounded text-[11px]"
                      style={{
                        background: edited ? 'rgba(0,200,255,0.06)' : 'rgba(255,255,255,0.02)',
                        border: edited ? '1px solid rgba(0,200,255,0.2)' : '1px solid transparent',
                      }}
                    >
                      <span
                        className="font-mono flex-shrink-0 text-right"
                        style={{ color: 'rgba(0,200,255,0.6)', minWidth: '10rem' }}
                      >
                        {pair.key}
                      </span>
                      <span style={{ color: 'rgba(255,255,255,0.2)' }}>=</span>
                      <input
                        type="text"
                        value={displayVal}
                        onChange={(e) => setField(sec.name, pair.key, e.target.value)}
                        className="flex-1 bg-transparent text-xs font-mono px-1.5 py-0.5 rounded focus:outline-none"
                        style={{
                          color: edited ? 'rgba(74,222,128,0.9)' : 'rgba(180,220,255,0.75)',
                          border: '1px solid rgba(0,200,255,0.15)',
                        }}
                      />
                      {edited && (
                        <button
                          onClick={() => resetField(sec.name, pair.key)}
                          className="text-[9px] text-ark-cyan/50 hover:text-ark-cyan/90 flex-shrink-0"
                          title={tk('revert', 'Revert')}
                        >
                          ↩
                        </button>
                      )}
                      <button
                        onClick={() => removeField(sec.name, pair.key)}
                        className="text-[9px] text-red-400/40 hover:text-red-400/80 flex-shrink-0"
                        title={tk('remove_key', 'Remove key')}
                      >
                        ✕
                      </button>
                    </div>
                  )
                })}

                {/* Added keys */}
                {Object.entries(secAdded).map(([key, val]) => (
                  <div
                    key={`added-${key}`}
                    className="flex items-center gap-2 px-2 py-1.5 rounded text-[11px]"
                    style={{
                      background: 'rgba(74,222,128,0.05)',
                      border: '1px dashed rgba(74,222,128,0.3)',
                    }}
                  >
                    <span
                      className="font-mono flex-shrink-0 text-right"
                      style={{ color: 'rgba(74,222,128,0.7)', minWidth: '10rem' }}
                    >
                      {key}
                    </span>
                    <span style={{ color: 'rgba(255,255,255,0.2)' }}>=</span>
                    <input
                      type="text"
                      value={val}
                      onChange={(e) => setAddedValue(sec.name, key, e.target.value)}
                      className="flex-1 bg-transparent text-xs font-mono px-1.5 py-0.5 rounded focus:outline-none"
                      style={{
                        color: 'rgba(74,222,128,0.85)',
                        border: '1px solid rgba(74,222,128,0.25)',
                      }}
                    />
                    <button
                      onClick={() => removeAddedField(sec.name, key)}
                      className="text-[9px] text-red-400/50 hover:text-red-400/80 flex-shrink-0"
                    >
                      ✕
                    </button>
                  </div>
                ))}

                {/* Add new key row */}
                <div
                  className="flex items-center gap-2 px-2 py-1.5 rounded"
                  style={{ border: '1px dashed rgba(0,200,255,0.2)' }}
                >
                  <input
                    type="text"
                    value={newKey[sec.name] ?? ''}
                    onChange={(e) => setNewKey((prev) => ({ ...prev, [sec.name]: e.target.value }))}
                    placeholder={tk('new_key', 'new key')}
                    className="bg-transparent text-[11px] font-mono px-1.5 py-0.5 rounded focus:outline-none"
                    style={{
                      color: 'rgba(0,200,255,0.7)',
                      border: '1px solid rgba(0,200,255,0.15)',
                      minWidth: '10rem',
                    }}
                    onKeyDown={(e) => e.key === 'Enter' && addField(sec.name)}
                  />
                  <span style={{ color: 'rgba(255,255,255,0.15)' }}>=</span>
                  <input
                    type="text"
                    value={newVal[sec.name] ?? ''}
                    onChange={(e) => setNewVal((prev) => ({ ...prev, [sec.name]: e.target.value }))}
                    placeholder={tk('new_value', 'value')}
                    className="flex-1 bg-transparent text-[11px] font-mono px-1.5 py-0.5 rounded focus:outline-none"
                    style={{
                      color: 'rgba(0,200,255,0.7)',
                      border: '1px solid rgba(0,200,255,0.15)',
                    }}
                    onKeyDown={(e) => e.key === 'Enter' && addField(sec.name)}
                  />
                  <button
                    onClick={() => addField(sec.name)}
                    className="text-[10px] px-2 py-0.5 rounded"
                    style={{
                      color: 'rgba(0,200,255,0.7)',
                      border: '1px solid rgba(0,200,255,0.3)',
                    }}
                  >
                    +
                  </button>
                </div>
              </div>
            )}
          </div>
        )
      })}

      {/* Action bar */}
      <div className="flex items-center gap-3 pt-2">
        <button
          onClick={handleSave}
          disabled={saving || !hasChanges}
          className="ark-action-btn text-[10px] px-4 py-1.5 disabled:opacity-40"
        >
          {saving ? tk('saving', 'Saving...') : tk('save_changes', 'Save Changes')}
        </button>
        <button onClick={onCancel} className="ark-action-btn text-[10px] px-3 py-1.5">
          {tk('cancel', 'Cancel')}
        </button>
        {hasChanges && (
          <span className="text-[10px]" style={{ color: 'rgba(0,200,255,0.45)' }}>
            {tk('unsaved_changes', 'Unsaved changes')}
          </span>
        )}
      </div>
    </div>
  )
}
