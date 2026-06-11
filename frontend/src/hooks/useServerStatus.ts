import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

export interface ServerStatusInfo {
  running: boolean
  process_id: number | null
  uptime_seconds: number
}

export function useServerStatus() {
  const [status, setStatus] = useState<ServerStatusInfo>({
    running: false,
    process_id: null,
    uptime_seconds: 0,
  })
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refreshStatus = useCallback(async () => {
    try {
      const result = await invoke('server_status')
      setStatus(result as ServerStatusInfo)
      setError(null)
    } catch (err) {
      setError(`Failed to get status: ${err}`)
    }
  }, [])

  const startServer = useCallback(
    async (config: any) => {
      setLoading(true)
      try {
        await invoke('start_server', { config })
        await refreshStatus()
        setError(null)
      } catch (err) {
        setError(`Failed to start server: ${err}`)
      } finally {
        setLoading(false)
      }
    },
    [refreshStatus]
  )

  const stopServer = useCallback(async () => {
    setLoading(true)
    try {
      await invoke('stop_server')
      await refreshStatus()
      setError(null)
    } catch (err) {
      setError(`Failed to stop server: ${err}`)
    } finally {
      setLoading(false)
    }
  }, [refreshStatus])

  const restartServer = useCallback(
    async (config: any) => {
      setLoading(true)
      try {
        await invoke('restart_server', { config })
        await refreshStatus()
        setError(null)
      } catch (err) {
        setError(`Failed to restart server: ${err}`)
      } finally {
        setLoading(false)
      }
    },
    [refreshStatus]
  )

  // Auto-refresh status every 5 seconds when server is running
  useEffect(() => {
    const interval = setInterval(refreshStatus, 5000)
    return () => clearInterval(interval)
  }, [refreshStatus])

  // Initial status check
  useEffect(() => {
    refreshStatus()
  }, [refreshStatus])

  return {
    status,
    loading,
    error,
    refreshStatus,
    startServer,
    stopServer,
    restartServer,
  }
}
