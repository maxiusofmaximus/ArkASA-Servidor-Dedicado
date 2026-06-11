import type { ServerConfig } from '../types'

interface Props {
  config: ServerConfig
  onConfigChange: (config: ServerConfig) => void
}

export default function AdvancedSettings({ config, onConfigChange }: Props) {
  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-bold text-ark-cyan">Advanced Settings</h3>
      <div className="text-gray-400">Advanced configuration panel - Coming soon</div>
    </div>
  )
}
