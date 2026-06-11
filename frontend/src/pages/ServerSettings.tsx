import type { ServerConfig } from '../types'

interface Props {
  config: ServerConfig
  onConfigChange: (config: ServerConfig) => void
}

export default function ServerSettings({ config, onConfigChange }: Props) {
  const handleWorldChange = (field: string, value: any) => {
    onConfigChange({
      ...config,
      world: {
        ...config.world,
        [field]: parseFloat(value),
      },
    })
  }

  const handlePerformanceChange = (field: string, value: any) => {
    onConfigChange({
      ...config,
      performance: {
        ...config.performance,
        [field]: typeof value === 'string' ? parseFloat(value) : value,
      },
    })
  }

  const handleModsChange = (modsStr: string) => {
    const mods = modsStr
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s && s !== '0')
    onConfigChange({
      ...config,
      mods: {
        ...config.mods,
        active_mods: mods,
      },
    })
  }

  return (
    <div className="space-y-8">
      <div>
        <h3 className="text-2xl font-bold text-ark-cyan mb-6">World Settings</h3>
        <div className="grid grid-cols-2 gap-6">
          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Day Cycle Speed
            </label>
            <input
              type="number"
              step="0.1"
              value={config.world.day_cycle_speed_scale}
              onChange={(e) => handleWorldChange('day_cycle_speed_scale', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Night Time Speed
            </label>
            <input
              type="number"
              step="0.1"
              value={config.world.night_time_speed_scale}
              onChange={(e) => handleWorldChange('night_time_speed_scale', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Day Time Speed
            </label>
            <input
              type="number"
              step="0.1"
              value={config.world.day_time_speed_scale}
              onChange={(e) => handleWorldChange('day_time_speed_scale', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Overall Damage Multiplier
            </label>
            <input
              type="number"
              step="0.1"
              value={config.world.overall_damage_multiplier}
              onChange={(e) => handleWorldChange('overall_damage_multiplier', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-2xl font-bold text-ark-cyan mb-6">Performance Settings</h3>
        <div className="grid grid-cols-2 gap-6">
          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Max Structures in Range
            </label>
            <input
              type="number"
              value={config.performance.max_structure_in_range}
              onChange={(e) => handlePerformanceChange('max_structure_in_range', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-ark-cyan mb-2">
              Structure Prevention Radius
            </label>
            <input
              type="number"
              step="10"
              value={config.performance.structure_prevention_radius}
              onChange={(e) => handlePerformanceChange('structure_prevention_radius', e.target.value)}
              className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan"
            />
          </div>

          <div className="col-span-2">
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={config.performance.use_optimization}
                onChange={(e) => handlePerformanceChange('use_optimization', e.target.checked)}
                className="w-5 h-5"
              />
              <span className="text-ark-cyan font-semibold">Use Optimization</span>
            </label>
          </div>

          <div className="col-span-2">
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={config.performance.enable_debug_logging}
                onChange={(e) => handlePerformanceChange('enable_debug_logging', e.target.checked)}
                className="w-5 h-5"
              />
              <span className="text-ark-cyan font-semibold">Enable Debug Logging</span>
            </label>
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-2xl font-bold text-ark-cyan mb-6">Active Mods</h3>
        <div>
          <label className="block text-sm font-semibold text-ark-cyan mb-2">
            Mod IDs (comma-separated)
          </label>
          <textarea
            value={config.mods.active_mods.join(', ')}
            onChange={(e) => handleModsChange(e.target.value)}
            rows={4}
            placeholder="Enter mod IDs separated by commas. E.g: 955131, 1102729, 1306435"
            className="w-full bg-ark-dark border border-ark-cyan/30 rounded px-4 py-2 text-white focus:outline-none focus:border-ark-cyan font-mono text-sm"
          />
          <p className="text-gray-400 text-xs mt-2">
            Active mods: {config.mods.active_mods.length} mod(s)
          </p>
        </div>
      </div>
    </div>
  )
}
