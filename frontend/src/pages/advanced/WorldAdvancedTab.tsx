import React from 'react'
import SettingRow from '../../components/SettingRow'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface WorldAdvancedTabProps {
  config: ServerConfig
}

export default function WorldAdvancedTab({ config }: WorldAdvancedTabProps) {
  const updateWorld = useConfigUpdate('world')
  const updateMult = useConfigUpdate('multipliers')

  const col1 = [
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
      label: 'No Resource Radius (Players)',
      value: config.world.resource_no_replenish_radius_players,
      type: 'number' as const,
      onChange: (v: number) => updateWorld('resource_no_replenish_radius_players', v),
      step: 0.1,
    },
    {
      label: 'No Resource Radius (Structures)',
      value: config.world.resource_no_replenish_radius_structures,
      type: 'number' as const,
      onChange: (v: number) => updateWorld('resource_no_replenish_radius_structures', v),
      step: 0.1,
    },
  ]

  const col2 = [
    {
      label: 'Harvest Health',
      value: config.multipliers.harvest_health_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('harvest_health_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Resource Respawn',
      value: config.world.resource_respawn_period_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateWorld('resource_respawn_period_multiplier', v),
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
    {
      label: 'Mating Interval',
      value: config.multipliers.mating_interval_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('mating_interval_multiplier', v),
      step: 0.01,
    },
    {
      label: 'Lay Egg Interval',
      value: config.multipliers.lay_egg_interval_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('lay_egg_interval_multiplier', v),
      step: 0.01,
    },
    {
      label: 'Egg Hatch Speed',
      value: config.multipliers.egg_hatch_speed_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('egg_hatch_speed_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Poop Interval',
      value: config.multipliers.poops_interval_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('poops_interval_multiplier', v),
      step: 0.1,
    },
  ]

  const col3 = [
    {
      label: 'Baby Mature Speed',
      value: config.multipliers.baby_mature_speed_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('baby_mature_speed_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Baby Food Consumption',
      value: config.multipliers.baby_food_consumption_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('baby_food_consumption_multiplier', v),
      step: 0.01,
    },
    {
      label: 'Baby Cuddle Interval',
      value: config.multipliers.baby_cuddle_interval_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('baby_cuddle_interval_multiplier', v),
      step: 0.01,
    },
    {
      label: 'Baby Cuddle Grace Period',
      value: config.multipliers.baby_cuddle_grace_period_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('baby_cuddle_grace_period_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Baby Cuddle Loss (Imprint)',
      value: config.multipliers.baby_cuddle_loss_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('baby_cuddle_loss_multiplier', v),
      step: 0.001,
    },
    {
      label: 'Baby Imprint Stat Scale',
      value: config.multipliers.baby_imprint_stat_scale_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('baby_imprint_stat_scale_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Force Reset Wild Dinos',
      value: config.world.force_reset_wild_dinos,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateWorld('force_reset_wild_dinos', v),
    },
  ]

  return (
    <div className="px-8 py-6">
      <div className="grid grid-cols-3 gap-4">
        {[col1, col2, col3].map((col, ci) => (
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
                  testId={`world-adv-${s.label.toLowerCase().replace(/ /g, '-')}`}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
