import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface WildDinoTabProps {
  config: ServerConfig
}

const statLabels = ['Health', 'Stamina', 'Oxygen', 'Food', 'Water', 'Weight', 'Melee Damage', 'Speed', 'Fortitude', 'Torpidity']
const statKeys = ['health', 'stamina', 'oxygen', 'food', 'water', 'weight', 'melee_damage', 'speed', 'fortitude', 'torpidity'] as const

export default function WildDinoTab({ config }: WildDinoTabProps) {
  const updateDinoWild = useConfigUpdate('dino_wild_stats')

  const settings = useMemo(
    () =>
      statLabels.map((label, i) => ({
        label: `${label} Per Level`,
        key: statKeys[i],
        value: config.dino_wild_stats[statKeys[i]],
      })),
    [config.dino_wild_stats]
  )

  return (
    <SettingsPanel title="WILD DINO STAT MULTIPLIERS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type="number"
          onChange={(v: number) => updateDinoWild(setting.key, v)}
          step={0.01}
          testId={`wild-dino-${setting.key}`}
        />
      ))}
    </SettingsPanel>
  )
}
