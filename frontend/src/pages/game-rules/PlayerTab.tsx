import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface PlayerTabProps {
  config: ServerConfig
}

export default function PlayerTab({ config }: PlayerTabProps) {
  const updateMult = useConfigUpdate('multipliers')

  const settings = useMemo(
    () => [
      {
        label: 'Damage Multiplier',
        value: config.multipliers.player_damage_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('player_damage_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Resistance Multiplier',
        value: config.multipliers.player_resistance_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('player_resistance_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Water Drain',
        value: config.multipliers.player_character_water_drain_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('player_character_water_drain_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Food Drain',
        value: config.multipliers.player_character_food_drain_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('player_character_food_drain_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Stamina Drain',
        value: config.multipliers.player_character_stamina_drain_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('player_character_stamina_drain_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Health Recovery',
        value: config.multipliers.player_character_health_recovery_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('player_character_health_recovery_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Harvesting Damage',
        value: config.multipliers.harvest_amount_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('harvest_amount_multiplier', v),
        step: 0.1,
      },
    ],
    [config.multipliers, updateMult]
  )

  return (
    <SettingsPanel title="PLAYER SETTINGS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          step={setting.step}
          testId={`player-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
