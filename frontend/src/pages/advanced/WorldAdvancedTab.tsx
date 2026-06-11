import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface WorldAdvancedTabProps {
  config: ServerConfig
}

export default function WorldAdvancedTab({ config }: WorldAdvancedTabProps) {
  const updateWorld = useConfigUpdate('world')
  const updateMult = useConfigUpdate('multipliers')

  const settings = useMemo(
    () => [
      {
        label: 'Day Cycle Speed',
        value: config.world.day_cycle_speed_scale,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('day_cycle_speed_scale', v),
        step: 0.01,
      },
      {
        label: 'Day Time Speed',
        value: config.world.day_time_speed_scale,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('day_time_speed_scale', v),
        step: 0.01,
      },
      {
        label: 'Night Time Speed',
        value: config.world.night_time_speed_scale,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('night_time_speed_scale', v),
        step: 0.01,
      },
      {
        label: 'Spoiling Time',
        value: config.world.global_spoiling_time_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('global_spoiling_time_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Item Decomposition',
        value: config.world.global_item_decomposition_time_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('global_item_decomposition_time_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Corpse Decomposition',
        value: config.world.global_corpse_decomposition_time_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('global_corpse_decomposition_time_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Crop Growth Speed',
        value: config.world.crop_growth_speed_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('crop_growth_speed_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Crop Decay Speed',
        value: config.world.crop_decay_speed_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateWorld('crop_decay_speed_multiplier', v),
        step: 0.1,
      },
    ],
    [config.world, config.multipliers, updateWorld, updateMult]
  )

  return (
    <SettingsPanel title="WORLD ADVANCED SETTINGS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          step={setting.step}
          testId={`world-adv-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
