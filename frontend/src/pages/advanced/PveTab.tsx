import React from 'react'
import SettingRow from '../../components/SettingRow'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface PveTabProps {
  config: ServerConfig
}

export default function PveTab({ config }: PveTabProps) {
  const updatePve = useConfigUpdate('pve')

  const col1 = [
    {
      label: 'Allow Cave Building',
      value: config.pve.allow_cave_building,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('allow_cave_building', v),
    },
    {
      label: 'Force Allow Cave Flyers',
      value: config.pve.force_allow_cave_flyers,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('force_allow_cave_flyers', v),
    },
    {
      label: 'Allow Flyer Carry',
      value: config.pve.allow_flyer_carry,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('allow_flyer_carry', v),
    },
    {
      label: 'Disable Structure Decay',
      value: config.pve.disable_structure_decay_pve,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('disable_structure_decay_pve', v),
    },
    {
      label: 'Structure Decay Period',
      value: config.pve.structure_decay_period_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updatePve('structure_decay_period_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Disable Dino Decay',
      value: config.pve.disable_dino_decay_pve,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('disable_dino_decay_pve', v),
    },
    {
      label: 'Dino Decay Period',
      value: config.pve.dino_decay_period_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updatePve('dino_decay_period_multiplier', v),
      step: 0.1,
    },
  ]

  const col2 = [
    {
      label: 'Prevent Tribe Alliances',
      value: config.pve.prevent_tribe_alliances,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('prevent_tribe_alliances', v),
    },
    {
      label: 'PVE Allow Tribe War',
      value: config.pve.pve_allow_tribe_war,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('pve_allow_tribe_war', v),
    },
    {
      label: 'PVE Tribe War Cancel',
      value: config.pve.pve_allow_tribe_war_cancel,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('pve_allow_tribe_war_cancel', v),
    },
    {
      label: 'Extra Structure Prevention Volumes',
      value: config.pve.extra_structure_prevention_volumes,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('extra_structure_prevention_volumes', v),
    },
    {
      label: 'Prevent Diseases',
      value: config.pve.prevent_diseases,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('prevent_diseases', v),
    },
    {
      label: 'Non-Permanent Diseases',
      value: config.pve.non_permanent_diseases,
      type: 'boolean' as const,
      onChange: (v: boolean) => updatePve('non_permanent_diseases', v),
    },
  ]

  return (
    <div className="px-8 py-6">
      <div className="grid grid-cols-2 gap-4">
        {[col1, col2].map((col, ci) => (
          <div key={ci} className="ark-panel rounded-lg overflow-hidden">
            <div className="ark-scroll overflow-y-auto max-h-[calc(100vh-280px)]">
              {col.map((s, i) => (
                <SettingRow
                  key={i}
                  label={s.label}
                  value={s.value}
                  type={s.type}
                  onChange={s.onChange as any}
                  step={(s as any).step}
                  testId={`pve-${s.label.toLowerCase().replace(/ /g, '-')}`}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
