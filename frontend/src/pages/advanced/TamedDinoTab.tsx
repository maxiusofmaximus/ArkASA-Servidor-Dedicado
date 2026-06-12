import React, { useMemo } from 'react'
import TripleStatRow from '../../components/TripleStatRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface TamedDinoTabProps {
  config: ServerConfig
}

const statLabels = ['Health', 'Stamina', 'Oxygen', 'Food', 'Water', 'Weight', 'Melee Damage', 'Speed', 'Fortitude', 'Torpidity']
const statKeys = ['health', 'stamina', 'oxygen', 'food', 'water', 'weight', 'melee_damage', 'speed', 'fortitude', 'torpidity'] as const

export default function TamedDinoTab({ config }: TamedDinoTabProps) {
  const updateDinoStats = useConfigUpdate('dino_tamed_stats')
  const stats = config.dino_tamed_stats

  return (
    <SettingsPanel wide>
      <div className="px-4 py-3 border-b border-ark-cyan/20 bg-ark-secondary/10">
        <div className="flex justify-between">
          <span className="text-ark-cyan/60 text-xs font-semibold tracking-widest w-32">STAT</span>
          <div className="flex gap-8 flex-1 justify-end">
            <span className="text-ark-cyan/60 text-xs font-semibold tracking-widest" data-testid="col-per-level">PER LEVEL</span>
            <span className="text-ark-cyan/60 text-xs font-semibold tracking-widest" data-testid="col-add-per-level">ADD PER LEVEL</span>
            <span className="text-ark-cyan/60 text-xs font-semibold tracking-widest" data-testid="col-affinity">AFFINITY</span>
          </div>
        </div>
      </div>

      {statLabels.map((label, i) => {
        const key = statKeys[i]
        return (
          <TripleStatRow
            key={i}
            label={label}
            perLevel={stats.per_level[key]}
            addPerLevel={stats.add_per_level[key]}
            affinity={stats.affinity[key]}
            onPerLevelChange={(v) => updateDinoStats('per_level', { ...stats.per_level, [key]: v })}
            onAddPerLevelChange={(v) => updateDinoStats('add_per_level', { ...stats.add_per_level, [key]: v })}
            onAffinityChange={(v) => updateDinoStats('affinity', { ...stats.affinity, [key]: v })}
          />
        )
      })}
    </SettingsPanel>
  )
}
