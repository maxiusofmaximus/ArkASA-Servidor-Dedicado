/**
 * useServerLifecycle
 *
 * Encapsulates all server-running state and the start/stop handlers so
 * App.tsx stays lean (SRP).  Also owns the crash-detection poll.
 */
import { useState, useEffect, useCallback, useMemo } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'
import { useBackupStore } from '../stores/backupStore'
import type { ServerConfig, MapInstanceStatus, ServerStatusEvent } from '../types'

interface LifecycleEvent {
  phase: 'starting' | 'running' | 'stopping' | 'stopped' | 'crashed' | 'error'
  observed_at_ms: number
}

interface Options {
  config:     ServerConfig | null
  setSaving:  (v: boolean) => void
  /** Called with a non-null string to show a transient error banner. */
  setError:   (msg: string | null) => void
  /** Used to block server start when internet is unavailable. */
  online?:    boolean
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

export function useServerLifecycle({ config, setSaving, setError, online = true }: Options): ServerLifecycle {
  const [mapStatuses,      setMapStatuses]      = useState<MapInstanceStatus[]>([])
  const [stubsRunning,     setStubsRunning]     = useState(false)
  const [isServerStarting, setIsServerStarting] = useState(false)
  const [isServerStopping, setIsServerStopping] = useState(false)
  const [_startingMaps,    setStartingMaps]     = useState<Set<number>>(new Set())
  const [_stoppingMaps,    setStoppingMaps]     = useState<Set<number>>(new Set())

  const { onDemandEnabled, onDemandMaps, autoShutdownMin, clusterStartDelaySec, recordModsActive } = useBackupStore()

  const maps = useMemo(
    () => config?.cluster_maps?.length ? config.cluster_maps : ['TheIsland_WP'],
    [config?.cluster_maps]
  )
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

  // ── Event-driven instance status with a slow safety fallback ───────────────
  useEffect(() => {
    if (!config) return
    let disposed = false
    let unlistenStatus: (() => void) | undefined
    let unlistenLifecycle: (() => void) | undefined
    let fallbackTimer: ReturnType<typeof setTimeout> | undefined
    const lastEventAt = { current: 0 }

    const onStatus = (event: { payload: ServerStatusEvent }) => {
      lastEventAt.current = Date.now()
      setMapStatuses(event.payload.maps)
      if (event.payload.running) setIsServerStopping(false)
    }
    const onLifecycle = (event: { payload: LifecycleEvent }) => {
      lastEventAt.current = Date.now()
      const phase = event.payload.phase
      setIsServerStarting(phase === 'starting')
      setIsServerStopping(phase === 'stopping')
      if (phase === 'crashed') setError('El proceso del servidor terminó inesperadamente.')
      if (phase === 'error') setError('El backend reportó un error del servidor.')
    }
    const armFallback = () => {
      if (disposed) return
      fallbackTimer = setTimeout(async () => {
        if (Date.now() - lastEventAt.current >= 35_000) await refreshStatuses()
        armFallback()
      }, 30_000)
    }

    void refreshStatuses()
    void listen<ServerStatusEvent>('server://status', onStatus).then((fn) => {
      if (disposed) fn()
      else unlistenStatus = fn
    }).catch(() => {})
    void listen<LifecycleEvent>('server://lifecycle', onLifecycle).then((fn) => {
      if (disposed) fn()
      else unlistenLifecycle = fn
    }).catch(() => {})
    armFallback()

    return () => {
      disposed = true
      if (fallbackTimer) clearTimeout(fallbackTimer)
      unlistenStatus?.()
      unlistenLifecycle?.()
    }
  }, [config, refreshStatuses, setError])

  // ── Start ────────────────────────────────────────────────────────────────────
  const handleStartServer = useCallback(async (mapIndex?: number) => {
    if (!config || isServerStarting) return

    // Block start when offline — ARK servers cannot run without internet.
    // The operator can opt out in Options → General → Internet.
    const allowOffline = config.network?.allow_start_without_internet ?? false
    if (!online && !allowOffline) {
      setError('No internet connection — cannot start servers. Reconnect and try again, or disable the check in Options → General → Internet.')
      return
    }
    if (!online && allowOffline) {
      // Surface a one-shot info so the user knows they bypassed the guard.
      setError('⚠ Started without internet — ARK may fail to log into EOS/Steam servers until connection is restored.')
    }

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

      // ── Auto-update before start ────────────────────────────────────────
      // The Steam buildid must match before ARK can show up in the official
      // server browser. Compare against Steam once, and if a fresh build is
      // available, run SteamCMD before `start_server` actually fires.
      if (config.network?.auto_update_before_start && online) {
        try {
          const info = await invoke<import('../types').ServerVersionInfo>('check_server_version', { config })
          if (info.needs_update) {
            logger.info(
              `Auto-update: local buildid=${info.local_buildid} < latest=${info.latest_buildid}. Running SteamCMD update…`
            )
            await invoke<string>('update_server', { config })
            logger.info('Auto-update finished')
          }
        } catch (err) {
          // Auto-update is best-effort — if SteamCMD is missing or can't
          // reach Steam, just keep going so the user isn't blocked.
          logger.warn('Auto-update check failed; continuing without update', err)
        }
      }

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
    config, isServerStarting, online, mapStatuses, maps, isCluster,
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
