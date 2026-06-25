import type { ServerConfig } from '../../types'
import RawConfigViewer from '../RawConfigViewer'

interface ConfigTabProps {
  config: ServerConfig | null
  onConfigSaved: (updated: ServerConfig) => void
}

export default function ConfigTab({ config, onConfigSaved }: ConfigTabProps) {
  if (!config) {
    return (
      <p className="text-ark-cyan/40 text-sm text-center py-8">
        Load the server configuration first.
      </p>
    )
  }

  return (
    <RawConfigViewer config={config} onConfigSaved={onConfigSaved} />
  )
}
