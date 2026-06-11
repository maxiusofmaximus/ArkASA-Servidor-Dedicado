import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface PvpTabProps {
  config: ServerConfig
}

export default function PvpTab({ config }: PvpTabProps) {
  const updatePvp = useConfigUpdate('pvp')

  const settings = useMemo(
    () => [
      {
        label: 'PVP Dino Decay',
        value: config.pvp.pvp_dino_decay,
        type: 'boolean' as const,
        onChange: (v: boolean) => updatePvp('pvp_dino_decay', v),
      },
      {
        label: 'Zone Structure Damage',
        value: config.pvp.pvp_zone_structure_damage_multiplier,
        type: 'number' as const,
        onChange: (v: number) => updatePvp('pvp_zone_structure_damage_multiplier', v),
        step: 0.1,
      },
    ],
    [config.pvp, updatePvp]
  )

  return (
    <SettingsPanel title="PVP SETTINGS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          step={setting.step}
          testId={`pvp-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
