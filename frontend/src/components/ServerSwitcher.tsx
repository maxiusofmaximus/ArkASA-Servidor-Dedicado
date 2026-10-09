import { useEffect, useRef, useState } from 'react'
import { invoke } from '../services/tauri'
import { logger } from '../services/logger'

export interface ServerProfile {
  id: string
  name: string
  config_path: string
  created_at_unix: number
}

interface ServerSwitcherProps {
  /** Called after the active profile changes so App can reload the config. */
  onSwitched: () => void | Promise<void>
}

/**
 * Multi-server selector. Lists the profiles registered in
 * `%APPDATA%\ARK ASA Config Manager\profiles.json`, lets the operator switch,
 * create, rename and delete them. Switching just flips the active profile in
 * the backend and asks App to reload its config.
 */
export default function ServerSwitcher({ onSwitched }: ServerSwitcherProps) {
  const [profiles, setProfiles] = useState<ServerProfile[]>([])
  const [activeId, setActiveId] = useState<string>('')
  const [open, setOpen] = useState(false)
  const [busy, setBusy] = useState(false)
  const rootRef = useRef<HTMLDivElement>(null)

  const refresh = async () => {
    try {
      const [list, active] = await Promise.all([
        invoke<ServerProfile[]>('list_server_profiles'),
        invoke<string>('active_server_profile'),
      ])
      setProfiles(list ?? [])
      setActiveId(active ?? '')
    } catch (err) {
      logger.warn('Failed to load server profiles', err)
    }
  }

  useEffect(() => { void refresh() }, [])

  useEffect(() => {
    if (!open) return
    const close = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) setOpen(false)
    }
    document.addEventListener('mousedown', close)
    return () => document.removeEventListener('mousedown', close)
  }, [open])

  const active = profiles.find((p) => p.id === activeId)

  const switchTo = async (id: string) => {
    if (id === activeId) { setOpen(false); return }
    setBusy(true)
    try {
      await invoke('set_active_server_profile', { id })
      setActiveId(id)
      setOpen(false)
      await onSwitched()
    } catch (err) {
      logger.error('Failed to switch server', err)
      window.alert(`No se pudo cambiar de servidor: ${err instanceof Error ? err.message : String(err)}`)
    } finally {
      setBusy(false)
    }
  }

  const createServer = async () => {
    const name = window.prompt('Nombre del nuevo servidor:')
    if (!name || !name.trim()) return
    setBusy(true)
    try {
      const created = await invoke<ServerProfile>('create_server_profile', { name: name.trim() })
      await refresh()
      if (created?.id) await switchTo(created.id)
    } catch (err) {
      logger.error('Failed to create server', err)
      window.alert(`No se pudo crear: ${err instanceof Error ? err.message : String(err)}`)
    } finally {
      setBusy(false)
    }
  }

  const renameServer = async (p: ServerProfile) => {
    const name = window.prompt('Nuevo nombre:', p.name)
    if (!name || !name.trim()) return
    setBusy(true)
    try {
      await invoke('rename_server_profile', { id: p.id, name: name.trim() })
      await refresh()
    } catch (err) {
      logger.error('Failed to rename server', err)
    } finally {
      setBusy(false)
    }
  }

  const deleteServer = async (p: ServerProfile) => {
    if (profiles.length <= 1) { window.alert('No puedes borrar el único servidor.'); return }
    if (!window.confirm(`¿Borrar el servidor "${p.name}"? Se eliminará su carpeta de configuración.`)) return
    setBusy(true)
    try {
      await invoke('delete_server_profile', { id: p.id })
      await refresh()
      await onSwitched()
    } catch (err) {
      logger.error('Failed to delete server', err)
      window.alert(`No se pudo borrar: ${err instanceof Error ? err.message : String(err)}`)
    } finally {
      setBusy(false)
    }
  }

  return (
    <div ref={rootRef} className="relative select-none" style={{ zIndex: 40 }}>
      <button
        type="button"
        disabled={busy}
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 px-3 py-1.5 rounded border border-ark-accent/40 bg-black/40 hover:border-ark-accent transition-colors text-[11px] uppercase tracking-widest disabled:opacity-50"
        title="Cambiar de servidor"
      >
        <span className="text-ark-accent">▣</span>
        <span className="max-w-[180px] truncate">{active?.name ?? 'Servidor'}</span>
        <span className="text-ark-accent/70">▾</span>
      </button>

      {open && (
        <div className="absolute left-0 mt-1 w-80 rounded border border-ark-accent/40 bg-black/95 backdrop-blur shadow-xl">
          <ul className="max-h-72 overflow-auto py-1">
            {profiles.map((p) => (
              <li
                key={p.id}
                className={`flex items-center gap-2 px-3 py-1.5 text-[12px] hover:bg-ark-accent/10 ${
                  p.id === activeId ? 'text-ark-accent' : 'text-white/80'
                }`}
              >
                <button
                  type="button"
                  className="flex-1 text-left truncate"
                  onClick={() => void switchTo(p.id)}
                  title={p.config_path}
                >
                  {p.id === activeId ? '● ' : '○ '}{p.name}
                </button>
                <button
                  type="button"
                  className="px-1 text-white/50 hover:text-white"
                  title="Renombrar"
                  onClick={() => void renameServer(p)}
                >✎</button>
                <button
                  type="button"
                  className="px-1 text-white/50 hover:text-red-400"
                  title="Borrar"
                  onClick={() => void deleteServer(p)}
                >✕</button>
              </li>
            ))}
          </ul>
          <div className="border-t border-ark-accent/20 p-1">
            <button
              type="button"
              className="w-full text-left px-2 py-1.5 text-[12px] text-ark-accent hover:bg-ark-accent/10 rounded"
              onClick={() => void createServer()}
            >＋ Nuevo servidor…</button>
          </div>
        </div>
      )}
    </div>
  )
}
