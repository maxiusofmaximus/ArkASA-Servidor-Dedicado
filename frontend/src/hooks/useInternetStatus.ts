/**
 * useInternetStatus
 *
 * Polls the Rust `check_internet` command every 10 s.
 * Exposes { online, loading } so callers can disable actions
 * that require connectivity and surface a banner to the user.
 */
import { useState, useEffect, useCallback, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '../services/tauri'
import type { InternetStatusEvent } from '../types'

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

const FAIL_THRESHOLD = 3

export function useInternetStatus(): InternetStatus {
  const [online, setOnline] = useState<boolean>(true) // optimistic until first probe
  const [loading, setLoading] = useState<boolean>(false)
  const [lastCheckedAt, setLastCheckedAt] = useState<number | null>(null)
  const failCountRef = useRef(0)

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      const result = await invoke<boolean>('check_internet')
      if (result) {
        failCountRef.current = 0
        setOnline(true)
      } else {
        failCountRef.current += 1
        if (failCountRef.current >= FAIL_THRESHOLD) {
          setOnline(false)
        }
      }
    } catch {
      failCountRef.current += 1
      if (failCountRef.current >= FAIL_THRESHOLD) {
        setOnline(false)
      }
    } finally {
      setLoading(false)
      setLastCheckedAt(Date.now())
    }
  }, [])

  useEffect(() => {
    let disposed = false
    let unlisten: (() => void) | undefined
    let fallbackTimer: ReturnType<typeof setTimeout> | undefined
    const lastEventAt = { current: 0 }

    const onStatus = (event: { payload: InternetStatusEvent }) => {
      lastEventAt.current = Date.now()
      failCountRef.current = 0
      setOnline(event.payload.online)
      setLastCheckedAt(event.payload.observed_at_ms)
      setLoading(false)
    }
    const armFallback = () => {
      if (disposed) return
      fallbackTimer = setTimeout(async () => {
        if (Date.now() - lastEventAt.current >= 35_000) await refresh()
        armFallback()
      }, 30_000)
    }

    void refresh()
    void listen<InternetStatusEvent>('internet://status', onStatus).then((fn) => {
      if (disposed) fn()
      else unlisten = fn
    }).catch(() => {})
    armFallback()
    const onOnline = () => void refresh()
    const onOffline = () => setOnline(false)
    window.addEventListener('online', onOnline)
    window.addEventListener('offline', onOffline)
    return () => {
      disposed = true
      if (fallbackTimer) clearTimeout(fallbackTimer)
      unlisten?.()
      window.removeEventListener('online', onOnline)
      window.removeEventListener('offline', onOffline)
    }
  }, [refresh])

  return { online, loading, lastCheckedAt, refresh }
}
