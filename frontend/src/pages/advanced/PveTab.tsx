import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface PveTabProps {
  config: ServerConfig
}

export default function PveTab({ config }: PveTabProps) {
  const updatePve = useConfigUpdate('pve')

  const settings = useMemo(
    () => [
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
        label: 'Disable Structure Decay',
        value: config.pve.disable_structure_decay_pve,
        type: 'boolean' as const,
        onChange: (v: boolean) => updatePve('disable_structure_decay_pve', v),
      },
      {
        label: 'Disable Dino Decay',
        value: config.pve.disable_dino_decay_pve,
        type: 'boolean' as const,
        onChange: (v: boolean) => updatePve('disable_dino_decay_pve', v),
      },
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
    ],
    [config.pve, updatePve]
  )

  return (
    <SettingsPanel title="PVE SETTINGS">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          testId={`pve-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
