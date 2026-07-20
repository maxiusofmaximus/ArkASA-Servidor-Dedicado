/**
 * useServerVersion
 *
 * Polls the Rust `check_server_version` command to detect if the local
 * ARK install is behind Steam's public buildid.  Exposes
 * { info, loading, refresh, runUpdate } so callers can display a
 * "🟢 Up to date" / "🔴 Outdated" badge and trigger an on-demand update.
 */
import { useState, useEffect, useCallback, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'
import type { ServerConfig, ServerVersionInfo } from '../types'

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
  const aliveRef = useRef(true)

  const refresh = useCallback(async () => {
    if (!config) { setInfo(null); return }
    setLoading(true)
    try {
      const v = await invoke<ServerVersionInfo>('check_server_version', { config })
      if (aliveRef.current) setInfo(v)
    } catch (err) {
      logger.warn('check_server_version failed', err)
    } finally {
      if (aliveRef.current) setLoading(false)
    }
  }, [config])

  const runUpdate = useCallback(async () => {
    if (!config) return
    setUpdating(true)
    try {
      await invoke<string>('update_server', { config })
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
      if (aliveRef.current) {
        setInfo(event.payload)
        setLoading(false)
      }
    }
    const armFallback = () => {
      if (disposed) return
      fallbackTimer = setTimeout(async () => {
        if (Date.now() - lastEventAt.current >= 35_000) await refresh()
        armFallback()
      }, 30_000)
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
