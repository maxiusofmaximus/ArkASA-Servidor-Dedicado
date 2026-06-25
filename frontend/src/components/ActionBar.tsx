import React, { useState, useRef, useEffect } from 'react'
import Tooltip from './Tooltip'
import DropdownPortal from './DropdownPortal'
import { useI18n } from '../i18n/useI18n'
import { invoke } from '../services/tauri'
import type { ServerConfig } from '../types'

interface ActionBarProps {
  onSave?: () => Promise<void> | void
  onReset?: () => void
  onBack?: () => void
  onChooseDifficulty?: () => void
  onStartServer?: (mapIndex?: number) => void
  onStopServer?: (mapIndex?: number) => void
  onOpenOptions?: () => void
  onToggleLogs?: () => void
  onImportConfig?: (tomlText: string) => void
  onImportError?: (msg: string) => void
  mapStatuses?: import('../types').MapInstanceStatus[]
  isSaving?: boolean
  autoSave?: boolean
  isServerRunning?: boolean
  isServerStarting?: boolean
  isServerStopping?: boolean
  showLogsButton?: boolean
  isLogsOpen?: boolean
  canUndo?: boolean
  canRedo?: boolean
  onUndo?: () => void
  onRedo?: () => void
  variant?: 'default' | 'mod_settings'
}

export default function ActionBar({
  onSave,
  onReset,
  onBack,
  onChooseDifficulty,
  onStartServer,
  onStopServer,
  onOpenOptions,
  onToggleLogs,
  onImportConfig,
  onImportError,
  isSaving = false,
  autoSave = true,
  isServerStarting = false,
  isServerStopping = false,
  showLogsButton = false,
  isLogsOpen = false,
  canUndo = false,
  canRedo = false,
  onUndo,
  onRedo,
  mapStatuses = [],
  variant = 'default',
}: ActionBarProps) {
  const [saveOk, setSaveOk] = useState(false)
  const [startMenuOpen, setStartMenuOpen] = useState(false)
  const [stopMenuOpen, setStopMenuOpen] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const startBtnRef = useRef<HTMLDivElement>(null)
  const stopBtnRef = useRef<HTMLDivElement>(null)
  const { tk, tip } = useI18n()

  useEffect(() => {
    if (!startMenuOpen && !stopMenuOpen) return
    const close = () => { setStartMenuOpen(false); setStopMenuOpen(false) }
    document.addEventListener('mousedown', close)
    return () => document.removeEventListener('mousedown', close)
  }, [startMenuOpen, stopMenuOpen])

  const handleSave = async () => {
    if (!onSave) return
    try {
      await onSave()
      setSaveOk(true)
      setTimeout(() => setSaveOk(false), 1800)
    } catch { /* error shown in App */ }
  }

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file || !onImportConfig) return
    const isZip = file.name.toLowerCase().endsWith('.zip')
    if (isZip) {
      const reader = new FileReader()
      reader.onload = async (ev) => {
        const buf = ev.target?.result
        if (!(buf instanceof ArrayBuffer)) return
        const bytes = Array.from(new Uint8Array(buf))
        try {
          const config = await invoke<ServerConfig>('parse_config_from_zip', { zipData: bytes })
          // Re-serialize to TOML is not needed — pass config JSON through a round-trip
          // by serializing the parsed config back to TOML via Rust
          const tomlText = await invoke<string>('config_to_toml', { config })
          onImportConfig(tomlText)
        } catch (err) {
          onImportError?.(`Failed to extract config from zip: ${String(err)}`)
        }
      }
      reader.readAsArrayBuffer(file)
    } else {
      const reader = new FileReader()
      reader.onload = (ev) => {
        const text = ev.target?.result
        if (typeof text === 'string') onImportConfig(text)
      }
      reader.readAsText(file)
    }
    e.target.value = ''
  }

  const startDisabled = isServerStarting
  const stopDisabled  = isServerStopping

  const runningMaps = mapStatuses.filter((s) => s.running)
  const stoppedMaps = mapStatuses.filter((s) => !s.running)
  const isCluster = mapStatuses.length > 1
  const anyRunning = runningMaps.length > 0
  const anyStopped = stoppedMaps.length > 0

  return (
    <div className="fixed bottom-0 left-0 right-0 z-40 ark-panel border-t border-ark-cyan/40 px-6 py-3 flex items-center justify-between">

      {/* ── Left: action buttons ──────────────────────────────────────────── */}
      <div className="flex items-center gap-2">

        <Tooltip content={tip('back') ?? ''}>
          <button onClick={onBack} className="ark-action-btn">
            {tk('back', 'ATRÁS')}
          </button>
        </Tooltip>

        {variant === 'mod_settings' ? (
          <>
            <Tooltip content={tip('update') ?? ''}>
              <button className="ark-action-btn">{tk('update', 'ACTUALIZAR')}</button>
            </Tooltip>
            <Tooltip content={tip('get_mods') ?? ''}>
              <button className="ark-action-btn">{tk('get_mods', 'OBTENER MODS')}</button>
            </Tooltip>
            <Tooltip content={tip('deactivate_mod') ?? ''}>
              <button className="ark-action-btn">{tk('deactivate_mod', 'DESACTIVAR MOD')}</button>
            </Tooltip>
            <Tooltip content={tip('mod_info') ?? ''}>
              <button className="ark-action-btn">{tk('mod_info', 'INFO MOD')}</button>
            </Tooltip>
          </>
        ) : (
          <>
            {onReset && (
              <Tooltip content={tip('restore_defaults') ?? ''}>
                <button onClick={onReset} disabled={isSaving} className="ark-action-btn disabled:opacity-40">
                  {tk('restore_defaults', 'RESTABLECER')}
                </button>
              </Tooltip>
            )}

            <Tooltip content={tip('choose_difficulty') ?? ''}>
              <button onClick={onChooseDifficulty} className="ark-action-btn">
                {tk('choose_difficulty', 'DIFICULTAD')}
              </button>
            </Tooltip>

            {/* Undo / Redo */}
            <Tooltip content={tk('undo_tooltip', 'Undo last change (Ctrl+Z)')}>
              <button
                onClick={onUndo}
                disabled={!canUndo}
                className="ark-action-btn text-[10px] px-2 disabled:opacity-25"
                style={{ minWidth: 28 }}
              >
                ↩
              </button>
            </Tooltip>
            <Tooltip content={tk('redo_tooltip', 'Redo change (Ctrl+Y)')}>
              <button
                onClick={onRedo}
                disabled={!canRedo}
                className="ark-action-btn text-[10px] px-2 disabled:opacity-25"
                style={{ minWidth: 28 }}
              >
                ↪
              </button>
            </Tooltip>

            {onSave && (
              autoSave ? (
                <Tooltip content={tip('autosave') ?? ''}>
                  <span
                    className="ark-action-btn text-[10px] cursor-default"
                    style={{ color: 'rgba(0,200,255,0.3)', outlineColor: 'transparent' }}
                  >
                    {tk('autosave', '✦ AUTOGUARDADO')}
                  </span>
                </Tooltip>
              ) : (
                <Tooltip content={tip('save_settings') ?? ''}>
                  <button
                    onClick={handleSave}
                    disabled={isSaving}
                    className="ark-action-btn disabled:opacity-40"
                    style={saveOk ? { color: 'rgba(74,222,128,0.9)', outlineColor: 'rgba(74,222,128,0.4)' } : undefined}
                  >
                    {isSaving ? tk('saving', 'GUARDANDO...') : saveOk ? tk('saved', 'GUARDADO ✓') : tk('save_settings', 'GUARDAR')}
                  </button>
                </Tooltip>
              )
            )}
          </>
        )}

        {/* Import config */}
        {onImportConfig && (
          <>
            <input ref={fileInputRef} type="file" accept=".toml,.zip" className="hidden" onChange={handleFileSelect} />
            <Tooltip content={tip('import_config') ?? ''}>
              <button
                onClick={() => fileInputRef.current?.click()}
                className="ark-action-btn text-[10px] px-3"
                style={{ color: 'rgba(0,200,255,0.5)' }}
              >
                {tk('import_config', '↑ IMPORTAR')}
              </button>
            </Tooltip>
          </>
        )}

        {/* Options */}
        <Tooltip content={tip('options') ?? ''}>
          <button
            onClick={onOpenOptions}
            className="ark-action-btn text-[10px] px-3"
            style={{ color: 'rgba(0,200,255,0.5)' }}
          >
            {tk('options', '⚙ OPCIONES')}
          </button>
        </Tooltip>

        {/* Logs */}
        {showLogsButton && (
          <Tooltip content={tip('logs') ?? ''}>
            <button
              onClick={onToggleLogs}
              className="ark-action-btn text-[10px] px-3"
              style={{
                color: isLogsOpen ? 'rgba(74,222,128,0.8)' : 'rgba(0,200,255,0.5)',
                outlineColor: isLogsOpen ? 'rgba(74,222,128,0.4)' : undefined,
              }}
            >
              {tk('logs', '📋 LOGS')}
            </button>
          </Tooltip>
        )}
      </div>

      {/* ── Right: server control ─────────────────────────────────────────── */}
      <div className="flex items-center gap-3">
        <span className="text-ark-cyan/45 text-[9px] font-bold tracking-widest uppercase italic">
          - DEDICATED SERVER -
        </span>

        {/* Start — show when stopped instances exist */}
        {anyStopped && (
          <div ref={startBtnRef} className="relative flex">
            <Tooltip content={tip('start_server') ?? ''}>
              <button
                onClick={() => onStartServer?.()}
                disabled={startDisabled}
                className="ark-action-btn ark-action-btn-amber-active px-5 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
                style={{ borderRight: isCluster ? 'none' : undefined, borderRadius: isCluster ? '4px 0 0 4px' : undefined }}
              >
                {isServerStarting
                  ? tk('starting', '▶ INICIANDO...')
                  : isCluster && anyRunning
                    ? tk('start_all_stopped', '▶ INICIAR DETENIDOS')
                    : tk('start_server', '▶ SERVIDOR DEDICADO')}
              </button>
            </Tooltip>
            {isCluster && (
              <button
                onClick={(e) => { e.stopPropagation(); setStartMenuOpen((o) => !o); setStopMenuOpen(false) }}
                disabled={startDisabled}
                className="ark-action-btn ark-action-btn-amber-active px-2 py-2 disabled:opacity-50"
                style={{ borderLeft: '1px solid rgba(251,191,36,0.25)', borderRadius: '0 4px 4px 0' }}
                aria-label="Start menu"
              >
                ▼
              </button>
            )}
            <DropdownPortal anchorRef={startBtnRef} open={startMenuOpen}>
              <div style={{ border: '1px solid rgba(0,200,255,0.25)' }}>
                {stoppedMaps.map((m) => (
                  <button
                    key={m.map_index}
                    className="w-full text-left px-4 py-2 text-xs text-ark-cyan/80 hover:bg-ark-cyan/10"
                    onClick={() => {
                      setStartMenuOpen(false)
                      onStartServer?.(m.map_index)
                    }}
                  >
                    ▶ {m.map_label}
                  </button>
                ))}
              </div>
            </DropdownPortal>
          </div>
        )}

        {/* Stop — show when any instance is running */}
        {anyRunning && (
          <div ref={stopBtnRef} className="relative flex">
            <Tooltip content={tip('stop_server') ?? ''}>
              <button
                onClick={() => onStopServer?.()}
                disabled={stopDisabled}
                className="ark-action-btn px-5 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
                style={{
                  background:  stopDisabled ? 'rgba(239,68,68,0.07)' : 'rgba(239,68,68,0.15)',
                  color:       'rgba(239,68,68,0.9)',
                  outlineColor:'rgba(239,68,68,0.6)',
                  boxShadow:   stopDisabled ? 'none' : '0 0 12px rgba(239,68,68,0.3)',
                  borderRight: isCluster ? 'none' : undefined,
                  borderRadius: isCluster ? '4px 0 0 4px' : undefined,
                }}
              >
                {isServerStopping
                  ? tk('stopping', '■ DETENIENDO...')
                  : tk('stop_server', '■ DETENER SERVIDOR')}
              </button>
            </Tooltip>
            {isCluster && (
              <button
                onClick={(e) => { e.stopPropagation(); setStopMenuOpen((o) => !o); setStartMenuOpen(false) }}
                disabled={stopDisabled}
                className="ark-action-btn px-2 py-2 disabled:opacity-50"
                style={{
                  background: stopDisabled ? 'rgba(239,68,68,0.07)' : 'rgba(239,68,68,0.15)',
                  color: 'rgba(239,68,68,0.9)',
                  outlineColor: 'rgba(239,68,68,0.6)',
                  borderLeft: '1px solid rgba(239,68,68,0.35)',
                  borderRadius: '0 4px 4px 0',
                }}
                aria-label="Stop menu"
              >
                ▼
              </button>
            )}
            <DropdownPortal anchorRef={stopBtnRef} open={stopMenuOpen}>
              <div style={{ border: '1px solid rgba(239,68,68,0.25)' }}>
                {runningMaps.map((m) => (
                  <button
                    key={m.map_index}
                    className="w-full text-left px-4 py-2 text-xs text-red-300/80 hover:bg-red-500/10"
                    onClick={() => {
                      setStopMenuOpen(false)
                      onStopServer?.(m.map_index)
                    }}
                  >
                    ■ {m.map_label}
                  </button>
                ))}
              </div>
            </DropdownPortal>
          </div>
        )}
      </div>
    </div>
  )
}
