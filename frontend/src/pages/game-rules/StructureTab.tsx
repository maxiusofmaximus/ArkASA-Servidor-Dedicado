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
        label: 'Limit Generators Num',
        value: config.advanced.limit_generators_num,
        type: 'number' as const,
        onChange: (v: number) => updateAdvanced('limit_generators_num', v),
        step: 1,
        min: 0,
      },
      {
        label: 'Limit Generators Range',
        value: config.advanced.limit_generators_range,
        type: 'number' as const,
        onChange: (v: number) => updateAdvanced('limit_generators_range', v),
        step: 100,
        min: 0,
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
          min={(setting as { min?: number }).min}
          testId={`structure-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
