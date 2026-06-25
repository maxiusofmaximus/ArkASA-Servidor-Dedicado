import type { ServerConfig } from '../types'
import type { TauriCommandService } from './TauriCommandService'
import { tauriCommands } from './TauriCommandService'

export interface ConfigSaveResult {
  updatedConfig?: ServerConfig
  message?: string
}

export interface ConfigSaveStrategy {
  save(content: string, config: ServerConfig): Promise<ConfigSaveResult>
}

export class TomlSaveStrategy implements ConfigSaveStrategy {
  constructor(private cmd: TauriCommandService = tauriCommands) {}

  async save(content: string, _config: ServerConfig): Promise<ConfigSaveResult> {
    const parsed = await this.cmd.parseConfigFromToml(content)
    await this.cmd.saveConfig(parsed)
    return { updatedConfig: parsed, message: 'Configuration saved' }
  }
}

export class IniSaveStrategy implements ConfigSaveStrategy {
  constructor(
    private path: string,
    private cmd: TauriCommandService = tauriCommands,
  ) {}

  async save(content: string, config: ServerConfig): Promise<ConfigSaveResult> {
    await this.cmd.writeTextFile(this.path, content)
    const merged = await this.cmd.mergeConfigFromIni(config, content)
    await this.cmd.saveConfig(merged)
    return { updatedConfig: merged, message: 'Configuration saved' }
  }
}

export class CustomFileSaveStrategy implements ConfigSaveStrategy {
  constructor(
    private path: string,
    private cmd: TauriCommandService = tauriCommands,
  ) {}

  async save(content: string, _config: ServerConfig): Promise<ConfigSaveResult> {
    await this.cmd.writeTextFile(this.path, content)
    return { message: 'File saved' }
  }
}
