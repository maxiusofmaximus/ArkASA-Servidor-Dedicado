import React from 'react'
import SettingRow from '../../components/SettingRow'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface MiscTabProps {
  config: ServerConfig
}

export default function MiscTab({ config }: MiscTabProps) {
  const updateAdvanced = useConfigUpdate('advanced')
  const updateMult = useConfigUpdate('multipliers')

  const col1 = [
    {
      label: 'Allow Custom Recipes',
      value: config.advanced.allow_custom_recipes,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('allow_custom_recipes', v),
    },
    {
      label: 'Custom Recipe Effectiveness',
      value: config.advanced.custom_recipe_effectiveness_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateAdvanced('custom_recipe_effectiveness_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Custom Recipe Skill',
      value: config.advanced.custom_recipe_skill_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateAdvanced('custom_recipe_skill_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Supply Crate Loot Quality',
      value: config.advanced.supply_crate_loot_quality_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateAdvanced('supply_crate_loot_quality_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Fishing Loot Quality',
      value: config.advanced.fishing_loot_quality_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateAdvanced('fishing_loot_quality_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Crafting Speed',
      value: config.multipliers.crafting_speed_multiplier,
      type: 'number' as const,
      onChange: (v: number) => updateMult('crafting_speed_multiplier', v),
      step: 0.1,
    },
    {
      label: 'Max Tamed Dinos',
      value: config.advanced.max_tamed_dinos,
      type: 'number' as const,
      onChange: (v: number) => updateAdvanced('max_tamed_dinos', v),
      step: 100,
    },
    {
      label: 'Platform Structure Limit',
      value: config.advanced.platform_structure_limit,
      type: 'number' as const,
      onChange: (v: number) => updateAdvanced('platform_structure_limit', v),
      step: 1,
    },
  ]

  const col2 = [
    {
      label: 'Disable Friendly Fire',
      value: config.advanced.disable_friendly_fire,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('disable_friendly_fire', v),
    },
    {
      label: 'No Survivor Downloads',
      value: config.advanced.no_survivor_downloads,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('no_survivor_downloads', v),
    },
    {
      label: 'No Dino Downloads',
      value: config.advanced.no_dino_downloads,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('no_dino_downloads', v),
    },
    {
      label: 'No Item Downloads',
      value: config.advanced.no_item_downloads,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('no_item_downloads', v),
    },
    {
      label: 'Disable Dino Riding',
      value: config.advanced.disable_dino_riding,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('disable_dino_riding', v),
    },
    {
      label: 'Disable Dino Taming',
      value: config.advanced.disable_dino_taming,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('disable_dino_taming', v),
    },
    {
      label: 'Allow Raid Dino Feeding',
      value: config.advanced.allow_raid_dino_feeding,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('allow_raid_dino_feeding', v),
    },
    {
      label: 'Disable Photo Mode',
      value: config.advanced.disable_photo_mode,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('disable_photo_mode', v),
    },
    {
      label: 'Photo Mode Range Limit',
      value: config.advanced.photo_mode_range_limit,
      type: 'number' as const,
      onChange: (v: number) => updateAdvanced('photo_mode_range_limit', v),
      step: 1000,
    },
    {
      label: 'Disable Structure Collision',
      value: config.advanced.disable_structure_placement_collision,
      type: 'boolean' as const,
      onChange: (v: boolean) => updateAdvanced('disable_structure_placement_collision', v),
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
                  testId={`misc-${s.label.toLowerCase().replace(/ /g, '-')}`}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
