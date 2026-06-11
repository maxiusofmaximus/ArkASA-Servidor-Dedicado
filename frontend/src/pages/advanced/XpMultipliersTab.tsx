import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface XpMultipliersTabProps {
  config: ServerConfig
}

export default function XpMultipliersTab({ config }: XpMultipliersTabProps) {
  const updateXp = useConfigUpdate('xp_multipliers')

  const settings = useMemo(
    () => [
      { label: 'Generic XP', key: 'generic_xp_multiplier' },
      { label: 'Kill XP', key: 'kill_xp_multiplier' },
      { label: 'Harvest XP', key: 'harvest_xp_multiplier' },
      { label: 'Craft XP', key: 'craft_xp_multiplier' },
      { label: 'Special XP', key: 'special_xp_multiplier' },
      { label: 'Explorer Note XP', key: 'explorer_note_xp_multiplier' },
      { label: 'Boss Kill XP', key: 'boss_kill_xp_multiplier' },
      { label: 'Alpha Kill XP', key: 'alpha_kill_xp_multiplier' },
      { label: 'Wild Kill XP', key: 'wild_kill_xp_multiplier' },
      { label: 'Cave Kill XP', key: 'cave_kill_xp_multiplier' },
      { label: 'Tamed Kill XP', key: 'tamed_kill_xp_multiplier' },
    ],
    []
  )

  return (
    <SettingsPanel title="XP MULTIPLIERS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={config.xp_multipliers[setting.key as keyof typeof config.xp_multipliers]}
          type="number"
          onChange={(v: number) =>
            updateXp(setting.key as keyof typeof config.xp_multipliers, v)
          }
          step={0.1}
          testId={`xp-${setting.key}`}
        />
      ))}
    </SettingsPanel>
  )
}
