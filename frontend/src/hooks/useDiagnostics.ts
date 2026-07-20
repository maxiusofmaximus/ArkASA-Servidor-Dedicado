/**
 * useDiagnostics
 *
 * Calls Rust's `diagnose_server_list` to run the three server-side
 * health checks (Culture=en in GameUserSettings.ini, EOS trust-root
 * certificate, Steam install build-id).  Includes an opt-in `repair=true`
 * pass that auto-fixes the first two (steam validate is informational
 * only — never run unattended).  Polling interval is generous (default
 * 60s) because the checks are network/disk-bound, not high-frequency.
 */
import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'

export interface DiagCheck {
  key:      string
  label:    string
  status:   string
  detail:   string
  repaired: boolean
}

export interface DiagReport {
  checks:     DiagCheck[]
  summary:    string
  overall_ok: boolean
}

export interface DiagnosticsStatus {
  serverDir:    string
  steamCmdDir:  string
}

export interface DiagnosticsState {
  serverDir:    string
  steamCmdDir:  string
  report:       DiagReport | null
  running:      boolean
  repairing:    boolean
  error:        string | null
  lastTickAt:   number | null
  run:          (opts?: { repair?: boolean }) => Promise<void>
}

export function useDiagnostics(status: DiagnosticsStatus | null): DiagnosticsState {
  const [report, setReport]       = useState<DiagReport | null>(null)
  const [running, setRunning]     = useState(false)
  const [repairing, setRepairing] = useState(false)
  const [error, setError]         = useState<string | null>(null)
  const [lastTickAt, setLast]     = useState<number | null>(null)
  const aliveRef = useRef(true)

  const run = useCallback(async (opts?: { repair?: boolean }) => {
    if (!status?.serverDir || !status?.steamCmdDir) {
      setError('Server path is not configured. Set it in Options → General → Paths first.')
      return
    }
    const isRepair = !!opts?.repair
    if (isRepair) setRepairing(true)
    else setRunning(true)
    setError(null)
    try {
      const res = await invoke<DiagReport>('diagnose_server_list', {
        serverDir:    status.serverDir,
        steamCmdDir:  status.steamCmdDir,
        repair:       isRepair,
      })
      if (aliveRef.current) {
        setReport(res)
        setLast(Date.now())
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      logger.error('diagnose_server_list failed', msg)
      if (aliveRef.current) setError(msg)
    } finally {
      if (aliveRef.current) {
        setRunning(false)
        setRepairing(false)
      }
    }
  }, [status?.serverDir, status?.steamCmdDir])

  useEffect(() => {
    aliveRef.current = true
    return () => {
      aliveRef.current = false
    }
  }, [])

  return {
    serverDir:    status?.serverDir ?? '',
    steamCmdDir:  status?.steamCmdDir ?? '',
    report,
    running,
    repairing,
    error,
    lastTickAt,
    run,
  }
}
