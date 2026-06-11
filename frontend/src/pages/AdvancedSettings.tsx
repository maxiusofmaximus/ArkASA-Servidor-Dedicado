import type { ServerConfig } from '../types'

interface Props {
  config: ServerConfig
  onConfigChange: (config: ServerConfig) => void
}

export default function AdvancedSettings({ config, onConfigChange }: Props) {
  const handleAdvancedChange = (field: string, value: any) => {
    onConfigChange({
      ...config,
      advanced: {
        ...config.advanced,
        [field]: value,
      },
    })
  }

  const handleGameplayChange = (field: string, value: any) => {
    onConfigChange({
      ...config,
      gameplay: {
        ...config.gameplay,
        [field]: value,
      },
    })
  }

  return (
    <div className="space-y-8">
      <div>
        <h3 className="text-2xl font-bold text-ark-cyan mb-6">Advanced Game Options</h3>
        <div className="space-y-4">
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.allow_unlimited_respecs}
              onChange={(e) => handleAdvancedChange('allow_unlimited_respecs', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Allow Unlimited Respecs</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.allow_flyer_carry}
              onChange={(e) => handleAdvancedChange('allow_flyer_carry', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Allow Flyer Carry</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.allow_cryo_sick_pve}
              onChange={(e) => handleAdvancedChange('allow_cryo_sick_pve', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Allow Cryo Sickness in PvE</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.disable_structure_decay}
              onChange={(e) => handleAdvancedChange('disable_structure_decay', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Disable Structure Decay</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.enable_cave_flyers}
              onChange={(e) => handleAdvancedChange('enable_cave_flyers', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Enable Cave Flyers</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.no_survivor_downloads}
              onChange={(e) => handleAdvancedChange('no_survivor_downloads', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Disable Survivor Downloads</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.no_dino_downloads}
              onChange={(e) => handleAdvancedChange('no_dino_downloads', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Disable Dino Downloads</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.advanced.no_item_downloads}
              onChange={(e) => handleAdvancedChange('no_item_downloads', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Disable Item Downloads</span>
          </label>
        </div>
      </div>

      <div>
        <h3 className="text-2xl font-bold text-ark-cyan mb-6">Gameplay Features</h3>
        <div className="space-y-4">
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.gameplay.allow_third_person_player}
              onChange={(e) => handleGameplayChange('allow_third_person_player', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Allow Third Person Camera</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.gameplay.enable_pvp_gamma_bypass}
              onChange={(e) => handleGameplayChange('enable_pvp_gamma_bypass', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Enable PvP Gamma Bypass</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={config.gameplay.allow_cryopod_nerf_removal}
              onChange={(e) => handleGameplayChange('allow_cryopod_nerf_removal', e.target.checked)}
              className="w-5 h-5"
            />
            <span className="text-ark-cyan font-semibold">Allow Cryopod Nerf Removal</span>
          </label>
        </div>
      </div>

      <div className="p-4 bg-ark-dark border border-ark-purple/30 rounded">
        <p className="text-xs text-gray-400">
          <strong>Note:</strong> Some settings require a server restart to take effect. The UI will
          show which settings need a restart when they are modified.
        </p>
      </div>
    </div>
  )
}
