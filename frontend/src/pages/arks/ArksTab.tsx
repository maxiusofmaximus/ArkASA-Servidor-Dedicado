import React, { useMemo } from 'react'
import SettingRow from '../../components/SettingRow'
import SettingsPanel from '../../components/SettingsPanel'
import { useConfigUpdate } from '../../hooks/useConfigUpdate'
import type { ServerConfig } from '../../types'

interface ArksTabProps {
  config: ServerConfig
}

export default function ArksTab({ config }: ArksTabProps) {
  const updateId = useConfigUpdate('identification')
  const updateNetwork = useConfigUpdate('network')

  const settings = useMemo(
    () => [
      // Identification
      {
        label: 'Server Name',
        value: config.identification.session_name,
        type: 'text' as const,
        onChange: (v: string) => updateId('session_name', v),
      },
      {
        label: 'Server Password',
        value: config.identification.server_password,
        type: 'text' as const,
        onChange: (v: string) => updateId('server_password', v),
      },
      {
        label: 'Admin Password',
        value: config.identification.admin_password,
        type: 'text' as const,
        onChange: (v: string) => updateId('admin_password', v),
      },
      {
        label: 'MOTD',
        value: config.identification.server_message_of_the_day,
        type: 'text' as const,
        onChange: (v: string) => updateId('server_message_of_the_day', v),
      },
      // Network
      {
        label: 'Game Port',
        value: config.network.port,
        type: 'number' as const,
        onChange: (v: number) => updateNetwork('port', v as any),
        min: 1024,
        max: 65535,
      },
      {
        label: 'Query Port',
        value: config.network.query_port,
        type: 'number' as const,
        onChange: (v: number) => updateNetwork('query_port', v as any),
        min: 1024,
        max: 65535,
      },
      {
        label: 'RCON Port',
        value: config.network.rcon_port,
        type: 'number' as const,
        onChange: (v: number) => updateNetwork('rcon_port', v as any),
        min: 1024,
        max: 65535,
      },
    ],
    [config.identification, config.network, updateId, updateNetwork]
  )

  return (
    <SettingsPanel title="ARKS IDENTIFICATION & NETWORK">
      {settings.map((setting, i) => (
        <SettingRow
          key={i}
          label={setting.label}
          value={setting.value}
          type={setting.type}
          onChange={setting.onChange}
          min={setting.min}
          max={setting.max}
          testId={`arks-${setting.label.toLowerCase().replace(/ /g, '-')}`}
        />
      ))}
    </SettingsPanel>
  )
}
