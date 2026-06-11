import type { ServerConfig } from '../types'

interface GeneralSettingsProps {
  config: ServerConfig
  onConfigChange: (config: ServerConfig) => void
}

export default function GeneralSettings({
  config,
  onConfigChange,
}: GeneralSettingsProps) {
  const handleChange = (field: string, value: any) => {
    onConfigChange({
      ...config,
      identification: {
        ...config.identification,
        [field]: value,
      },
    })
  }

  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-bold text-ark-cyan">Server Identification</h3>

      <div className="grid grid-cols-2 gap-6">
        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Session Name
          </label>
          <input
            type="text"
            value={config.identification.session_name}
            onChange={(e) => handleChange('session_name', e.target.value)}
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Server Password
          </label>
          <input
            type="password"
            value={config.identification.server_password}
            onChange={(e) => handleChange('server_password', e.target.value)}
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Admin Password
          </label>
          <input
            type="password"
            value={config.identification.admin_password}
            onChange={(e) => handleChange('admin_password', e.target.value)}
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Message of the Day
          </label>
          <input
            type="text"
            value={config.identification.server_message_of_the_day}
            onChange={(e) => handleChange('server_message_of_the_day', e.target.value)}
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          />
        </div>
      </div>

      <h3 className="text-2xl font-bold text-ark-cyan mt-8">Network Settings</h3>

      <div className="grid grid-cols-2 gap-6">
        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Game Port
          </label>
          <input
            type="number"
            value={config.network.port}
            onChange={(e) =>
              onConfigChange({
                ...config,
                network: {
                  ...config.network,
                  port: parseInt(e.target.value),
                },
              })
            }
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Query Port
          </label>
          <input
            type="number"
            value={config.network.query_port}
            onChange={(e) =>
              onConfigChange({
                ...config,
                network: {
                  ...config.network,
                  query_port: parseInt(e.target.value),
                },
              })
            }
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            RCON Port
          </label>
          <input
            type="number"
            value={config.network.rcon_port}
            onChange={(e) =>
              onConfigChange({
                ...config,
                network: {
                  ...config.network,
                  rcon_port: parseInt(e.target.value),
                },
              })
            }
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Server Platform
          </label>
          <select
            value={config.network.server_platform}
            onChange={(e) =>
              onConfigChange({
                ...config,
                network: {
                  ...config.network,
                  server_platform: e.target.value,
                },
              })
            }
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
          >
            <option>ALL</option>
            <option>WIN</option>
            <option>LINUX</option>
          </select>
        </div>
      </div>
    </div>
  )
}
