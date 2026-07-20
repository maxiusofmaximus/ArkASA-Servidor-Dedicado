import type { ServerConfig } from '../types'
import VersionBadge from '../components/VersionBadge'
import { useServerVersion } from '../hooks/useServerVersion'

interface GeneralSettingsProps {
  config: ServerConfig
  onConfigChange: (config: ServerConfig) => void
}

// ARK password rules:
// • Max 27 characters (engine hard limit)
// • No spaces, quotes, backslash, forward-slash, backtick, or tilde
const ARK_PWD_REGEX = /^[A-Za-z0-9!@#$%^&*()\-_+=[\]{}|;:,.?]*$/
const ARK_PWD_MAX = 27

function validatePassword(value: string): string | null {
  if (value.length > ARK_PWD_MAX) return `Máximo ${ARK_PWD_MAX} caracteres (ARK engine limit)`
  if (value && !ARK_PWD_REGEX.test(value))
    return 'Caracter no permitido — evita espacios, comillas, \\ / ` ~'
  return null
}

function PasswordField({
  label,
  value,
  onChange,
}: {
  label: string
  value: string
  onChange: (v: string) => void
}) {
  const error = validatePassword(value)
  const tooLong = value.length > ARK_PWD_MAX

  return (
    <div>
      <label className="block text-sm font-semibold text-ark-cyan mb-2">{label}</label>
      <div className="relative">
        <input
          type="password"
          value={value}
          maxLength={ARK_PWD_MAX}
          onChange={(e) => onChange(e.target.value)}
          className={`w-full bg-ark-dark border rounded px-4 py-2 text-white focus:outline-none transition ${
            error
              ? 'border-red-500/70 focus:border-red-400'
              : 'border-ark-cyan/30 focus:border-ark-cyan'
          }`}
        />
        {/* Character counter */}
        <span
          className={`absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-mono pointer-events-none ${
            tooLong ? 'text-red-400' : value.length >= 20 ? 'text-yellow-400/70' : 'text-ark-cyan/30'
          }`}
        >
          {value.length}/{ARK_PWD_MAX}
        </span>
      </div>
      {error && (
        <p className="text-red-400/80 text-[11px] mt-1 flex items-center gap-1">
          <span>⚠</span> {error}
        </p>
      )}
    </div>
  )
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

  const version = useServerVersion(config)

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <h3 className="text-2xl font-bold text-ark-cyan">Server Identification</h3>
        <VersionBadge
          info={version.info}
          loading={version.loading}
          updating={version.updating}
          onRefresh={version.refresh}
          onUpdate={version.runUpdate}
        />
      </div>

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

        <PasswordField
          label="Server Password"
          value={config.identification.server_password}
          onChange={(v) => handleChange('server_password', v)}
        />

        <PasswordField
          label="Admin Password"
          value={config.identification.admin_password}
          onChange={(v) => handleChange('admin_password', v)}
        />

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
                network: { ...config.network, port: parseInt(e.target.value) },
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
                network: { ...config.network, query_port: parseInt(e.target.value) },
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
                network: { ...config.network, rcon_port: parseInt(e.target.value) },
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
                network: { ...config.network, server_platform: e.target.value },
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
