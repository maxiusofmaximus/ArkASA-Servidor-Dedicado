import type { ServerConfig } from '../types'
import { invoke } from './tauri'

export interface TauriCommandService {
  loadConfigOrDefault(): Promise<ServerConfig>
  getDefaultConfig(): Promise<ServerConfig>
  saveConfig(config: ServerConfig): Promise<void>
  parseConfigFromToml(tomlStr: string): Promise<ServerConfig>
  mergeConfigFromIni(config: ServerConfig, iniContent: string): Promise<ServerConfig>
  validateConfig(config: ServerConfig): Promise<any>
  readTextFile(path: string): Promise<string>
  writeTextFile(path: string, content: string): Promise<void>
  serverStatus(): Promise<any>
  startServer(config: ServerConfig): Promise<void>
  stopServer(): Promise<void>
  restartServer(config: ServerConfig): Promise<void>
  serverLogs(): Promise<string[]>
  serverMetrics(): Promise<any>
  backupSaves(): Promise<string>
  setMinimizeToTray(enabled: boolean): Promise<void>
  quitApp(): Promise<void>
  enableOnDemand(config: ServerConfig, mapIndex: number, autoShutdownMin: number): Promise<void>
  disableAllOnDemand(): Promise<void>
  startPing(ip: string): Promise<void>
  stopPing(): Promise<void>
  openExternalUrl(url: string): Promise<void>
  detectIps(): Promise<any>
}

export class TauriCommandServiceImpl implements TauriCommandService {
  async loadConfigOrDefault(): Promise<ServerConfig> {
    return invoke<ServerConfig>('load_config_or_default')
  }

  async getDefaultConfig(): Promise<ServerConfig> {
    return invoke<ServerConfig>('get_default_config')
  }

  async saveConfig(config: ServerConfig): Promise<void> {
    await invoke('save_config', { config })
  }

  async parseConfigFromToml(tomlStr: string): Promise<ServerConfig> {
    return invoke<ServerConfig>('parse_config_from_toml', { tomlStr })
  }

  async mergeConfigFromIni(config: ServerConfig, iniContent: string): Promise<ServerConfig> {
    return invoke<ServerConfig>('merge_config_from_ini', { config, iniContent })
  }

  async validateConfig(config: ServerConfig): Promise<any> {
    return invoke('validate_config', { config })
  }

  async readTextFile(path: string): Promise<string> {
    return invoke<string>('read_text_file', { path })
  }

  async writeTextFile(path: string, content: string): Promise<void> {
    await invoke('write_text_file', { path, content })
  }

  async serverStatus(): Promise<any> {
    return invoke('server_status')
  }

  async startServer(config: ServerConfig): Promise<void> {
    await invoke('start_server', { config })
  }

  async stopServer(): Promise<void> {
    await invoke('stop_server')
  }

  async restartServer(config: ServerConfig): Promise<void> {
    await invoke('restart_server', { config })
  }

  async serverLogs(): Promise<string[]> {
    return invoke('get_server_logs')
  }

  async serverMetrics(): Promise<any> {
    return invoke('get_server_metrics')
  }

  async backupSaves(): Promise<string> {
    return invoke<string>('backup_saves')
  }

  async setMinimizeToTray(enabled: boolean): Promise<void> {
    await invoke('set_minimize_to_tray', { enabled })
  }

  async quitApp(): Promise<void> {
    await invoke('quit_app')
  }

  async enableOnDemand(config: ServerConfig, mapIndex: number, autoShutdownMin: number): Promise<void> {
    await invoke('enable_on_demand', { config, mapIndex, autoShutdownMin })
  }

  async disableAllOnDemand(): Promise<void> {
    await invoke('disable_all_on_demand')
  }

  async startPing(ip: string): Promise<void> {
    await invoke('start_ping', { ip })
  }

  async stopPing(): Promise<void> {
    await invoke('stop_ping')
  }

  async openExternalUrl(url: string): Promise<void> {
    await invoke('open_external_url', { url })
  }

  async detectIps(): Promise<any> {
    return invoke('detect_ips')
  }
}

export const tauriCommands: TauriCommandService = new TauriCommandServiceImpl()
