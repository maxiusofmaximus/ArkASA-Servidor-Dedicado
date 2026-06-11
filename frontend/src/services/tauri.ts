/**
 * Safe Tauri service that handles both dev and production modes
 */

import { logger } from './logger'

let invokeFunction: ((cmd: string, args?: any) => Promise<any>) | null = null
let isTauriAvailable = false

/**
 * Initialize Tauri invoke function
 * Handles both dev mode (mock) and production mode (real Tauri)
 */
export async function initializeTauri() {
  logger.info('Initializing Tauri connection...')

  // Check if we're in Tauri context by checking for __TAURI__ global
  const isTauriContext = typeof window !== 'undefined' && '__TAURI__' in window

  if (!isTauriContext) {
    logger.info('Not in Tauri context - using mock implementation')
    invokeFunction = createMockInvoke()
    isTauriAvailable = false
    return false
  }

  try {
    // We're in Tauri context, try to import the real invoke
    const { invoke } = await import('@tauri-apps/api/core')

    if (!invoke || typeof invoke !== 'function') {
      throw new Error('invoke is not a function')
    }

    logger.info('Tauri invoke loaded successfully')
    invokeFunction = invoke
    isTauriAvailable = true
    return true
  } catch (error) {
    logger.warn('Failed to load Tauri invoke, falling back to mock', error)
    logger.info('Running in dev mode - will use mock implementation')

    // Fallback for dev mode
    invokeFunction = createMockInvoke()
    isTauriAvailable = false
    return false
  }
}

/**
 * Create mock implementation for dev mode
 */
function createMockInvoke() {
  logger.info('Setting up mock Tauri invoke for development')

  return async (command: string, args?: any) => {
    logger.debug(`Mock invoke called: ${command}`, args)

    // Mock responses for development
    const mocks: Record<string, any> = {
      get_default_config: {
        identification: {
          session_name: 'Test ARK Server',
          admin_password: 'password123',
          server_map: 'TheIsland_WP',
        },
        network: {
          port: 7777,
          query_port: 27015,
          rcon_port: 27020,
        },
        gameplay: {
          max_players: 32,
          difficulty: 4.6,
        },
        multipliers: {
          xp_multiplier: 1.0,
          harvest_multiplier: 1.0,
          taming_speed_multiplier: 1.0,
          breeding_speed_multiplier: 1.0,
        },
        paths: {
          saved_dir: 'C:\\Game\\Saved',
          logs_dir: 'C:\\Game\\Logs',
        },
      },
      server_status: {
        running: false,
        process_id: null,
        uptime_seconds: 0,
      },
      get_server_logs: ['[INFO] Mock server started', '[WARN] Mock warning'],
      get_server_metrics: { cpu: 5, memory: 256, fps: 60 },
    }

    const response = mocks[command]
    if (response) {
      logger.info(`Mock response for ${command}`, response)
      return response
    }

    logger.warn(`No mock implementation for command: ${command}`)
    return { mock: true, command }
  }
}

/**
 * Invoke a Tauri command
 */
export async function invoke(command: string, args?: any): Promise<any> {
  if (!invokeFunction) {
    throw new Error('Tauri not initialized. Call initializeTauri() first.')
  }

  logger.debug(`Invoking command: ${command}`, args)

  try {
    const result = await invokeFunction(command, args)
    logger.info(`Command ${command} succeeded`, result)
    return result
  } catch (error) {
    logger.error(`Command ${command} failed`, error)
    throw error
  }
}

/**
 * Check if running in Tauri
 */
export function isTauri(): boolean {
  return isTauriAvailable
}

/**
 * Get Tauri status for debugging
 */
export function getTauriStatus() {
  return {
    available: isTauriAvailable,
    initialized: invokeFunction !== null,
    mode: isTauriAvailable ? 'production' : 'development',
  }
}
