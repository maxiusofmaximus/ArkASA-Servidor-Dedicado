/**
 * Centralized logging service for debugging
 * Logs to both console and a visible in-app log file
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

    // Save to localStorage for persistence
    try {
      localStorage.setItem('app_logs', JSON.stringify(this.logs.slice(-100))) // Keep last 100
    } catch (e) {
      console.error('Failed to save logs to localStorage', e)
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
