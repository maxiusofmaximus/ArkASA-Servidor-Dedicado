import type { ServerConfig } from '../types'

interface Props {
  config: ServerConfig
  onConfigChange: (config: ServerConfig) => void
}

export default function GameplaySettings({ config, onConfigChange }: Props) {
  const handleGameplayChange = (field: string, value: any) => {
    onConfigChange({
      ...config,
      gameplay: {
        ...config.gameplay,
        [field]: value,
      },
    })
  }

  const handleMultiplierChange = (field: string, value: any) => {
    onConfigChange({
      ...config,
      multipliers: {
        ...config.multipliers,
        [field]: parseFloat(value),
      },
    })
  }

  return (
    <div className="space-y-8">
      <div>
        <h3 className="text-2xl font-bold text-ark-cyan mb-6">Gameplay Settings</h3>
        <div className="grid grid-cols-2 gap-6">
          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Server Type
            </label>
            <select
              value={config.gameplay.server_pve ? 'pve' : 'pvp'}
              onChange={(e) => handleGameplayChange('server_pve', e.target.value === 'pve')}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            >
              <option value="pve">PvE (Player vs Environment)</option>
              <option value="pvp">PvP (Player vs Player)</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Max Players
            </label>
            <input
              type="number"
              value={config.gameplay.max_players}
              onChange={(e) => handleGameplayChange('max_players', parseInt(e.target.value))}
              min="1"
              max="1000"
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Difficulty Offset
            </label>
            <input
              type="number"
              step="0.1"
              value={config.gameplay.difficulty_offset}
              onChange={(e) => handleGameplayChange('difficulty_offset', parseFloat(e.target.value))}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Dino Count Multiplier
            </label>
            <input
              type="number"
              step="0.1"
              value={config.gameplay.dino_count_multiplier}
              onChange={(e) => handleGameplayChange('dino_count_multiplier', parseFloat(e.target.value))}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-2xl font-bold text-ark-cyan mb-6">Multipliers</h3>
        <div className="grid grid-cols-3 gap-6">
          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">XP Multiplier</label>
            <input
              type="number"
              step="0.1"
              value={config.multipliers.xp_multiplier}
              onChange={(e) => handleMultiplierChange('xp_multiplier', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Taming Speed
            </label>
            <input
              type="number"
              step="0.1"
              value={config.multipliers.taming_speed_multiplier}
              onChange={(e) => handleMultiplierChange('taming_speed_multiplier', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Harvest Amount
            </label>
            <input
              type="number"
              step="0.1"
              value={config.multipliers.harvest_amount_multiplier}
              onChange={(e) => handleMultiplierChange('harvest_amount_multiplier', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Baby Mature Speed
            </label>
            <input
              type="number"
              step="0.1"
              value={config.multipliers.baby_mature_speed_multiplier}
              onChange={(e) => handleMultiplierChange('baby_mature_speed_multiplier', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Egg Hatch Speed
            </label>
            <input
              type="number"
              step="0.1"
              value={config.multipliers.egg_hatch_speed_multiplier}
              onChange={(e) => handleMultiplierChange('egg_hatch_speed_multiplier', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Crafting Speed
            </label>
            <input
              type="number"
              step="0.1"
              value={config.multipliers.crafting_speed_multiplier}
              onChange={(e) => handleMultiplierChange('crafting_speed_multiplier', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>
        </div>
      </div>
    </div>
  )
}
