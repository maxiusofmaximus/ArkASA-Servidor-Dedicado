import { useState } from 'react'
import type { ServerConfig } from '../types'

interface ConfigPreviewProps {
  config: ServerConfig
}

export default function ConfigPreview({ config }: ConfigPreviewProps) {
  const [activeTab, setActiveTab] = useState<'toml' | 'ini' | 'game'>('toml')

  const generateTOML = () => {
    const sections = [
      '[identification]',
      `session_name = "${config.identification.session_name}"`,
      `admin_password = "${config.identification.admin_password}"`,
      `server_password = "${config.identification.server_password}"`,
      '',
      '[network]',
      `port = ${config.network.port}`,
      `query_port = ${config.network.query_port}`,
      `rcon_port = ${config.network.rcon_port}`,
      `server_platform = "${config.network.server_platform}"`,
      '',
      '[gameplay]',
      `server_pve = ${config.gameplay.server_pve}`,
      `max_players = ${config.gameplay.max_players}`,
      `dino_count_multiplier = ${config.gameplay.dino_count_multiplier}`,
      '',
      '[multipliers]',
      `xp_multiplier = ${config.multipliers.xp_multiplier}`,
      `taming_speed_multiplier = ${config.multipliers.taming_speed_multiplier}`,
      `harvest_amount_multiplier = ${config.multipliers.harvest_amount_multiplier}`,
      '',
      '[mods]',
      `active_mods = [${config.mods.active_mods.map((m) => `"${m}"`).join(', ')}]`,
    ]
    return sections.join('\n')
  }

  const generateGameINI = () => {
    const sections = [
      '[/script/shootergame.shootergamemode]',
      `SessionName=${config.identification.session_name}`,
      `AdminPassword=${config.identification.admin_password}`,
      `ServerPassword=${config.identification.server_password}`,
      '',
      '[/script/shootergame.shootergamestate]',
      `MaxPlayers=${config.gameplay.max_players}`,
      `DinoCountMultiplier=${config.gameplay.dino_count_multiplier}`,
      `bServerPVE=${config.gameplay.server_pve ? 'true' : 'false'}`,
    ]
    return sections.join('\n')
  }

  const generateGameUserSettingsINI = () => {
    const sections = [
      '[ServerSettings]',
      `Port=${config.network.port}`,
      `QueryPort=${config.network.query_port}`,
      `RCONPort=${config.network.rcon_port}`,
      '',
      '[Multipliers]',
      `XPMultiplier=${config.multipliers.xp_multiplier}`,
      `TamingSpeedMultiplier=${config.multipliers.taming_speed_multiplier}`,
      `HarvestAmountMultiplier=${config.multipliers.harvest_amount_multiplier}`,
      `BabyMatureSpeedMultiplier=${config.multipliers.baby_mature_speed_multiplier}`,
      `EggHatchSpeedMultiplier=${config.multipliers.egg_hatch_speed_multiplier}`,
      `CraftingSpeedMultiplier=${config.multipliers.crafting_speed_multiplier}`,
      '',
      '[Mods]',
      `ActiveMods=${config.mods.active_mods.join(',')}`,
    ]
    return sections.join('\n')
  }

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  const content =
    activeTab === 'toml'
      ? generateTOML()
      : activeTab === 'game'
        ? generateGameINI()
        : generateGameUserSettingsINI()

  return (
    <div className="bg-ark-secondary border border-ark-cyan/30 rounded-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-bold text-ark-cyan">Configuration Preview</h3>
        <button
          onClick={() => handleCopy(content)}
          className="px-4 py-2 bg-ark-cyan/20 text-ark-cyan rounded hover:bg-ark-cyan/30 transition text-sm"
        >
          Copy to Clipboard
        </button>
      </div>

      <div className="flex gap-2 mb-4">
        {['toml', 'game', 'ini'].map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab as any)}
            className={`px-4 py-2 rounded transition ${
              activeTab === tab
                ? 'bg-ark-cyan text-ark-dark font-semibold'
                : 'bg-ark-dark border border-ark-cyan/30 text-ark-cyan hover:bg-ark-cyan/10'
            }`}
          >
            {tab === 'toml' ? 'config.toml' : tab === 'game' ? 'Game.ini' : 'GameUserSettings.ini'}
          </button>
        ))}
      </div>

      <div className="bg-ark-dark rounded p-4 font-mono text-sm overflow-x-auto max-h-96 overflow-y-auto">
        <pre className="text-gray-300 whitespace-pre-wrap break-words">{content}</pre>
      </div>

      <div className="mt-4 text-xs text-gray-400 space-y-1">
        <p>📋 Generated files:</p>
        <p>• config.toml (primary configuration)</p>
        <p>• Game.ini (game rules)</p>
        <p>• GameUserSettings.ini (server settings)</p>
      </div>
    </div>
  )
}
