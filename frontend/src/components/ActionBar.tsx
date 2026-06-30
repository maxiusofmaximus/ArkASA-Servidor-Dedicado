import React, { useState, useRef, useEffect } from 'react'
import Tooltip from './Tooltip'
import DropdownPortal from './DropdownPortal'
import { useI18n } from '../i18n/useI18n'

interface ActionBarProps {
  onSave?: () => Promise<void> | void
  onStartServer?: (mapIndex?: number) => void
  onStopServer?: (mapIndex?: number) => void
  mapStatuses?: import('../types').MapInstanceStatus[]
  isSaving?: boolean
  autoSave?: boolean
  isServerRunning?: boolean
  isServerStarting?: boolean
  isServerStopping?: boolean
  canUndo?: boolean
  canRedo?: boolean
  onUndo?: () => void
  onRedo?: () => void
  online?: boolean
  variant?: 'default' | 'mod_settings'
}

export default function ActionBar({
  onSave,
  onStartServer,
  onStopServer,
  isSaving = false,
  autoSave = true,
  isServerStarting = false,
  isServerStopping = false,
  canUndo = false,
  canRedo = false,
  onUndo,
  onRedo,
  mapStatuses = [],
  online = true,
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

  const startDisabled = isServerStarting || !online
  const stopDisabled  = isServerStopping

  const runningMaps = mapStatuses.filter((s) => s.running)
  const stoppedMaps = mapStatuses.filter((s) => !s.running)
  const isCluster = mapStatuses.length > 1
  const anyRunning = runningMaps.length > 0
  const anyStopped = stoppedMaps.length > 0

  return (
    <div className="fixed bottom-0 left-0 right-0 z-40 ark-panel border-t border-ark-cyan/40 px-6 py-3 flex items-center justify-between">

      {/* ── Left: Save only ──────────────────────────────────────────── */}
      <div className="flex items-center gap-2">

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
            {onSave && (
              autoSave ? (
                <Tooltip content={tip('autosave') ?? ''}>
                  <span
                    className="ark-action-btn text-[10px] cursor-default"
                    style={{ color: 'rgba(0,200,255,0.3)', outlineColor: 'transparent' }}
                  >
                    {tk('autosave', '✦ AUTOSAVE')}
                  </span>
                </Tooltip>
              ) : (
                <Tooltip content={tip('save') ?? ''}>
                  <button
                    onClick={handleSave}
                    disabled={isSaving}
                    className="ark-action-btn disabled:opacity-40"
                    style={saveOk ? { color: 'rgba(74,222,128,0.9)', outlineColor: 'rgba(74,222,128,0.4)' } : undefined}
                  >
                    {isSaving ? tk('saving', 'SAVING...') : saveOk ? tk('saved', 'SAVED ✓') : tk('save', 'SAVE')}
                  </button>
                </Tooltip>
              )
            )}

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
          </>
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
