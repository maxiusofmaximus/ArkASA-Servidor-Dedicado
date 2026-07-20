import React from 'react'
import { logger } from '../services/logger'

interface ErrorBoundaryProps {
  children: React.ReactNode
}

interface ErrorBoundaryState {
  error: Error | null
}

/**
 * Glass-box safety net for the React tree.
 *
 * Throwable React errors (rendering, lifecycle, Tauri invoke crashes) used
 * to leave the user staring at a flat `bg-ark-dark` page — the classic
 * "blue screen of death" — because the only thing they got from the App
 * was `<ArkLayout>` with `renderPage()` returning `null`. This boundary
 * surfaces the message inline so the user can copy it and keep working.
 */
export default class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error }
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    logger.error('React tree crashed', { error, info })
  }

  render() {
    if (this.state.error) {
      const err = this.state.error
      return (
        <div
          className="ark-panel rounded-lg m-6 p-6 max-w-2xl mx-auto"
          style={{ border: '1px solid rgba(248,113,113,0.45)' }}
        >
          <p className="text-red-400 text-sm font-bold tracking-widest uppercase">
            ⚠ React tree crashed
          </p>
          <p className="text-ark-cyan/40 text-[10px] font-mono mt-1">
            {err.name}: {err.message}
          </p>
          <pre
            className="text-ark-cyan/55 text-[10px] font-mono leading-relaxed mt-3 whitespace-pre-wrap overflow-y-auto"
            style={{ maxHeight: '40vh' }}
          >
            {err.stack ?? String(err)}
          </pre>
          <button
            onClick={() => location.reload()}
            className="mt-4 ark-action-btn text-[10px] tracking-widest"
          >
            RELOAD WINDOW
          </button>
        </div>
      )
    }
    return this.props.children
  }
}
