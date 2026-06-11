import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface StructureTabProps {
  config: ServerConfig
}

export default function StructureTab({ config }: StructureTabProps) {
  const updateMult = useConfigUpdate('multipliers')
  const updateAdvanced = useConfigUpdate('advanced')

  const settings = useMemo(
    () => [
      {
        label: 'Damage Multiplier',
        value: config.multipliers.structure_damage_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('structure_damage_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Resistance Multiplier',
        value: config.multipliers.structure_resistance_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updateMult('structure_resistance_multiplier', v),
        step: 0.1,
      },
      {
        label: 'Disable Placement Collision',
        value: config.advanced.disable_structure_placement_collision,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateAdvanced('disable_structure_placement_collision', v),
      },
      {
        label: 'Repair Cooldown',
        value: config.pvp.structure_damage_repair_cooldown,
        type: 'number' as const,
        onChange: (v: number) => {
          const updatePvp = useConfigUpdate('pvp')
          updatePvp('structure_damage_repair_cooldown', v as any)
        },
        step: 1,
      },
    ],
    [config.multipliers, config.advanced, config.pvp, updateMult, updateAdvanced]
  )

  return (
    <SettingsPanel title="STRUCTURE SETTINGS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          step={setting.step}
          testId={`structure-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
