import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface WorldRulesTabProps {
  config: ServerConfig
}

export default function WorldRulesTab({ config }: WorldRulesTabProps) {
  const updateMult = useConfigUpdate('multipliers')
  const updateGameplay = useConfigUpdate('gameplay')

  const settings = useMemo(
    () => [
      {
        label: 'XP Multiplier',
        value: config.multipliers.xp_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('xp_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Taming Speed',
        value: config.multipliers.taming_speed_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('taming_speed_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Harvest Yield',
        value: config.multipliers.harvest_amount_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('harvest_amount_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Allow Speed Leveling',
        value: config.gameplay.allow_speed_leveling,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('allow_speed_leveling', v),
      },
      {
        label: 'Allow Flyer Speed Leveling',
        value: config.gameplay.allow_flyer_speed_leveling,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('allow_flyer_speed_leveling', v),
      },
      {
        label: 'Maximum Difficulty',
        value: config.gameplay.override_official_difficulty,
        type: 'number' as const,
        onChange: (v: number) => updateGameplay('override_official_difficulty', v),
        step: 0.1,
      },
      {
        label: 'PVE Mode',
        value: config.gameplay.server_pve,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('server_pve', v),
      },
      {
        label: 'Hardcore Mode',
        value: config.gameplay.server_hardcore,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('server_hardcore', v),
      },
    ],
    [config.multipliers, config.gameplay, updateMult, updateGameplay]
  )

  return (
    <SettingsPanel title="WORLD RULES">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          step={setting.step}
          testId={`world-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
