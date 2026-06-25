import { useState } from 'react'
import { useConfigStore, type ConfigStore } from '../stores/configStore'
import { useShallow } from 'zustand/react/shallow'
import { useI18n } from '../i18n/useI18n'
import { useBackupActions } from '../hooks/useBackupActions'
import GeneralTab from './options/GeneralTab'
import BackupTab from './options/BackupTab'
import ConfigTab from './options/ConfigTab'

interface OptionsModalProps {
  onClose: () => void
}

type OptionsTab = 'general' | 'backup' | 'config'

export default function OptionsModal({ onClose }: OptionsModalProps) {
  const [tab, setTab] = useState<OptionsTab>('general')
  const { config, setConfig, setSavedConfig } = useConfigStore(useShallow((s: ConfigStore) => ({ config: s.config, setConfig: s.setConfig, setSavedConfig: s.setSavedConfig })))
  const { tk } = useI18n()
  const backupActions = useBackupActions(config)

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: 'rgba(0,0,0,0.75)', backdropFilter: 'blur(4px)' }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        className={`ark-panel rounded-xl w-full max-h-[90vh] flex flex-col transition-all ${tab === 'config' ? 'max-w-4xl' : 'max-w-2xl'}`}
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

        <div className="flex border-b border-ark-cyan/20 flex-shrink-0">
          {(['general', 'backup', 'config'] as OptionsTab[]).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className="px-6 py-3 text-xs font-bold tracking-widest uppercase transition-colors"
              style={{
                color: tab === t ? 'rgba(0,200,255,0.9)' : 'rgba(0,200,255,0.35)',
                borderBottom: tab === t ? '2px solid rgba(0,200,255,0.8)' : '2px solid transparent',
              }}
            >
              {t === 'general' ? tk('tab_general', 'General') : t === 'backup' ? tk('tab_backup', 'Backup') : tk('tab_config_ini', 'Config INI')}
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
