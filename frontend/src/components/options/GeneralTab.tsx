import { useBackupStore, type BackupScope } from '../../stores/backupStore'
import { useConfigStore, type ConfigStore } from '../../stores/configStore'
import { useShallow } from 'zustand/react/shallow'
import { useI18n } from '../../i18n/useI18n'
import { Section, Toggle } from '../ui/OptionsUI'

const SCOPE_VALUES: BackupScope[] = ['map', 'map_players_tribes', 'full']

const LANGUAGES: { code: string; label: string; native: string }[] = [
  { code: 'es', label: 'Español',            native: 'Español' },
  { code: 'en', label: 'English',            native: 'English' },
  { code: 'fr', label: 'Français',           native: 'Français' },
  { code: 'zh', label: 'Simplified Chinese', native: '简体中文' },
  { code: 'ja', label: 'Japanese',           native: '日本語' },
  { code: 'ko', label: 'Korean',             native: '한국어' },
  { code: 'pt', label: 'Português',          native: 'Português' },
  { code: 'de', label: 'Deutsch',            native: 'Deutsch' },
  { code: 'it', label: 'Italiano',           native: 'Italiano' },
  { code: 'ru', label: 'Russian',           native: 'Русский' },
]

export default function GeneralTab() {
  const store = useBackupStore()
  const cfg = useConfigStore(
    useShallow((s: ConfigStore) => ({
      network: s.config?.network,
      updateNetwork: (patch: Partial<NonNullable<typeof s.config>['network']>) => {
        if (!s.config) return
        s.setConfig({ ...s.config, network: { ...s.config.network, ...patch } })
      },
    })),
  )
  const { tk } = useI18n()

  return (
    <>
      <Section title={tk('section_language', 'Language')}>
        <div className="grid grid-cols-2 gap-2">
          {LANGUAGES.map((lang) => (
            <button
              key={lang.code}
              onClick={() => store.setLanguage(lang.code)}
              className="rounded-md p-2.5 text-left transition-all flex items-center gap-2"
              style={{
                background: store.language === lang.code ? 'rgba(0,200,255,0.1)' : 'rgba(255,255,255,0.03)',
                border: `1px solid ${store.language === lang.code ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.07)'}`,
              }}
            >
              <span className="text-xs font-semibold" style={{ color: store.language === lang.code ? 'rgba(0,200,255,0.9)' : 'rgba(255,255,255,0.45)' }}>
                {lang.native}
              </span>
              {lang.native !== lang.label && (
                <span className="text-[10px]" style={{ color: store.language === lang.code ? 'rgba(0,200,255,0.5)' : 'rgba(255,255,255,0.2)' }}>
                  {lang.label}
                </span>
              )}
            </button>
          ))}
        </div>
        <p className="text-ark-cyan/30 text-[10px] mt-1">
          {tk('lang_pref_note', 'Language preference is saved. Translations are applied progressively.')}
        </p>
      </Section>

      <Section title={tk('section_on_demand', 'On-Demand Server')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('on_demand_title', 'Enable sleep mode')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('on_demand_desc', 'Lets individual maps appear in the ARK browser without the server running.')}</p>
          </div>
          <Toggle value={store.onDemandEnabled} onChange={store.setOnDemandEnabled} />
        </div>
      </Section>

      {!store.onDemandEnabled && (
        <Section title={tk('section_cluster', 'Server Cluster')}>
          <div className="space-y-4">
            <div>
              <p className="text-ark-cyan/80 text-sm">{tk('cluster_delay_title', 'Delay between cluster instances')}</p>
              <p className="text-ark-cyan/40 text-xs mt-0.5 mb-3">{tk('cluster_delay_desc', 'Wait time between each map startup in a cluster.')}</p>
              <div className="flex items-center gap-4">
                <input
                  type="range"
                  min={0}
                  max={180}
                  step={5}
                  value={store.clusterStartDelaySec}
                  onChange={(e) => store.setClusterStartDelaySec(Number(e.target.value))}
                  className="flex-1 accent-ark-cyan"
                />
                <span className="text-ark-cyan/80 font-mono text-sm w-20 text-right">
                  {store.clusterStartDelaySec === 0 ? tk('no_delay', 'No delay') : `${store.clusterStartDelaySec} s`}
                </span>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-ark-cyan/80 text-sm">{tk('fixed_ports_title', 'Fixed port assignment per map (Recommended)')}</p>
                <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('fixed_ports_desc', 'Each cluster map always lands on the same ports.')}</p>
              </div>
              <Toggle
                value={cfg.network?.fixed_port_assignment_per_map ?? true}
                onChange={(v) => cfg.updateNetwork({ fixed_port_assignment_per_map: v })}
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-ark-cyan/80 text-sm">{tk('no_battleye_title', 'Disable BattleEye anti-cheat')}</p>
                <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('no_battleye_desc', 'Adds -NoBattlEye to launch args.')}</p>
              </div>
              <Toggle
                value={cfg.network?.no_battleye ?? false}
                onChange={(v) => cfg.updateNetwork({ no_battleye: v })}
              />
            </div>
          </div>
        </Section>
      )}

      {/* ── Internet gate (v2.1) ────────────────────────────────────────── */}
      <Section title={tk('section_internet', 'Internet')}>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-ark-cyan/80 text-sm">{tk('allow_start_offline_title', 'Allow start without internet')}</p>
              <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('allow_start_offline_desc', 'Skip the offline check and let the server launch anyway.')}</p>
            </div>
            <Toggle
              value={cfg.network?.allow_start_without_internet ?? false}
              onChange={(v) => cfg.updateNetwork({ allow_start_without_internet: v })}
            />
          </div>
        </div>
      </Section>

      <Section title={tk('section_close_behavior', 'Close Behavior')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('minimize_tray_title', 'Minimize to system tray')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('minimize_tray_desc', 'When closing the window, the app minimizes to the system tray.')}</p>
          </div>
          <Toggle value={store.minimizeToTray} onChange={store.setMinimizeToTray} />
        </div>
      </Section>

      <Section title={tk('section_save', 'Save Settings')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('manual_save_title', 'Manual save')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('manual_save_desc', 'By default changes are saved automatically. Enable this to save only when you press SAVE SETTINGS.')}</p>
          </div>
          <Toggle value={store.manualSave} onChange={store.setManualSave} />
        </div>
      </Section>

      <Section title={tk('section_logs', 'Log Viewer')}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-ark-cyan/80 text-sm">{tk('logs_btn_title', 'Server logs button')}</p>
            <p className="text-ark-cyan/40 text-xs mt-0.5">{tk('logs_btn_desc', 'Shows the LOGS button in the bottom bar to view ShooterGame.log in real time')}</p>
          </div>
          <Toggle value={store.logsEnabled} onChange={store.setLogsEnabled} />
        </div>
      </Section>

      <Section title={tk('section_backup_scope', 'Backup Scope')}>
        <div className="space-y-2">
          {SCOPE_VALUES.map((val) => {
            const labelKey = val === 'map' ? 'scope_map_label' : val === 'map_players_tribes' ? 'scope_map_players_label' : 'scope_full_label'
            const descKey  = val === 'map' ? 'scope_map_desc'  : val === 'map_players_tribes' ? 'scope_map_players_desc'  : 'scope_full_desc'
            return (
              <label
                key={val}
                className="flex items-center gap-3 cursor-pointer p-2.5 rounded-md transition-colors"
                style={{
                  background: store.backupScope === val ? 'rgba(0,200,255,0.08)' : 'transparent',
                  border: `1px solid ${store.backupScope === val ? 'rgba(0,200,255,0.4)' : 'rgba(255,255,255,0.06)'}`,
                }}
              >
                <input
                  type="radio"
                  name="scope"
                  value={val}
                  checked={store.backupScope === val}
                  onChange={() => store.setBackupScope(val)}
                  className="accent-ark-cyan"
                />
                <div>
                  <p className="text-ark-cyan/80 text-sm font-semibold">{tk(labelKey, val)}</p>
                  <p className="text-ark-cyan/40 text-xs">{tk(descKey, '')}</p>
                </div>
              </label>
            )
          })}
        </div>
      </Section>

      <Section title={tk('section_saves_to_keep', 'Saves to Keep')}>
        <div className="flex items-center gap-4">
          <input
            type="range"
            min={1}
            max={10}
            value={store.maxSaves}
            onChange={(e) => store.setMaxSaves(Number(e.target.value))}
            className="flex-1 accent-ark-cyan"
          />
          <span className="text-ark-cyan/80 font-mono text-sm w-16 text-right">
            {store.maxSaves === 1 ? tk('only_last_save', 'Only the last') : tk('last_n_saves', 'Last {{n}}').replace('{{n}}', String(store.maxSaves))}
          </span>
        </div>
      </Section>
    </>
  )
}
