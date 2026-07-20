/**
 * useServerVersion
 *
 * Polls the Rust `check_server_version` command to detect if the local
 * ARK install is behind Steam's public buildid.  Exposes
 * { info, loading, updating, refresh, runUpdate } so callers can display a
 * "🟢 Up to date" / "🔴 Outdated" badge and trigger an on-demand update.
 *
 * Concurrency:
 *  - Single in-flight refresh at a time: if a refresh starts and a second
 *    is requested, the second is coalesced into the first (returns the
 *    same promise).  This prevents multiple SteamCMD subprocesses from
 *    being spawned in parallel when sources call refresh() back-to-back
 *    (e.g. the badge mounts, the GeneralTab mounts and the lifecycle
 *    auto-update path fires all at once).
 *  - Minimum gap of 30s between consecutive refreshes.  Anything sooner
 *    returns the cached `info` synchronously instead of forking steamcmd
 *    again.  Each steamcmd invocation is otherwise a 4–10s window where
 *    ARK players see a CMD box pop up (SteamCMD is a console app).
 *  - Event-driven: callers should prefer `listen('server://version')`
 *    bulk updates; the 60s fallback only fires when we haven't heard
 *    from the backend in a while.
 */
import { useState, useEffect, useCallback, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'
import type { ServerConfig, ServerVersionInfo } from '../types'

const MIN_REFRESH_GAP_MS = 30_000
const IDLE_FALLBACK_MS   = 60_000
const MISSED_GRACE_MS    = 45_000

export interface VersionStatus {
  info:     ServerVersionInfo | null
  loading:  boolean
  updating: boolean
  refresh:  () => Promise<void>
  runUpdate: () => Promise<void>
}

export function useServerVersion(config: ServerConfig | null): VersionStatus {
  const [info, setInfo]               = useState<ServerVersionInfo | null>(null)
  const [loading, setLoading]         = useState(false)
  const [updating, setUpdating]       = useState(false)
  const liveInfoRef                   = useRef<ServerVersionInfo | null>(null)
  const aliveRef                      = useRef(true)
  const lastRefreshAt                 = useRef(0)
  const inflightRefresh               = useRef<Promise<void> | null>(null)

  // Track latest info in a ref so the debounce can read the cached value
  // without depending on it for `refresh` (which would invalidate the
  // useCallback on every backend emit).
  liveInfoRef.current = info

  const refresh = useCallback(async () => {
    if (!config) { setInfo(null); return }
    const now = Date.now()
    if (inflightRefresh.current) return inflightRefresh.current
    if (now - lastRefreshAt.current < MIN_REFRESH_GAP_MS && liveInfoRef.current) {
      return Promise.resolve()
    }
    lastRefreshAt.current = now
    setLoading(true)
    const p = (async () => {
      try {
        const v = await invoke<ServerVersionInfo>('check_server_version', { config })
        if (aliveRef.current) {
          setInfo(v)
          liveInfoRef.current = v
        }
      } catch (err) {
        logger.warn('check_server_version failed', err)
      } finally {
        inflightRefresh.current = null
        if (aliveRef.current) setLoading(false)
      }
    })()
    inflightRefresh.current = p
    return p
  }, [config])

  const runUpdate = useCallback(async () => {
    if (!config) return
    setUpdating(true)
    try {
      await invoke<string>('update_server', { config })
      // Force refresh after a real update so the badge flips to current
      // immediately rather than waiting out the debounce.
      lastRefreshAt.current = 0
      await refresh()
    } catch (err) {
      throw err
    } finally {
      if (aliveRef.current) setUpdating(false)
    }
  }, [config, refresh])

  useEffect(() => {
    aliveRef.current = true
    let disposed = false
    let unlisten: (() => void) | undefined
    let fallbackTimer: ReturnType<typeof setTimeout> | undefined
    const lastEventAt = { current: 0 }
    const onVersion = (event: { payload: ServerVersionInfo }) => {
      lastEventAt.current = Date.now()
      // Backend-driven updates bypass the debounce entirely (these emissions
      // arrive after a real update_server, not from passive probing).
      lastRefreshAt.current = Date.now()
      if (aliveRef.current) {
        setInfo(event.payload)
        liveInfoRef.current = event.payload
        setLoading(false)
      }
    }
    const armFallback = () => {
      if (disposed) return
      fallbackTimer = setTimeout(async () => {
        if (Date.now() - lastEventAt.current >= MISSED_GRACE_MS) await refresh()
        armFallback()
      }, IDLE_FALLBACK_MS)
    }

    void refresh()
    void listen<ServerVersionInfo>('server://version', onVersion).then((fn) => {
      if (disposed) fn()
      else unlisten = fn
    }).catch(() => {})
    armFallback()
    return () => {
      disposed = true
      aliveRef.current = false
      if (fallbackTimer) clearTimeout(fallbackTimer)
      unlisten?.()
    }
  }, [refresh])

  return { info, loading, updating, refresh, runUpdate }
}
