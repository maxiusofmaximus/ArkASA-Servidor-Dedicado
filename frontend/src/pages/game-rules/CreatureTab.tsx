import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface CreatureTabProps {
  config: ServerConfig
}

export default function CreatureTab({ config }: CreatureTabProps) {
  const updateMult = useConfigUpdate('multipliers')
  const updateGameplay = useConfigUpdate('gameplay')
  const updateAdvanced = useConfigUpdate('advanced')

  const settings = useMemo(
    () => [
      {
        label: 'Max Count',
        value: config.gameplay.dino_count_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateGameplay('dino_count_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Damage Multiplier',
        value: config.multipliers.dino_damage_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('dino_damage_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Resistance Multiplier',
        value: config.multipliers.dino_resistance_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('dino_resistance_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Food Drain',
        value: config.multipliers.dino_character_food_drain_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('dino_character_food_drain_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Stamina Drain',
        value: config.multipliers.dino_character_stamina_drain_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('dino_character_stamina_drain_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Health Recovery',
        value: config.multipliers.dino_character_health_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('dino_character_health_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Disable Taming',
        value: config.advanced.disable_dino_taming,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateAdvanced('disable_dino_taming', v),
      },
      {
        label: 'Disable Riding',
        value: config.advanced.disable_dino_riding,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateAdvanced('disable_dino_riding', v),
      },
    ],
    [config.gameplay, config.multipliers, config.advanced, updateMult, updateGameplay, updateAdvanced]
  )

  return (
    <SettingsPanel title="CREATURE SETTINGS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          step={setting.step}
          testId={`creature-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
