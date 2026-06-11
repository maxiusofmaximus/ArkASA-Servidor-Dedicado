import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface RulesTabProps {
  config: ServerConfig
}

export default function RulesTab({ config }: RulesTabProps) {
  const updateGameplay = useConfigUpdate('gameplay')

  const settings = useMemo(
    () => [
      {
        label: 'Allow Third Person Camera',
        value: config.gameplay.allow_third_person_player,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('allow_third_person_player', v),
      },
      {
        label: 'Global Voice Chat',
        value: config.gameplay.global_voice_chat,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('global_voice_chat', v),
      },
      {
        label: 'Proximity Chat',
        value: config.gameplay.proximity_chat,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('proximity_chat', v),
      },
      {
        label: 'Admin Logging',
        value: config.gameplay.admin_logging,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('admin_logging', v),
      },
      {
        label: 'Notify Player Joined',
        value: !config.gameplay.dont_always_notify_player_joined,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('dont_always_notify_player_joined', !v),
      },
      {
        label: 'Notify Player Left',
        value: config.gameplay.always_notify_player_left,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('always_notify_player_left', v),
      },
      {
        label: 'Server Crosshair',
        value: config.gameplay.server_crosshair,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('server_crosshair', v),
      },
      {
        label: 'Show Floating Damage',
        value: config.gameplay.show_floating_damage_text,
        type: 'boolean' as const,
        onChange: (v: boolean) => updateGameplay('show_floating_damage_text', v),
      },
    ],
    [config.gameplay, updateGameplay]
  )

  return (
    <SettingsPanel title="SERVER RULES">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          testId={`rules-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
