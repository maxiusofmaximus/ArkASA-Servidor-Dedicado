/**
 * useServerLifecycle
 *
 * Encapsulates all server-running state and the start/stop handlers so
 * App.tsx stays lean (SRP).  Also owns the crash-detection poll.
 */
import { useState, useEffect, useCallback } from 'react'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'
import { useBackupStore } from '../stores/backupStore'
import type { ServerConfig } from '../types'

interface Options {
  config:     ServerConfig | null
  setSaving:  (v: boolean) => void
  /** Called with a non-null string to show a transient error banner. */
  setError:   (msg: string | null) => void
}

export interface ServerLifecycle {
  serverRunning:    boolean
  stubsRunning:     boolean
  isServerStarting: boolean
  isServerStopping: boolean
  handleStartServer: () => Promise<void>
  handleStopServer:  () => Promise<void>
}

export function useServerLifecycle({ config, setSaving, setError }: Options): ServerLifecycle {
  const [serverRunning,    setServerRunning]    = useState(false)
  const [stubsRunning,     setStubsRunning]     = useState(false)
  const [isServerStarting, setIsServerStarting] = useState(false)
  const [isServerStopping, setIsServerStopping] = useState(false)

  const { onDemandEnabled, onDemandMaps, autoShutdownMin } = useBackupStore()

  // ── Start ────────────────────────────────────────────────────────────────────
  const handleStartServer = useCallback(async () => {
    if (!config || isServerStarting || serverRunning) return

    try {
      setIsServerStarting(true)
      setError(null)

      // Validate active mods before launch (non-blocking warning)
      const activeMods = config.mods?.active_mods ?? []
      if (activeMods.length > 0) {
        try {
          const unavailable = await invoke<string[]>('check_mods_available', { modIds: activeMods })
          if (unavailable.length > 0) {
            const ok = confirm(
              `⚠️ Los siguientes mods no están disponibles en CurseForge:\n\n${unavailable.join(', ')}\n\n¿Continuar de todos modos?`
            )
            if (!ok) return
          }
        } catch { /* ignore validation errors — don't block launch */ }
      }

      // Auto-save so INI files are up to date before launch
      setSaving(true)
      await invoke('save_config', { config })
      setSaving(false)

      const maps = config.cluster_maps?.length ? config.cluster_maps : ['TheIsland_WP']

      // Partition maps into always-on vs dormant stubs
      const normalIndices:  number[] = []
      const dormantIndices: number[] = []
      maps.forEach((mapId, i) => {
        if (onDemandEnabled && onDemandMaps.includes(mapId)) dormantIndices.push(i)
        else normalIndices.push(i)
      })

      // Start on-demand stubs (just binds ports — very fast)
      for (const idx of dormantIndices) {
        try {
          await invoke('enable_on_demand', { config, mapIndex: idx, autoShutdownMin })
          logger.info(`On-demand stub started for map index ${idx}`)
        } catch (err) {
          logger.warn(`Failed to start stub for map index ${idx}`, err)
        }
      }

      // Launch always-on maps
      if (normalIndices.length > 0) {
        const normalConfig: ServerConfig = {
          ...config,
          cluster_maps: normalIndices.map((i) => maps[i]),
        }
        const msg = await invoke<string>('start_server', { config: normalConfig })
        logger.info('Server start result', msg)
        setServerRunning(true)
      }

      // Track stub-only mode separately to avoid false crash alerts
      if (dormantIndices.length > 0 && normalIndices.length === 0) {
        setStubsRunning(true)
      }

      setError(null)
    } catch (err) {
      setSaving(false)
      const msg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to start server', err)
      setError(`Server start failed: ${msg}`)
    } finally {
      setIsServerStarting(false)
    }
  }, [config, isServerStarting, serverRunning, onDemandEnabled, onDemandMaps, autoShutdownMin, setSaving, setError])

  // ── Stop ─────────────────────────────────────────────────────────────────────
  const handleStopServer = useCallback(async () => {
    if (isServerStopping) return

    try {
      setIsServerStopping(true)

      // 1. RCON graceful shutdown first (saveworld + doexit) for always-on servers.
      //    This must run BEFORE signaling the stubs so stop_server gets a chance
      //    to do RCON while the ARK process is still alive.
      try {
        const msg = await invoke<string>('stop_server', { config })
        logger.info('Server stop result', msg)
      } catch (err) {
        logger.warn('stop_server RCON error (continuing with stub teardown)', err)
      }

      // 2. Signal on-demand stubs to do their own RCON graceful shutdown.
      //    disable_all_on_demand just sends the shutdown signal — each stub
      //    task will do saveworld+doexit internally before exiting.
      await invoke('disable_all_on_demand').catch(() => {})

      setError(null)
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      logger.warn('handleStopServer error', err)
      setError(`Server stop failed: ${msg}`)
    } finally {
      setServerRunning(false)
      setStubsRunning(false)
      setIsServerStopping(false)
    }
  }, [config, isServerStopping, setError])

  // ── Crash-detection poll (every 5 s while ARK should be running) ────────────
  useEffect(() => {
    if (!serverRunning) return
    const interval = setInterval(async () => {
      try {
        const running = await invoke<boolean>('is_server_running')
        if (!running) {
          logger.info('Server process no longer detected — updating UI state')
          setServerRunning(false)
          setError('El servidor se detuvo inesperadamente')
        }
      } catch { /* ignore transient poll errors */ }
    }, 5_000)
    return () => clearInterval(interval)
  }, [serverRunning]) // eslint-disable-line react-hooks/exhaustive-deps

  return {
    serverRunning,
    stubsRunning,
    isServerStarting,
    isServerStopping,
    handleStartServer,
    handleStopServer,
  }
}
