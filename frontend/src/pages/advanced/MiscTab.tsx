import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface MiscTabProps {
  config: ServerConfig
}

export default function MiscTab({ config }: MiscTabProps) {
  const updateAdvanced = useConfigUpdate('advanced')

  const settings = useMemo(
    () => [
      {
        label: 'Allow Custom Recipes',
        value: config.advanced.allow_custom_recipes,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateAdvanced('allow_custom_recipes', v),
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
        step: 100,
      },
      {
        label: 'Disable Friendly Fire',
        value: config.advanced.disable_friendly_fire,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateAdvanced('disable_friendly_fire', v),
      },
    ],
    [config.advanced, updateAdvanced]
  )

  return (
    <SettingsPanel title="MISCELLANEOUS SETTINGS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          step={setting.step}
          testId={`misc-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
