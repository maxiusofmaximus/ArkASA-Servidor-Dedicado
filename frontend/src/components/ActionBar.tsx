import React, { useState, useRef } from 'react'
import Tooltip from './Tooltip'
import { useI18n } from '../i18n/useI18n'

interface ActionBarProps {
  onSave?: () => Promise<void> | void
  onReset?: () => void
  onBack?: () => void
  onChooseDifficulty?: () => void
  onStartServer?: () => void
  onStopServer?: () => void
  onOpenOptions?: () => void
  onToggleLogs?: () => void
  onImportConfig?: (tomlText: string) => void
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
  isSaving = false,
  autoSave = true,
  isServerRunning = false,
  isServerStarting = false,
  isServerStopping = false,
  showLogsButton = false,
  isLogsOpen = false,
  canUndo = false,
  canRedo = false,
  onUndo,
  onRedo,
  variant = 'default',
}: ActionBarProps) {
  const [saveOk, setSaveOk] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const { tk, tip } = useI18n()

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
    const reader = new FileReader()
    reader.onload = (ev) => {
      const text = ev.target?.result
      if (typeof text === 'string') onImportConfig(text)
    }
    reader.readAsText(file)
    e.target.value = ''
  }

  const startDisabled = isServerStarting || isServerRunning
  const stopDisabled  = isServerStopping

  return (
    <div className="fixed bottom-0 left-0 right-0 z-20 ark-panel border-t border-ark-cyan/40 px-6 py-3 flex items-center justify-between">

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
            <Tooltip content="Deshacer último cambio (Ctrl+Z)">
              <button
                onClick={onUndo}
                disabled={!canUndo}
                className="ark-action-btn text-[10px] px-2 disabled:opacity-25"
                style={{ minWidth: 28 }}
              >
                ↩
              </button>
            </Tooltip>
            <Tooltip content="Rehacer cambio desecho (Ctrl+Y)">
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
            <input ref={fileInputRef} type="file" accept=".toml" className="hidden" onChange={handleFileSelect} />
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

        {isServerRunning || isServerStopping ? (
          <Tooltip content={tip('stop_server') ?? ''}>
            <button
              onClick={onStopServer}
              disabled={stopDisabled}
              className="ark-action-btn px-5 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
              style={{
                background:  stopDisabled ? 'rgba(239,68,68,0.07)' : 'rgba(239,68,68,0.15)',
                color:       'rgba(239,68,68,0.9)',
                outlineColor:'rgba(239,68,68,0.6)',
                boxShadow:   stopDisabled ? 'none' : '0 0 12px rgba(239,68,68,0.3)',
              }}
            >
              {isServerStopping ? tk('stopping', '■ DETENIENDO...') : tk('stop_server', '■ DETENER SERVIDOR')}
            </button>
          </Tooltip>
        ) : (
          <Tooltip content={tip('start_server') ?? ''}>
            <button
              onClick={onStartServer}
              disabled={startDisabled}
              className="ark-action-btn ark-action-btn-amber-active px-5 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isServerStarting ? tk('starting', '▶ INICIANDO...') : tk('start_server', '▶ SERVIDOR DEDICADO')}
            </button>
          </Tooltip>
        )}
      </div>
    </div>
  )
}
