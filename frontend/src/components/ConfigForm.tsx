import { invoke } from '@tauri-apps/api/tauri'
import { useState } from 'react'
import { useConfigStore } from '../stores/configStore'
import type { ServerConfig, Tab } from '../types'
import GeneralSettings from '../pages/GeneralSettings'
import GameplaySettings from '../pages/GameplaySettings'
import ServerSettings from '../pages/ServerSettings'
import AdvancedSettings from '../pages/AdvancedSettings'

interface ConfigFormProps {
  config: ServerConfig
  activeTab: Tab
  onConfigChange: (config: ServerConfig) => void
}

export default function ConfigForm({
  config,
  activeTab,
  onConfigChange,
}: ConfigFormProps) {
  const { setSaving } = useConfigStore()
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const handleSave = async () => {
    try {
      setSaving(true)
      setError(null)

      // Validate
      const validation = await invoke('validate_config', { config })
      console.log('Validation result:', validation)

      // Save
      await invoke('save_config', {
        config,
        configPath: 'config.toml',
      })

      setSuccess(true)
      setTimeout(() => setSuccess(false), 3000)
    } catch (err) {
      setError(`Failed to save config: ${err}`)
    } finally {
      setSaving(false)
    }
  }

  const handleReset = () => {
    if (confirm('Reset to defaults?')) {
      const defaults: ServerConfig = {
        identification: {
          session_name: 'ARK Server',
          server_password: 'changeme',
          admin_password: 'changeme',
          server_message_of_the_day: 'Welcome to ARK!',
        },
        network: {
          port: 7777,
          query_port: 27015,
          rcon_port: 27020,
          server_platform: 'ALL',
        },
        gameplay: {
          server_pve: true,
          max_players: 70,
          difficulty_offset: 2.0,
          dino_count_multiplier: 2.0,
          enable_pvp_gamma_bypass: false,
          allow_third_person_player: true,
          allow_cryopod_nerf_removal: false,
        },
        multipliers: {
          xp_multiplier: 3.0,
          taming_speed_multiplier: 15.0,
          harvest_amount_multiplier: 8.0,
          harvest_health_multiplier: 3.0,
          baby_mature_speed_multiplier: 40.0,
          baby_food_consumption_multiplier: 4.0,
          baby_cuddle_loss_multiplier: 0.07,
          egg_hatch_speed_multiplier: 20.0,
          poops_interval_multiplier: 1.0,
          lay_egg_interval_multiplier: 5.04,
          mating_interval_multiplier: 2.98,
          crafting_skill_bonus_multiplier: 3.0,
          crafting_speed_multiplier: 3.0,
        },
        mods: {
          active_mods: [],
          mod_config: {},
        },
        paths: {
          steam_cmd_dir: 'C:\\ASA\\steamcmd',
          server_dir: 'C:\\ASA\\server',
          backup_dir: 'C:\\ASA\\backups',
          game_ini_path: 'C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\Game.ini',
          gamesettings_ini_path:
            'C:\\ASA\\server\\ShooterGame\\Saved\\Config\\WindowsServer\\GameUserSettings.ini',
        },
        performance: {
          max_structure_in_range: 10500,
          structure_prevention_radius: 1000.0,
          use_optimization: true,
          enable_debug_logging: false,
        },
        world: {
          day_cycle_speed_scale: 0.5,
          night_time_speed_scale: 1.46,
          day_time_speed_scale: 0.5,
          overall_damage_multiplier: 1.0,
          player_character_health_multiplier: 3.0,
          dino_character_health_multiplier: 1.0,
        },
        advanced: {
          allow_unlimited_respecs: true,
          allow_flyer_carry: true,
          allow_cryo_sick_pve: false,
          disable_structure_decay: false,
          enable_cave_flyers: true,
          no_survivor_downloads: false,
          no_dino_downloads: false,
          no_item_downloads: false,
          custom_config: {},
        },
      }
      onConfigChange(defaults)
    }
  }

  return (
    <div className="space-y-6">
      {error && (
        <div className="bg-ark-accent/20 border-l-4 border-ark-accent p-4 text-ark-accent rounded">
          {error}
        </div>
      )}

      {success && (
        <div className="bg-ark-cyan/20 border-l-4 border-ark-cyan p-4 text-ark-cyan rounded">
          Configuration saved successfully!
        </div>
      )}

      <div className="bg-ark-secondary p-6 rounded-lg border border-ark-cyan/30">
        {activeTab === 'general' && (
          <GeneralSettings config={config} onConfigChange={onConfigChange} />
        )}
        {activeTab === 'gameplay' && (
          <GameplaySettings config={config} onConfigChange={onConfigChange} />
        )}
        {activeTab === 'server' && (
          <ServerSettings config={config} onConfigChange={onConfigChange} />
        )}
        {activeTab === 'advanced' && (
          <AdvancedSettings config={config} onConfigChange={onConfigChange} />
        )}
      </div>

      <div className="flex gap-4 justify-end">
        <button
          onClick={handleReset}
          className="px-6 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 transition"
        >
          Reset to Defaults
        </button>
        <button
          onClick={handleSave}
          className="px-6 py-2 bg-ark-cyan text-ark-dark rounded font-semibold hover:bg-ark-cyan/80 transition shadow-ark"
        >
          Save Configuration
        </button>
      </div>
    </div>
  )
}
