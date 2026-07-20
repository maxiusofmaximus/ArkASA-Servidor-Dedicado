import { useState, useRef } from 'react'
import { useConfigStore, type ConfigStore } from '../stores/configStore'
import { useShallow } from 'zustand/react/shallow'
import { useI18n } from '../i18n/useI18n'
import { useBackupActions } from '../hooks/useBackupActions'
import { invoke } from '../services/tauri'
import GeneralTab from './options/GeneralTab'
import BackupTab from './options/BackupTab'
import ConfigTab from './options/ConfigTab'
import HostingTab from './options/HostingTab'
import DatabaseTab from './options/DatabaseTab'
import PluginsTab from './options/PluginsTab'
import ModsTab from './options/ModsTab'
import DiagnosticsTab from './options/DiagnosticsTab'
import type { ServerConfig } from '../types'

interface OptionsModalProps {
  onClose: () => void
  onReset?: () => void
  onChooseDifficulty?: () => void
  onImportConfig?: (tomlText: string) => void
  onImportError?: (msg: string) => void
  onToggleLogs?: () => void
  isLogsOpen?: boolean
  isSaving?: boolean
}

type OptionsTab = 'general' | 'backup' | 'config' | 'hosting' | 'database' | 'diagnostics' | 'plugins' | 'mods' | 'actions'

export default function OptionsModal({
  onClose,
  onReset,
  onChooseDifficulty,
  onImportConfig,
  onImportError,
  onToggleLogs,
  isLogsOpen = false,
  isSaving = false,
}: OptionsModalProps) {
  const [tab, setTab] = useState<OptionsTab>('general')
  const { config, setConfig, setSavedConfig } = useConfigStore(useShallow((s: ConfigStore) => ({ config: s.config, setConfig: s.setConfig, setSavedConfig: s.setSavedConfig })))
  const { tk } = useI18n()
  const backupActions = useBackupActions(config)
  const fileInputRef = useRef<HTMLInputElement>(null)

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
          const parsed = await invoke<ServerConfig>('parse_config_from_zip', { zipData: bytes })
          const tomlText = await invoke<string>('config_to_toml', { config: parsed })
          onImportConfig(tomlText)
          onClose()
        } catch (err) {
          onImportError?.(`Failed to extract config from zip: ${String(err)}`)
        }
      }
      reader.readAsArrayBuffer(file)
    } else {
      const reader = new FileReader()
      reader.onload = (ev) => {
        const text = ev.target?.result
        if (typeof text === 'string') {
          onImportConfig(text)
          onClose()
        }
      }
      reader.readAsText(file)
    }
    e.target.value = ''
  }

  const tabs: OptionsTab[] = ['general', 'backup', 'config', 'hosting', 'database', 'diagnostics', 'plugins', 'mods', 'actions']

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: 'rgba(0,0,0,0.75)', backdropFilter: 'blur(4px)' }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        className={`ark-panel rounded-xl w-full max-h-[90vh] flex flex-col transition-all ${(tab === 'config' || tab === 'mods') ? 'max-w-4xl' : 'max-w-2xl'}`}
        style={{ border: '1px solid rgba(0,200,255,0.3)', boxShadow: '0 0 40px rgba(0,200,255,0.1)' }}
      >
        <div className="flex items-center justify-between px-6 pt-5 pb-4 border-b border-ark-cyan/20 flex-shrink-0">
          <span className="text-ark-cyan font-bold tracking-widest text-sm uppercase">{tk('options_title', '⚙ Options')}</span>
          <button
            onClick={onClose}
            className="text-ark-cyan/40 hover:text-ark-cyan/80 text-xs tracking-widest transition-colors"
          >
            {tk('esc_close', 'ESC / CLOSE')}
          </button>
        </div>

        <div
          className="flex flex-wrap border-b border-ark-cyan/20 flex-shrink-0"
          role="tablist"
          aria-label={tk('options_title', 'Options')}
        >
          {tabs.map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className="shrink-0 whitespace-nowrap px-5 py-3 text-xs font-bold tracking-widest uppercase transition-colors"
              role="tab"
              aria-selected={tab === t}
              style={{
                color: tab === t ? 'rgba(0,200,255,0.9)' : 'rgba(0,200,255,0.35)',
                borderBottom: tab === t ? '2px solid rgba(0,200,255,0.8)' : '2px solid transparent',
              }}
            >
              {t === 'general' ? tk('tab_general', 'General')
                : t === 'backup' ? tk('tab_backup', 'Backup')
                : t === 'config' ? tk('tab_config_ini', 'Config INI')
                : t === 'hosting' ? tk('tab_hosting', 'Hosting')
                : t === 'database' ? tk('tab_database', 'Database')
                : t === 'diagnostics' ? tk('tab_diagnostics', 'Diagnostics')
                : t === 'plugins' ? tk('tab_plugins', 'Plugins')
                : t === 'mods' ? tk('tab_mods', 'Mods')
                : tk('tab_actions', 'Actions')}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto p-6 space-y-5">
          {tab === 'general' && <GeneralTab />}
          {tab === 'backup' && <BackupTab actions={backupActions} />}
          {tab === 'config' && (
            <ConfigTab
              config={config}
              onConfigSaved={(updated) => {
                setConfig(updated)
                setSavedConfig(updated)
              }}
            />
          )}
          {tab === 'hosting' && <HostingTab />}
          {tab === 'database' && <DatabaseTab />}
          {tab === 'diagnostics' && <DiagnosticsTab config={config} />}
          {tab === 'plugins' && (
            <PluginsTab config={config} />
          )}
          {tab === 'mods' && (
            <ModsTab
              config={config}
              onConfigSaved={(updated) => {
                setConfig(updated)
                setSavedConfig(updated)
              }}
              onRequestSwitchToConfigTab={() => setTab('config')}
            />
          )}
          {tab === 'actions' && (
            <div className="space-y-3">
              {onReset && (
                <button
                  onClick={() => { onReset(); onClose() }}
                  disabled={isSaving}
                  className="w-full text-left rounded-md px-4 py-3 transition-colors disabled:opacity-40"
                  style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}
                >
                  <p className="text-ark-cyan/80 text-sm font-semibold">{tk('restore_defaults', 'RESTORE DEFAULTS')}</p>
                  <p className="text-ark-cyan/35 text-xs">{tk('restore_defaults_desc', 'Reset all settings to their default values')}</p>
                </button>
              )}
              {onChooseDifficulty && (
                <button
                  onClick={() => { onChooseDifficulty(); onClose() }}
                  className="w-full text-left rounded-md px-4 py-3 transition-colors"
                  style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}
                >
                  <p className="text-ark-cyan/80 text-sm font-semibold">{tk('choose_difficulty', 'CHOOSE DIFFICULTY')}</p>
                  <p className="text-ark-cyan/35 text-xs">{tk('choose_difficulty_desc', 'Set the override official difficulty')}</p>
                </button>
              )}
              {onImportConfig && (
                <>
                  <input ref={fileInputRef} type="file" accept=".toml,.zip" className="hidden" onChange={handleFileSelect} />
                  <button
                    onClick={() => fileInputRef.current?.click()}
                    className="w-full text-left rounded-md px-4 py-3 transition-colors"
                    style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}
                  >
                    <p className="text-ark-cyan/80 text-sm font-semibold">{tk('import_config', '↑ IMPORT')}</p>
                    <p className="text-ark-cyan/35 text-xs">{tk('import_desc', 'Import configuration from .toml or backup .zip file')}</p>
                  </button>
                </>
              )}
              {onToggleLogs && (
                <button
                  onClick={onToggleLogs}
                  className="w-full text-left rounded-md px-4 py-3 transition-colors"
                  style={{
                    background: isLogsOpen ? 'rgba(74,222,128,0.06)' : 'rgba(255,255,255,0.03)',
                    border: isLogsOpen ? '1px solid rgba(74,222,128,0.25)' : '1px solid rgba(255,255,255,0.08)',
                  }}
                >
                  <p className="text-sm font-semibold" style={{ color: isLogsOpen ? 'rgba(74,222,128,0.8)' : 'rgba(0,212,255,0.8)' }}>
                    {isLogsOpen ? tk('hide_logs', '◂ Hide Logs') : tk('show_logs', 'Show Logs ▸')}
                  </p>
                  <p className="text-ark-cyan/35 text-xs">{tk('logs_desc', 'Toggle server logs panel')}</p>
                </button>
              )}
            </div>
          )}
        </div>

        <div className="px-6 py-4 border-t border-ark-cyan/20 flex justify-end flex-shrink-0">
          <button onClick={onClose} className="ark-action-btn px-6 py-2 text-xs tracking-widest">
            {tk('close', 'CLOSE')}
          </button>
        </div>
      </div>
    </div>
  )
}
