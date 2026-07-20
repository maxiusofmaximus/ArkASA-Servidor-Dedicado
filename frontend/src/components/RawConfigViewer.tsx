import { useState, useMemo, useCallback, useEffect } from 'react'
import type { ServerConfig } from '../types'
import { invoke } from '../services/tauri'
import { generateGameUserSettings, generateGameIni, generateToml } from '../services/configGenerators'
import { type ConfigSaveStrategy, TomlSaveStrategy, IniSaveStrategy, CustomFileSaveStrategy } from '../services/configSaveStrategies'
import { useBackupStore, type BackupStore } from '../stores/backupStore'
import { useI18n } from '../i18n/useI18n'
import { useTextHistory } from '../hooks/useTextHistory'
import { useShallow } from 'zustand/react/shallow'
import ConfigFormEditor from './ConfigFormEditor'

interface RawConfigViewerProps {
  config: ServerConfig
  onConfigSaved?: (config: ServerConfig) => void
}

type BuiltinTab = 'gameusersettings' | 'game' | 'toml'
type EditMode = 'idle' | 'form' | 'raw'

const BUILTIN_TABS: { id: BuiltinTab; label: string }[] = [
  { id: 'gameusersettings', label: 'GameUserSettings.ini' },
  { id: 'game', label: 'Game.ini' },
  { id: 'toml', label: 'config.toml' },
]

function pathForBuiltinTab(tab: BuiltinTab, config: ServerConfig): string | null {
  if (tab === 'game') return config.paths.game_ini_path
  if (tab === 'gameusersettings') return config.paths.gamesettings_ini_path
  return null
}

function generatedForTab(tab: BuiltinTab, config: ServerConfig): string {
  if (tab === 'gameusersettings') return generateGameUserSettings(config)
  if (tab === 'game') return generateGameIni(config)
  return generateToml(config)
}

function getSaveStrategy(tab: BuiltinTab | undefined, customPath: string | null, config: ServerConfig): ConfigSaveStrategy | null {
  if (tab === 'toml') return new TomlSaveStrategy()
  if (tab === 'game' || tab === 'gameusersettings') {
    const path = pathForBuiltinTab(tab, config)
    return path ? new IniSaveStrategy(path) : null
  }
  if (customPath) return new CustomFileSaveStrategy(customPath)
  return null
}

export default function RawConfigViewer({ config, onConfigSaved }: RawConfigViewerProps) {
  const { customConfigTabs, addCustomConfigTab, removeCustomConfigTab, pendingCustomTabId, setPendingCustomTabId } = useBackupStore(
    useShallow((s: BackupStore) => ({
      customConfigTabs: s.customConfigTabs,
      addCustomConfigTab: s.addCustomConfigTab,
      removeCustomConfigTab: s.removeCustomConfigTab,
      pendingCustomTabId: s.pendingCustomTabId,
      setPendingCustomTabId: s.setPendingCustomTabId,
    }))
  )
  const { tk } = useI18n()

  const [activeTab, setActiveTab] = useState<string>('gameusersettings')
  const [editMode, setEditMode] = useState<EditMode>('idle')
  const [search, setSearch] = useState('')
  const [copied, setCopied] = useState(false)
  const [saveStatus, setSaveStatus] = useState<string | null>(null)
  const [saveError, setSaveError] = useState<string | null>(null)
  const [isSaving, setIsSaving] = useState(false)

  const { text, commit, reset, undo, redo, canUndo, canRedo } = useTextHistory('')

  const allTabs = useMemo(
    () => [
      ...BUILTIN_TABS,
      ...customConfigTabs.map((t) => ({ id: t.id, label: t.label })),
    ],
    [customConfigTabs]
  )

  const activeBuiltin = BUILTIN_TABS.find((t) => t.id === activeTab)?.id as BuiltinTab | undefined
  const activeCustom = customConfigTabs.find((t) => t.id === activeTab)

  const generatedContent = useMemo(() => {
    if (activeBuiltin) return generatedForTab(activeBuiltin, config)
    return ''
  }, [activeBuiltin, config])

  const isEditing = editMode !== 'idle'
  const displayContent = isEditing ? text : (activeBuiltin ? generatedContent : text)

  const loadTabContent = useCallback(async (tabId: string) => {
    const builtin = BUILTIN_TABS.find((t) => t.id === tabId)?.id as BuiltinTab | undefined
    const custom = customConfigTabs.find((t) => t.id === tabId)

    if (builtin) {
      const path = pathForBuiltinTab(builtin, config)
      const generated = generatedForTab(builtin, config)
      if (path) {
        try {
          const disk = await invoke<string>('read_text_file', { path })
          reset(disk.trim() ? disk : generated)
        } catch {
          reset(generated)
        }
      } else {
        reset(generated)
      }
    } else if (custom) {
      try {
        const disk = await invoke<string>('read_text_file', { path: custom.path })
        reset(disk)
      } catch {
        reset('')
      }
    }
  }, [config, customConfigTabs, reset])

  useEffect(() => {
    loadTabContent(activeTab)
    setEditMode('idle')
    setSearch('')
  }, [activeTab]) // eslint-disable-line react-hooks/exhaustive-deps

  // Consume a pending "open this custom tab" request from the ModsTab "Open in Config INI ↗" button.
  useEffect(() => {
    if (!pendingCustomTabId) return
    const exists = customConfigTabs.find((t) => t.id === pendingCustomTabId)
    if (exists) {
      setActiveTab(pendingCustomTabId)
      setPendingCustomTabId(null)
    }
  }, [pendingCustomTabId, customConfigTabs, setPendingCustomTabId])

  const handleStartEdit = async (mode: EditMode) => {
    await loadTabContent(activeTab)
    setEditMode(mode)
  }

  const handleCancelEdit = () => {
    setEditMode('idle')
    loadTabContent(activeTab)
  }

  const saveContent = async (content: string) => {
    const strategy = getSaveStrategy(activeBuiltin, activeCustom?.path ?? null, config)
    if (!strategy) return

    setIsSaving(true)
    setSaveStatus(null)
    setSaveError(null)
    try {
      const result = await strategy.save(content, config)
      if (result.updatedConfig) onConfigSaved?.(result.updatedConfig)
      setSaveStatus(result.message ?? tk('config_saved', 'Configuration saved'))
      setEditMode('idle')
    } catch (e) {
      setSaveError(String(e))
    } finally {
      setIsSaving(false)
    }
  }

  const handleSave = () => saveContent(text)

  const handleFormSave = async (newContent: string) => {
    await saveContent(newContent)
  }

  const handleAddTab = () => {
    const label = prompt(tk('custom_tab_label_prompt', 'Tab name (e.g. Custom.ini):'))
    if (!label?.trim()) return
    const path = prompt(tk('custom_tab_path_prompt', 'Full file path:'))
    if (!path?.trim()) return
    const id = `custom-${Date.now()}`
    addCustomConfigTab({ id, label: label.trim(), path: path.trim() })
    setActiveTab(id)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isEditing) return
    if (e.ctrlKey && e.key === 'z' && !e.shiftKey) {
      e.preventDefault()
      undo()
    }
    if ((e.ctrlKey && e.key === 'y') || (e.ctrlKey && e.shiftKey && e.key === 'z')) {
      e.preventDefault()
      redo()
    }
  }

  const q = search.trim().toLowerCase()
  const lines = useMemo(() => displayContent.split('\n'), [displayContent])
  const matchCount = useMemo(
    () => (q ? lines.filter((l) => l.toLowerCase().includes(q)).length : 0),
    [lines, q]
  )

  const handleCopy = () => {
    navigator.clipboard.writeText(displayContent)
    setCopied(true)
    setTimeout(() => setCopied(false), 1800)
  }

  const activeLabel = allTabs.find((t) => t.id === activeTab)?.label ?? activeTab

  return (
    <div className="flex flex-col gap-3 h-full" onKeyDown={handleKeyDown}>
      {/* Sub-tab bar */}
      <div className="flex gap-1 flex-wrap items-center">
        {allTabs.map((t) => (
          <button
            key={t.id}
            onClick={() => setActiveTab(t.id)}
            className="px-3 py-1.5 text-[11px] font-bold tracking-wider rounded-md transition-all font-mono"
            style={{
              background: activeTab === t.id ? 'rgba(0,200,255,0.12)' : 'rgba(255,255,255,0.03)',
              border: `1px solid ${activeTab === t.id ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.08)'}`,
              color: activeTab === t.id ? 'rgba(0,200,255,0.9)' : 'rgba(255,255,255,0.35)',
            }}
          >
            {t.label}
          </button>
        ))}
        <button
          onClick={handleAddTab}
          className="px-2 py-1.5 text-[11px] rounded-md"
          style={{ border: '1px dashed rgba(0,200,255,0.35)', color: 'rgba(0,200,255,0.5)' }}
          title={tk('add_config_tab', 'Add config file')}
        >
          + {tk('add', 'Add')}
        </button>
        {activeCustom && (
          <button
            onClick={() => {
              if (confirm(tk('remove_tab_confirm', 'Remove this tab?'))) {
                removeCustomConfigTab(activeCustom.id)
                setActiveTab('gameusersettings')
              }
            }}
            className="px-2 py-1 text-[10px] text-red-400/60 hover:text-red-400"
          >
            ✕
          </button>
        )}
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 flex-wrap">
        {editMode === 'idle' ? (
          <>
            <button onClick={() => handleStartEdit('form')} className="ark-action-btn text-[10px] px-3 py-1.5">
              {tk('form_edit', 'Form Edit')}
            </button>
            <button onClick={() => handleStartEdit('raw')} className="ark-action-btn text-[10px] px-3 py-1.5">
              {tk('raw_edit', 'Raw Edit')}
            </button>
          </>
        ) : (
          <>
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="ark-action-btn text-[10px] px-3 py-1.5 disabled:opacity-40"
              style={{ display: editMode === 'raw' ? undefined : 'none' }}
            >
              {isSaving ? tk('saving', 'Saving...') : tk('save', 'Save')}
            </button>
            <button onClick={handleCancelEdit} className="ark-action-btn text-[10px] px-3 py-1.5">
              {tk('cancel', 'Cancel')}
            </button>
            {editMode === 'raw' && (
              <>
                <button
                  onClick={undo}
                  disabled={!canUndo}
                  className="ark-action-btn text-[10px] px-2 py-1.5 disabled:opacity-25"
                >
                  ↩
                </button>
                <button
                  onClick={redo}
                  disabled={!canRedo}
                  className="ark-action-btn text-[10px] px-2 py-1.5 disabled:opacity-25"
                >
                  ↪
                </button>
                <span className="text-ark-cyan/30 text-[10px]">{tk('undo_redo_hint', 'Ctrl+Z / Ctrl+Y')}</span>
              </>
            )}
          </>
        )}

        <div className="relative flex-1 min-w-[140px]">
          <span className="absolute left-3 top-1/2 -translate-y-1/2 text-ark-cyan/40 text-xs pointer-events-none">🔍</span>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={tk('config_search_placeholder', 'Search keys…')}
            disabled={isEditing}
            className="w-full bg-transparent border border-ark-cyan/25 text-ark-cyan/90 text-xs pl-8 pr-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/25 font-mono disabled:opacity-40"
          />
        </div>

        <button onClick={handleCopy} className="ark-action-btn text-[10px] px-3 py-1.5 flex-shrink-0">
          {copied ? '✓ Copiado' : '⎘ Copiar'}
        </button>
      </div>

      {saveStatus && <p className="text-green-400/80 text-xs">{saveStatus}</p>}
      {saveError && <p className="text-red-400/80 text-xs">{saveError}</p>}

      {/* Content */}
      {editMode === 'form' ? (
        <ConfigFormEditor
          content={text}
          onSave={handleFormSave}
          onCancel={handleCancelEdit}
        />
      ) : editMode === 'raw' ? (
        <textarea
          value={text}
          onChange={(e) => commit(e.target.value)}
          className="flex-1 w-full font-mono text-[11px] leading-relaxed rounded-lg p-3 resize-none focus:outline-none"
          style={{
            background: 'rgba(0,0,0,0.35)',
            border: '1px solid rgba(0,200,255,0.25)',
            color: 'rgba(180,220,255,0.85)',
            minHeight: 'calc(90vh - 320px)',
            maxHeight: 'calc(90vh - 320px)',
          }}
          spellCheck={false}
        />
      ) : (
        <div
          className="flex-1 overflow-y-auto font-mono text-[11px] leading-relaxed rounded-lg"
          style={{
            background: 'rgba(0,0,0,0.35)',
            border: '1px solid rgba(0,200,255,0.12)',
            maxHeight: 'calc(90vh - 260px)',
          }}
        >
          <div className="p-3 space-y-0">
            {lines.map((line, i) => {
              const isMatch = q && line.toLowerCase().includes(q)
              const isComment = line.trimStart().startsWith(';') || line.trimStart().startsWith('#')
              const isSection = line.startsWith('[')
              const isEmpty = line.trim() === ''

              let color = 'rgba(180,220,255,0.6)'
              if (isComment) color = 'rgba(100,160,100,0.55)'
              else if (isSection) color = 'rgba(0,200,255,0.85)'
              else if (line.includes('=')) color = 'rgba(180,220,255,0.75)'

              return (
                <div
                  key={i}
                  className="flex"
                  style={{
                    background: isMatch ? 'rgba(0,200,255,0.12)' : 'transparent',
                    borderLeft: isMatch ? '2px solid rgba(0,200,255,0.6)' : '2px solid transparent',
                    paddingLeft: isMatch ? '6px' : '8px',
                  }}
                >
                  <span
                    className="select-none flex-shrink-0 text-right pr-3 w-9"
                    style={{ color: 'rgba(100,130,150,0.4)', fontSize: '10px', lineHeight: '1.6' }}
                  >
                    {isEmpty ? '' : i + 1}
                  </span>
                  <span style={{ color, whiteSpace: 'pre-wrap', flex: 1 }}>
                    {isMatch && q ? highlightMatch(line, q) : line || ' '}
                  </span>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {q && !isEditing && matchCount === 0 && (
        <p className="text-ark-cyan/30 text-xs text-center py-1">
          Sin resultados para "{search}" en {activeLabel}
        </p>
      )}
    </div>
  )
}

// Highlight the matching substring in a line
function highlightMatch(line: string, q: string) {
  const idx = line.toLowerCase().indexOf(q)
  if (idx === -1) return line
  const before = line.slice(0, idx)
  const match = line.slice(idx, idx + q.length)
  const after = line.slice(idx + q.length)
  return (
    <>
      {before}
      <span style={{ background: 'rgba(0,200,255,0.35)', color: '#fff', borderRadius: '2px', padding: '0 1px' }}>
        {match}
      </span>
      {after}
    </>
  )
}
