/**
 * useTauriEvents
 *
 * Wires Tauri window/tray lifecycle events:
 *   - onCloseRequested: save config, then hide or destroy based on minimizeToTray
 *   - tray-quit:        save config and exit cleanly
 *
 * Both use a ref to the latest config so they always see the current value
 * without re-registering the listener on every config change.
 */
import { useEffect, useRef } from 'react'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'
import type { ServerConfig } from '../types'

interface Options {
  config:          ServerConfig | null
  minimizeToTray:  boolean
}

export function useTauriEvents({ config, minimizeToTray }: Options): void {
  const configRef        = useRef(config)
  const minimizeRef      = useRef(minimizeToTray)

  // Keep refs current without re-running the effects below
  useEffect(() => { configRef.current = config },        [config])
  useEffect(() => { minimizeRef.current = minimizeToTray }, [minimizeToTray])

  // Sync minimize-to-tray preference to Rust
  useEffect(() => {
    invoke('set_minimize_to_tray', { enabled: minimizeToTray }).catch(() => {})
  }, [minimizeToTray])

  // Save config when the window close button is pressed
  useEffect(() => {
    let unlisten: (() => void) | undefined
    import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
      getCurrentWindow()
        .onCloseRequested(async (event) => {
          event.preventDefault()
          const cfg = configRef.current
          if (cfg) {
            try { await invoke('save_config', { config: cfg }) }
            catch (err) { logger.warn('Close-save failed', err) }
          }
          if (minimizeRef.current) {
            await getCurrentWindow().hide()
          } else {
            await getCurrentWindow().destroy()
          }
        })
        .then((fn) => { unlisten = fn })
    }).catch(() => { /* not in Tauri */ })
    return () => unlisten?.()
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // Save config when "Salir" is chosen from the tray menu
  useEffect(() => {
    let unlisten: (() => void) | undefined
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen('tray-quit', async () => {
        const cfg = configRef.current
        if (cfg) {
          try { await invoke('save_config', { config: cfg }) } catch {}
        }
        await invoke('quit_app')
      }).then((fn) => { unlisten = fn })
    }).catch(() => {})
    return () => unlisten?.()
  }, []) // eslint-disable-line react-hooks/exhaustive-deps
}
