import { useState, useMemo, useEffect, useCallback } from 'react'
import type { ServerConfig } from '../../types'
import { invoke } from '../../services/tauri'
import { useBackupStore, type BackupStore } from '../../stores/backupStore'
import { useShallow } from 'zustand/react/shallow'
import { useModsStore } from '../../stores/modsStore'
import { useI18n } from '../../i18n/useI18n'
import { useTextHistory } from '../../hooks/useTextHistory'
import { CustomFileSaveStrategy } from '../../services/configSaveStrategies'
import { syncModIniToServerFile } from '../../services/iniSectionMerge'
import ConfigFormEditor from '../ConfigFormEditor'

interface ModsTabProps {
  config: ServerConfig | null
  onConfigSaved?: (config: ServerConfig) => void
  /** Called when the user clicks "Open in Config INI ↗" — parent should switch the OptionsModal to the 'config' tab. */
  onRequestSwitchToConfigTab?: () => void
}

interface ModMeta {
  id: string
  name?: string
  summary?: string
  logoUrl?: string | null
  fetched: boolean
}

/**
 * Path on disk where a per-mod INI lives.
 * Convention: <server_dir>\ShooterGame\Saved\ModConfigs\<modid>.ini
 */
function modIniPath(serverDir: string, modId: string): string {
  const cleanId = modId.trim()
  const sep = serverDir.includes('\\') && !serverDir.includes('/') ? '\\' : '/'
  const base = serverDir.replace(/[\\/]+$/, '')
  return `${base}${sep}ShooterGame${sep}Saved${sep}ModConfigs${sep}${cleanId}.ini`
}

type EditMode = 'idle' | 'form' | 'raw'

export default function ModsTab({ config, onConfigSaved, onRequestSwitchToConfigTab }: ModsTabProps) {
  const { tk } = useI18n()
  const { addCustomConfigTab, setPendingCustomTabId } = useBackupStore(
    useShallow((s: BackupStore) => ({
      addCustomConfigTab: s.addCustomConfigTab,
      setPendingCustomTabId: s.setPendingCustomTabId,
    }))
  )
  const getModInfo = useModsStore((s) => s.getModInfo)
  const setModInfo = useModsStore((s) => s.setModInfo)

  const [selectedModId, setSelectedModId] = useState<string | null>(null)
  const [modMetas, setModMetas] = useState<Record<string, ModMeta>>({})
  const [editMode, setEditMode] = useState<EditMode>('idle')
  const [isSaving, setIsSaving] = useState(false)
  const [saveStatus, setSaveStatus] = useState<string | null>(null)
  const [syncStatus, setSyncStatus] = useState<string | null>(null)
  const [saveError, setSaveError] = useState<string | null>(null)

  const { text, commit, reset, undo, redo, canUndo, canRedo } = useTextHistory('')

  const activeMods = useMemo(() => config?.mods?.active_mods ?? [], [config?.mods?.active_mods])

  // Build/init modMetas from active_mods (cheap; doesn't re-fetch).
  useEffect(() => {
    setModMetas((prev) => {
      const next: Record<string, ModMeta> = {}
      for (const id of activeMods) {
        next[id] = prev[id] ?? { id, fetched: false }
      }
      return next
    })
  }, [activeMods])

  // Resolve names: first from modsStore cache; if missing, async via get_curseforge_mod_by_id.
  useEffect(() => {
    let cancelled = false
    for (const id of activeMods) {
      const cached = getModInfo(id)
      if (cached) {
        setModMetas((prev) =>
          prev[id]?.fetched ? prev : { ...prev, [id]: { id, name: cached.name, summary: cached.summary, logoUrl: cached.logoUrl, fetched: true } }
        )
        continue
      }
      invoke<{ id: string; name: string; summary: string; logo_url: string | null; slug: string } | null>(
        'get_curseforge_mod_by_id',
        { modId: id }
      )
        .then((m) => {
          if (cancelled || !m) return
          setModInfo(id, { name: m.name, summary: m.summary, logoUrl: m.logo_url, slug: m.slug, downloadCount: 0, categories: [] })
          setModMetas((prev) => ({ ...prev, [id]: { id, name: m.name, summary: m.summary, logoUrl: m.logo_url, fetched: true } }))
        })
        .catch(() => {
          if (cancelled) return
          setModMetas((prev) => ({ ...prev, [id]: { id, fetched: true } }))
        })
    }
    return () => { cancelled = true }
  }, [activeMods, getModInfo, setModInfo])

  const selectedPath = useMemo(() => {
    if (!config || !selectedModId) return null
    return modIniPath(config.paths.server_dir, selectedModId)
  }, [config, selectedModId])

  const loadModIni = useCallback(async (modId: string, path: string) => {
    try {
      const disk = await invoke<string>('read_text_file', { path })
      reset(disk)
    } catch {
      reset('')
    }
  }, [reset])

  useEffect(() => {
    if (!selectedModId || !selectedPath) {
      reset('')
      setEditMode('idle')
      return
    }
    loadModIni(selectedModId, selectedPath)
    setEditMode('idle')
    setSaveStatus(null)
    setSyncStatus(null)
    setSaveError(null)
  }, [selectedModId, selectedPath, loadModIni, reset])

  /** Sync the operator's mod sections into the server's GameUserSettings.ini. */
  const syncToServer = useCallback(
    async (modIniText: string) => {
      const serverIniPath = (config as ServerConfig)?.paths?.gamesettings_ini_path
      if (!serverIniPath) return
      try {
        const sections = await syncModIniToServerFile(modIniText, serverIniPath)
        if (sections.length > 0) {
          setSyncStatus(tk('mods_synced_sections', 'Synced to GameUserSettings.ini: ') + sections.join(', '))
        } else {
          setSyncStatus(tk('mods_synced_none', 'No named sections to sync (mod INI saved only).'))
        }
      } catch (e) {
        setSaveError(tk('mods_sync_failed', 'Failed to sync to GameUserSettings.ini: ') + String(e))
      }
    },
    [config, tk],
  )

  const handleSave = async () => {
    if (!selectedPath) return
    setIsSaving(true)
    setSaveStatus(null)
    setSyncStatus(null)
    setSaveError(null)
    try {
      const strat = new CustomFileSaveStrategy(selectedPath)
      const result = await strat.save(text, config as ServerConfig)
      setSaveStatus(result.message ?? tk('mods_file_saved', 'Mod INI saved'))
      await syncToServer(text)
      setEditMode('idle')
    } catch (e) {
      setSaveError(String(e))
    } finally {
      setIsSaving(false)
    }
  }

  const handleFormSave = async (newContent: string) => {
    if (!selectedPath) return
    setIsSaving(true)
    setSaveStatus(null)
    setSyncStatus(null)
    setSaveError(null)
    try {
      const strat = new CustomFileSaveStrategy(selectedPath)
      await strat.save(newContent, config as ServerConfig)
      setSaveStatus(tk('mods_file_saved', 'Mod INI saved'))
      await syncToServer(newContent)
      setEditMode('idle')
    } catch (e) {
      setSaveError(String(e))
    } finally {
      setIsSaving(false)
    }
  }

  const handleOpenInConfigTab = () => {
    if (!selectedPath || !selectedModId) return
    const tabId = `mod-${selectedModId}-${Date.now()}`
    addCustomConfigTab({ id: tabId, label: `Mod ${selectedModId}.ini`, path: selectedPath })
    setPendingCustomTabId(tabId)
    onRequestSwitchToConfigTab?.()
  }

  const isEditing = editMode !== 'idle'

  if (!config) {
    return <p className="text-ark-cyan/40 text-sm text-center py-8">{tk('mods_load_config_first', 'Load the server configuration first.')}</p>
  }

  // Empty state
  if (activeMods.length === 0) {
    return (
      <div className="space-y-3">
        <h3 className="text-ark-cyan/80 text-sm font-bold tracking-widest uppercase">
          {tk('tab_mods', 'Mods')}
        </h3>
        <div className="rounded-lg p-4" style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}>
          <p className="text-ark-cyan/60 text-sm mb-2">{tk('mods_no_active_title', 'No active mods')}</p>
          <p className="text-ark-cyan/35 text-xs leading-relaxed">
            {tk(
              'mods_no_active_desc',
              'Add mods in the "Mod Settings" tab first — they will appear here so you can override their INI config per mod.'
            )}
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-3 flex flex-col" style={{ minHeight: '60vh' }}>
      <h3 className="text-ark-cyan/80 text-sm font-bold tracking-widest uppercase">
        {tk('tab_mods', 'Mods')}
      </h3>
      <p className="text-ark-cyan/35 text-xs leading-relaxed">
        {tk(
          'mods_help_intro',
          'Select a mod to edit its per-mod INI override. The file lives at <server>\\ShooterGame\\Saved\\ModConfigs\\<modid>.ini. ASE/ASA does not auto-load this file; you still need a ConfigOverrideFile= or the mod-specific entry in GameUserSettings.ini/Game.ini — see the in-app help below.'
        )}
      </p>

      {/* Layout: left = mod picker (1fr), right = editor (3fr) */}
      <div className="flex gap-3" style={{ minHeight: '50vh' }}>
        {/* Mod picker */}
        <div className="flex flex-col gap-1 flex-shrink-0" style={{ width: '240px' }}>
          <p className="text-ark-cyan/40 text-[10px] tracking-widest uppercase font-bold mb-1">
            {tk('mods_picker_title', 'Installed mods')}
          </p>
          <div className="flex flex-col gap-1 overflow-y-auto" style={{ maxHeight: 'calc(90vh - 200px)' }}>
            {activeMods.map((id) => {
              const meta = modMetas[id]
              const isSel = selectedModId === id
              const label = meta?.name ?? (meta?.fetched ? id : `${id}…`)
              return (
                <button
                  key={id}
                  onClick={() => setSelectedModId(id)}
                  className="text-left px-3 py-2 rounded-md transition-all"
                  style={{
                    background: isSel ? 'rgba(0,200,255,0.12)' : 'rgba(255,255,255,0.03)',
                    border: `1px solid ${isSel ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.08)'}`,
                  }}
                >
                  <p
                    className="text-xs font-bold leading-tight"
                    style={{ color: isSel ? 'rgba(0,200,255,0.9)' : 'rgba(180,220,255,0.7)' }}
                  >
                    {label}
                  </p>
                  <p className="text-[10px] text-ark-cyan/35 mt-0.5 leading-tight truncate" title={meta?.summary ?? id}>
                    {meta?.summary ?? `#${id}`}
                  </p>
                </button>
              )
            })}
          </div>
        </div>

        {/* Editor pane */}
        <div className="flex-1 flex flex-col gap-2 min-w-0">
          {!selectedModId ? (
            <div
              className="flex-1 flex items-center justify-center rounded-lg"
              style={{ background: 'rgba(0,0,0,0.35)', border: '1px dashed rgba(0,200,255,0.25)' }}
            >
              <p className="text-ark-cyan/35 text-xs">← {tk('mods_select_prompt', 'Select a mod on the left to edit its INI')}</p>
            </div>
          ) : (
            <>
              {/* Path bar */}
              <div className="flex items-center gap-2 flex-wrap">
                <code className="text-[11px] text-ark-cyan/45 font-mono break-all" style={{ flex: 1, minWidth: '200px' }}>
                  📄 {selectedPath}
                </code>
                <button
                  onClick={handleOpenInConfigTab}
                  className="ark-action-btn text-[10px] px-3 py-1.5"
                  title={tk('mods_open_in_config_title', 'Open in "Config INI" tab')}
                >
                  {tk('mods_open_in_config', 'Open in Config INI ↗')}
                </button>
              </div>

              {/* Toolbar */}
              <div className="flex items-center gap-2 flex-wrap">
                {editMode === 'idle' ? (
                  <>
                    <button onClick={() => setEditMode('form')} className="ark-action-btn text-[10px] px-3 py-1.5">
                      {tk('form_edit', 'Form Edit')}
                    </button>
                    <button onClick={() => setEditMode('raw')} className="ark-action-btn text-[10px] px-3 py-1.5">
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
                    <button
                      onClick={() => { setEditMode('idle'); if (selectedPath) loadModIni(selectedModId, selectedPath) }}
                      className="ark-action-btn text-[10px] px-3 py-1.5"
                    >
                      {tk('cancel', 'Cancel')}
                    </button>
                    {editMode === 'raw' && (
                      <>
                        <button onClick={undo} disabled={!canUndo} className="ark-action-btn text-[10px] px-2 py-1.5 disabled:opacity-25">↩</button>
                        <button onClick={redo} disabled={!canRedo} className="ark-action-btn text-[10px] px-2 py-1.5 disabled:opacity-25">↪</button>
                        <span className="text-ark-cyan/30 text-[10px]">{tk('undo_redo_hint', 'Ctrl+Z / Ctrl+Y')}</span>
                      </>
                    )}
                  </>
                )}
              </div>

              {saveStatus && <p className="text-green-400/80 text-xs animate-pulse">{saveStatus}</p>}
              {syncStatus && <p className="text-emerald-400/70 text-xs">{syncStatus}</p>}
              {saveError && <p className="text-red-400/80 text-xs">{saveError}</p>}

              {/* Content */}
              {editMode === 'form' ? (
                <ConfigFormEditor
                  content={text}
                  onSave={handleFormSave}
                  onCancel={() => { setEditMode('idle'); if (selectedPath) loadModIni(selectedModId, selectedPath) }}
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
                    minHeight: 'calc(80vh - 320px)',
                  }}
                  spellCheck={false}
                />
              ) : (
                <div
                  className="flex-1 overflow-y-auto font-mono text-[11px] leading-relaxed rounded-lg p-3"
                  style={{
                    background: 'rgba(0,0,0,0.35)',
                    border: '1px solid rgba(0,200,255,0.12)',
                    minHeight: 'calc(80vh - 320px)',
                  }}
                >
                  {text.trim() ? (
                    text.split('\n').map((line, i) => {
                      const isComment = line.trimStart().startsWith(';') || line.trimStart().startsWith('#')
                      const isSection = line.trimStart().startsWith('[')
                      let color = 'rgba(180,220,255,0.6)'
                      if (isComment) color = 'rgba(100,160,100,0.55)'
                      else if (isSection) color = 'rgba(0,200,255,0.85)'
                      else if (line.includes('=')) color = 'rgba(180,220,255,0.75)'
                      return (
                        <div key={i} className="flex">
                          <span
                            className="select-none flex-shrink-0 text-right pr-3 w-9"
                            style={{ color: 'rgba(100,130,150,0.4)', fontSize: '10px', lineHeight: '1.6' }}
                          >
                            {line.trim() === '' ? '' : i + 1}
                          </span>
                          <span style={{ color, whiteSpace: 'pre-wrap', flex: 1 }}>{line || ' '}</span>
                        </div>
                      )
                    })
                  ) : (
                    <p className="text-ark-cyan/30 text-xs py-4">{tk('mods_empty_ini_hint', 'File empty. Click "Form Edit" or "Raw Edit" to add mod settings.')}</p>
                  )}
                </div>
              )}
            </>
          )}
        </div>
      </div>

      {/* Help panel */}
      <details className="rounded-lg" style={{ background: 'rgba(255,255,255,0.02)', border: '1px solid rgba(255,255,255,0.06)' }}>
        <summary className="text-ark-cyan/50 text-xs cursor-pointer px-3 py-2 tracking-wider">
          {tk('mods_help_summary', 'How do I make ARK load the mod INI override?')}
        </summary>
        <div className="px-4 py-3 space-y-2 text-xs text-ark-cyan/55 leading-relaxed">
          <p>
            {tk(
              'mods_help_body_1',
              'When you hit Save, this tool writes your mod-config to ModConfigs\\<modid>.ini AND injects every [SectionName] you wrote directly into the server\'s GameUserSettings.ini (replacing the old section if it was there). Example for Upgrade Station (mod 930490):'
            )}
          </p>
          <pre
            className="font-mono text-[10px] p-2 rounded"
            style={{ background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(0,200,255,0.15)', color: 'rgba(180,220,255,0.8)' }}
          >{`; In the per-mod editor (you DON'T have to touch GameUserSettings.ini manually):
[UpgradeStation]
ItemRatingMultiplier=0.5
QualityIncreaseChance=1.5
ResourcesRequiredBaseMultiplier=0.1

; On Save, the above block is automatically written to:
;   <server>\\ShooterGame\\Saved\\ModConfigs\\930490.ini   (your editable copy)
;   <server>\\ShooterGame\\Saved\\Config\\WindowsServer\\GameUserSettings.ini   (where ARK reads it)
`}</pre>
          <p>
            {tk(
              'mods_help_body_2',
              'Some mods read their own section name in GameUserSettings.ini (e.g. [Upgradestation] for Upgrade Station by Ghazlawl). Check the mod\'s CurseForge page or wiki for the exact section name and key. After saving, run STOP SERVER → START SERVER for ARK to re-read the config.'
            )}
          </p>
          <p className="text-ark-cyan/35">
            {tk(
              'mods_help_body_3',
              'Note: the file path uses the server_dir configured in Options → General → Paths. If you moved the install elsewhere, this tool still writes to the configured path automatically.'
            )}
          </p>
        </div>
      </details>
    </div>
  )
}
