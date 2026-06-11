import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface PlayerStatsTabProps {
  config: ServerConfig
}

const statLabels = ['Health', 'Stamina', 'Oxygen', 'Food', 'Water', 'Weight', 'Melee Damage', 'Speed', 'Fortitude', 'Crafting Speed', 'Torpidity']
const statKeys = ['health', 'stamina', 'oxygen', 'food', 'water', 'weight', 'melee_damage', 'speed', 'fortitude', 'crafting_speed', 'torpidity'] as const

export default function PlayerStatsTab({ config }: PlayerStatsTabProps) {
  const updatePlayerStats = useConfigUpdate('player_stats')

  const settings = useMemo(
    () =>
      statLabels.map((label, i) => ({
        label: `${label} Per Level`,
        key: statKeys[i],
        value: config.player_stats[statKeys[i]],
      })),
    [config.player_stats]
  )

  return (
    <SettingsPanel title="PLAYER STAT MULTIPLIERS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type="number"
          onChange={(v: number) => updatePlayerStats(setting.key, v)}
          step={0.01}
          testId={`player-stat-${setting.key}`}
        />
      ))}
    </SettingsPanel>
  )
}
