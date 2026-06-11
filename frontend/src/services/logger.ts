/**
 * Centralized logging service for debugging
 * Logs to browser console, localStorage, and sends to backend for file storage
 */

export type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR'

interface LogEntry {
  timestamp: string
  level: LogLevel
  message: string
  data?: any
}

class LogService {
  private logs: LogEntry[] = []
  private isDev = import.meta.env.DEV
  private logQueue: string[] = []
  private isFlushingLogs = false

  log(level: LogLevel, message: string, data?: any) {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      data,
    }

    this.logs.push(entry)

    // Console output
    const prefix = `[${entry.timestamp}] [${level}]`
    const logFn = level === 'ERROR' ? console.error : level === 'WARN' ? console.warn : console.log
    if (data) {
      logFn(`${prefix} ${message}`, data)
    } else {
      logFn(`${prefix} ${message}`)
    }

    // Format for file output
    const fileLog = `[${entry.timestamp}] [${level}] ${message}${data ? ' | ' + JSON.stringify(data).substring(0, 200) : ''}`

    // Queue for batch writing
    this.logQueue.push(fileLog)

    // Save to localStorage for persistence
    try {
      localStorage.setItem('app_logs', JSON.stringify(this.logs.slice(-100))) // Keep last 100
    } catch (e) {
      console.error('Failed to save logs to localStorage', e)
    }

    // Flush logs periodically
    this.flushLogsIfNeeded()
  }

  private async flushLogsIfNeeded() {
    if (this.isFlushingLogs || this.logQueue.length < 5) {
      return
    }

    this.isFlushingLogs = true
    try {
      // Send logs to backend via fetch (Tauri backend handles the write)
      const logsToSend = [...this.logQueue]
      this.logQueue = []

      await fetch('/api/logs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ logs: logsToSend }),
      }).catch(() => {
        // Silently fail if backend not available (dev mode)
        this.logQueue = [...logsToSend, ...this.logQueue]
      })
    } finally {
      this.isFlushingLogs = false
    }
  }

  async flushAll() {
    if (this.logQueue.length > 0) {
      await this.flushLogsIfNeeded()
    }
  }

  debug(message: string, data?: any) {
    this.log('DEBUG', message, data)
  }

  info(message: string, data?: any) {
    this.log('INFO', message, data)
  }

  warn(message: string, data?: any) {
    this.log('WARN', message, data)
  }

  error(message: string, data?: any) {
    this.log('ERROR', message, data)
  }

  getLogs(): LogEntry[] {
    return this.logs
  }

  getLogsAsText(): string {
    return this.logs
      .map((log) => `[${log.timestamp}] [${log.level}] ${log.message}${log.data ? ` | ${JSON.stringify(log.data)}` : ''}`)
      .join('\n')
  }

  clearLogs() {
    this.logs = []
    localStorage.removeItem('app_logs')
  }
}

export const logger = new LogService()

// Load persisted logs
try {
  const saved = localStorage.getItem('app_logs')
  if (saved) {
    console.log('Loaded persisted logs from localStorage')
  }
} catch (e) {
  console.error('Failed to load persisted logs', e)
}
