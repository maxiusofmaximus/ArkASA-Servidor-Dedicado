/**
 * useInternetStatus
 *
 * Polls the Rust `check_internet` command every 10 s.
 * Exposes { online, loading } so callers can disable actions
 * that require connectivity and surface a banner to the user.
 */
import { useState, useEffect, useCallback } from 'react'
import { invoke } from '../services/tauri'

export interface InternetStatus {
  /** true if at least one probe endpoint returned 2xx/3xx within 3s */
  online: boolean
  /** transient network in flight */
  loading: boolean
  /** last probe timestamp (ms) */
  lastCheckedAt: number | null
  /** force a refresh now (used when user clicks Retry) */
  refresh: () => Promise<void>
}

const PROBE_INTERVAL_MS = 10_000

export function useInternetStatus(): InternetStatus {
  const [online, setOnline] = useState<boolean>(true) // optimistic until first probe
  const [loading, setLoading] = useState<boolean>(false)
  const [lastCheckedAt, setLastCheckedAt] = useState<number | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      const result = await invoke<boolean>('check_internet')
      setOnline(result)
    } catch {
      // Any error counts as offline
      setOnline(false)
    } finally {
      setLoading(false)
      setLastCheckedAt(Date.now())
    }
  }, [])

  useEffect(() => {
    void refresh()
    const interval = setInterval(refresh, PROBE_INTERVAL_MS)
    const onOnline = () => void refresh()
    const onOffline = () => setOnline(false)
    window.addEventListener('online', onOnline)
    window.addEventListener('offline', onOffline)
    return () => {
      clearInterval(interval)
      window.removeEventListener('online', onOnline)
      window.removeEventListener('offline', onOffline)
    }
  }, [refresh])

  return { online, loading, lastCheckedAt, refresh }
}
