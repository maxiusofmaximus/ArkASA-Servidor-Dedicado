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
import type { ServerConfig, MapInstanceStatus } from '../types'

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
  mapStatuses:      MapInstanceStatus[]
  handleStartServer: (mapIndex?: number) => Promise<void>
  handleStopServer:  (mapIndex?: number) => Promise<void>
}

export function useServerLifecycle({ config, setSaving, setError }: Options): ServerLifecycle {
  const [mapStatuses,      setMapStatuses]      = useState<MapInstanceStatus[]>([])
  const [stubsRunning,     setStubsRunning]     = useState(false)
  const [isServerStarting, setIsServerStarting] = useState(false)
  const [isServerStopping, setIsServerStopping] = useState(false)
  const [startingMaps,     setStartingMaps]     = useState<Set<number>>(new Set())
  const [stoppingMaps,     setStoppingMaps]     = useState<Set<number>>(new Set())

  const { onDemandEnabled, onDemandMaps, autoShutdownMin, clusterStartDelaySec, recordModsActive } = useBackupStore()

  const maps = config?.cluster_maps?.length ? config.cluster_maps : ['TheIsland_WP']
  const isCluster = maps.length > 1

  // Placeholder statuses until first poll
  useEffect(() => {
    if (!config) return
    setMapStatuses(
      maps.map((map_id, i) => ({
        map_index: i,
        map_id,
        map_label: map_id.trimEnd().replace(/_WP$/, '').replace(/_/g, ' '),
        running: false,
      }))
    )
  }, [config, maps])

  const refreshStatuses = useCallback(async () => {
    if (!config) return
    try {
      const statuses = await invoke<MapInstanceStatus[]>('get_cluster_instance_status', { config })
      setMapStatuses(statuses)
    } catch {
      /* ignore transient poll errors */
    }
  }, [config])

  const alwaysOnIndices = useCallback(() => {
    if (!config) return []
    return maps
      .map((mapId, i) => ({ mapId, i }))
      .filter(({ mapId }) => !onDemandEnabled || !onDemandMaps.includes(mapId))
      .map(({ i }) => i)
  }, [config, maps, onDemandEnabled, onDemandMaps])

  const serverRunning = mapStatuses.some((s) => s.running)

  // ── Poll instance status ───────────────────────────────────────────────────
  useEffect(() => {
    if (!config) return
    refreshStatuses()
    const interval = setInterval(refreshStatuses, 5_000)
    return () => clearInterval(interval)
  }, [config, refreshStatuses])

  // ── Start ────────────────────────────────────────────────────────────────────
  const handleStartServer = useCallback(async (mapIndex?: number) => {
    if (!config || isServerStarting) return

    const targetIndices: number[] = mapIndex !== undefined
      ? [mapIndex]
      : alwaysOnIndices()

    if (targetIndices.length === 0) return

    // Block if all targets already running
    const toStart = targetIndices.filter((i) => !mapStatuses.find((s) => s.map_index === i)?.running)
    if (toStart.length === 0) return

    try {
      setIsServerStarting(true)
      setError(null)

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
        } catch { /* ignore */ }
      }

      setSaving(true)
      await invoke('save_config', { config })
      setSaving(false)

      // On-demand stubs for dormant maps (full start only)
      if (mapIndex === undefined) {
        const dormantIndices = maps
          .map((mapId, i) => ({ mapId, i }))
          .filter(({ mapId }) => onDemandEnabled && onDemandMaps.includes(mapId))
          .map(({ i }) => i)

        for (const idx of dormantIndices) {
          try {
            await invoke('enable_on_demand', { config, mapIndex: idx, autoShutdownMin })
            logger.info(`On-demand stub started for map index ${idx}`)
          } catch (err) {
            logger.warn(`Failed to start stub for map index ${idx}`, err)
          }
        }

        if (dormantIndices.length > 0 && toStart.length === 0) {
          setStubsRunning(true)
        }
      }

      setStartingMaps(new Set(toStart))

      if (mapIndex !== undefined || !isCluster) {
        for (const idx of toStart) {
          const msg = await invoke<string>('start_server_instance', { config, mapIndex: idx })
          logger.info('Instance start', msg)
        }
      } else {
        const normalConfig: ServerConfig = {
          ...config,
          cluster_maps: toStart.map((i) => maps[i]),
        }
        const msg = await invoke<string>('start_server', { config: normalConfig, clusterDelaySec: clusterStartDelaySec })
        logger.info('Cluster start result', msg)
      }

      recordModsActive(config.mods?.active_mods ?? [])
      await refreshStatuses()
      setError(null)
    } catch (err) {
      setSaving(false)
      const msg = err instanceof Error ? err.message : String(err)
      logger.error('Failed to start server', err)
      setError(`Server start failed: ${msg}`)
    } finally {
      setStartingMaps(new Set())
      setIsServerStarting(false)
    }
  }, [
    config, isServerStarting, mapStatuses, maps, isCluster,
    alwaysOnIndices, onDemandEnabled, onDemandMaps, autoShutdownMin,
    clusterStartDelaySec, setSaving, setError, recordModsActive, refreshStatuses,
  ])

  // ── Stop ─────────────────────────────────────────────────────────────────────
  const handleStopServer = useCallback(async (mapIndex?: number) => {
    if (!config || isServerStopping) return

    const runningIndices = mapStatuses.filter((s) => s.running).map((s) => s.map_index)
    const targetIndices: number[] = mapIndex !== undefined
      ? [mapIndex]
      : runningIndices

    if (targetIndices.length === 0) return

    try {
      setIsServerStopping(true)
      setStoppingMaps(new Set(targetIndices))

      if (mapIndex !== undefined || (isCluster && targetIndices.length < runningIndices.length)) {
        for (const idx of targetIndices) {
          try {
            const msg = await invoke<string>('stop_server_instance', { config, mapIndex: idx })
            logger.info('Instance stop', msg)
          } catch (err) {
            logger.warn(`stop_server_instance failed for index ${idx}`, err)
          }
        }
      } else {
        try {
          const msg = await invoke<string>('stop_server', { config })
          logger.info('Server stop result', msg)
        } catch (err) {
          logger.warn('stop_server RCON error (continuing with stub teardown)', err)
        }
        await invoke('disable_all_on_demand').catch(() => {})
        setStubsRunning(false)
      }

      await refreshStatuses()
      setError(null)
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      logger.warn('handleStopServer error', err)
      setError(`Server stop failed: ${msg}`)
    } finally {
      setStoppingMaps(new Set())
      setIsServerStopping(false)
    }
  }, [config, isServerStopping, mapStatuses, isCluster, setError, refreshStatuses])

  return {
    serverRunning,
    stubsRunning,
    isServerStarting,
    isServerStopping,
    mapStatuses,
    handleStartServer,
    handleStopServer,
  }
}
