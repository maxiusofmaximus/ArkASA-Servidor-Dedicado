/**
 * useAutoSave
 *
 * Debounced auto-save of the config to the Tauri backend.
 * Runs `DELAY_MS` after the last config change, but only when `enabled` is true.
 */
import { useEffect, useRef } from 'react'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'
import type { ServerConfig } from '../types'

const DELAY_MS = 1_500

export function useAutoSave(config: ServerConfig | null, enabled: boolean): void {
  const timer = useRef<ReturnType<typeof setTimeout>>()

  useEffect(() => {
    if (!config || !enabled) return
    clearTimeout(timer.current)
    timer.current = setTimeout(() => {
      invoke('save_config', { config }).catch((err) =>
        logger.warn('Auto-save failed', err)
      )
    }, DELAY_MS)
    return () => clearTimeout(timer.current)
  }, [config, enabled])
}
