import React, { useState, useEffect, useCallback, useRef } from 'react'
import { useConfigStore } from '../../stores/configStore'
import { useModsStore, type ModInfo } from '../../stores/modsStore'
import { invoke } from '../../services/tauri'

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

interface CurseForgeMod {
  id: string
  name: string
  summary: string
  download_count: number
  categories: string[]
  logo_url: string | null
  slug: string
  client_only: boolean
}

interface FetchResult {
  mods: CurseForgeMod[]
  total_count: number
  from_cache: boolean
}

function formatDownloads(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}k`
  return String(n)
}

const PAGE_SIZE = 50

// ─────────────────────────────────────────────────────────────────────────────
// Component
// ─────────────────────────────────────────────────────────────────────────────

export default function AvailableModsTab() {
  const { config, setConfig } = useConfigStore()
  const { modCache, setModInfo, apiKey: storedKey, setApiKey: storeApiKey } = useModsStore()

  // API key — initialise from persisted store so it survives tab switches
  const [apiKey, setApiKeyState] = useState<string | null>(null)
  const [apiKeyInput, setApiKeyInput] = useState('')
  const [savingKey, setSavingKey] = useState(false)
  const [keyError, setKeyError] = useState('')

  // Mods state
  const [mods, setMods] = useState<CurseForgeMod[]>([])
  const [totalCount, setTotalCount] = useState(0)
  const [loading, setLoading] = useState(false)
  const [fetchError, setFetchError] = useState('')
  const [fromCache, setFromCache] = useState(false)
  const [currentPage, setCurrentPage] = useState(0)

  // Custom ID add
  const [customId, setCustomId] = useState('')
  const [customError, setCustomError] = useState('')
  const [resolvingId, setResolvingId] = useState(false)

  // Search
  const [search, setSearch] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('All')
  const searchTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined)

  const activeMods = config?.mods?.active_mods ?? []

  // Cache helper
  const cacheModInfo = useCallback((mod: CurseForgeMod) => {
    const info: ModInfo = {
      name: mod.name,
      summary: mod.summary,
      logoUrl: mod.logo_url,
      slug: mod.slug,
      downloadCount: mod.download_count,
      categories: mod.categories,
      clientOnly: mod.client_only ?? false,
    }
    setModInfo(mod.id, info)
  }, [setModInfo])

  // ── API key: read from store first (instant), then verify with Tauri ──────
  useEffect(() => {
    if (storedKey) {
      // Have it in localStorage — sync to backend disk file so fetch works
      setApiKeyState(storedKey)
      invoke('set_curseforge_api_key', { apiKey: storedKey }).catch(() => {})
    } else {
      // First time or cleared: ask the backend
      invoke('get_curseforge_api_key')
        .then((key: unknown) => {
          const k = (key as string) ?? ''
          setApiKeyState(k)
          if (k) storeApiKey(k)
        })
        .catch(() => setApiKeyState(''))
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // ── Fetch mods ─────────────────────────────────────────────────────────────
  const fetchMods = useCallback(async (page: number, forceRefresh = false, searchQuery = '') => {
    if (!apiKey) return
    if (forceRefresh) {
      await invoke('clear_mods_cache').catch(() => {})
    }
    setLoading(true)
    setFetchError('')
    try {
      const result = await invoke('fetch_curseforge_mods', {
        pageSize: PAGE_SIZE,
        index: page * PAGE_SIZE,
        searchFilter: searchQuery || null,
      }) as FetchResult

      const incoming = result.mods
      incoming.forEach(cacheModInfo)

      if (page === 0) {
        setMods(incoming)
        setSelectedCategory('All')
      } else {
        setMods(prev => [...prev, ...incoming])
      }
      setTotalCount(result.total_count)
      setFromCache(result.from_cache)
      setCurrentPage(page)
    } catch (err: any) {
      const msg = String(err)
      if (msg === 'INVALID_API_KEY') {
        setKeyError('Clave de API inválida — verifica tu key en console.curseforge.com')
        setApiKeyState('')
        storeApiKey('')
      } else if (msg === 'NO_API_KEY' && apiKey) {
        // Backend lost the key (file deleted or first run after localStorage-only save)
        // Re-sync and retry once
        try {
          await invoke('set_curseforge_api_key', { apiKey })
          const result = await invoke('fetch_curseforge_mods', {
            pageSize: PAGE_SIZE,
            index: page * PAGE_SIZE,
            searchFilter: searchQuery || null,
          }) as FetchResult
          const incoming = result.mods
          incoming.forEach(cacheModInfo)
          if (page === 0) { setMods(incoming); setSelectedCategory('All') }
          else setMods(prev => [...prev, ...incoming])
          setTotalCount(result.total_count)
          setFromCache(result.from_cache)
          setCurrentPage(page)
        } catch {
          setFetchError('No se pudo cargar mods — verifica tu API key')
        }
      } else if (msg !== 'NO_API_KEY') {
        setFetchError(`Error al cargar mods: ${msg}`)
      }
    } finally {
      setLoading(false)
    }
  }, [apiKey, cacheModInfo, storeApiKey])

  // Initial load when key is ready
  useEffect(() => {
    if (apiKey) fetchMods(0)
  }, [apiKey]) // eslint-disable-line react-hooks/exhaustive-deps

  // Debounced server-side search
  useEffect(() => {
    if (apiKey === null || apiKey === '') return
    clearTimeout(searchTimerRef.current)
    searchTimerRef.current = setTimeout(() => {
      fetchMods(0, false, search)
    }, 600)
    return () => clearTimeout(searchTimerRef.current)
  }, [search]) // eslint-disable-line react-hooks/exhaustive-deps

  // ── Save API key ───────────────────────────────────────────────────────────
  const handleSaveKey = async () => {
    const k = apiKeyInput.trim()
    if (!k) { setKeyError('Ingresa tu API key de CurseForge'); return }
    setSavingKey(true)
    setKeyError('')
    try {
      await invoke('set_curseforge_api_key', { apiKey: k })
      storeApiKey(k)           // persist to localStorage
      setApiKeyState(k)
      setApiKeyInput('')
    } catch (err) {
      setKeyError(`Error al guardar: ${err}`)
    } finally {
      setSavingKey(false)
    }
  }

  // ── Active mods helpers ────────────────────────────────────────────────────
  const addToActive = (mod: CurseForgeMod) => {
    if (!config || activeMods.includes(mod.id)) return
    // Warn if PC-only
    if (mod.client_only) {
      if (!confirm(`⚠️ "${mod.name}" es un mod solo para PC y no funcionará en un servidor cross-platform.\n¿Agregarlo de todos modos?`)) return
    }
    // Check for same name already active (different ID, same mod)
    const dupId = activeMods.find(id => {
      const cached = modCache[id]
      return cached?.name?.toLowerCase() === mod.name.toLowerCase()
    })
    if (dupId) {
      if (!confirm(`Ya tienes "${mod.name}" activo (ID: ${dupId}).\n¿Agregar de todos modos con el ID ${mod.id}?`)) return
    }
    cacheModInfo(mod)
    setConfig({ ...config, mods: { ...config.mods, active_mods: [...activeMods, mod.id] } })
  }

  const removeFromActive = (modId: string) => {
    if (!config) return
    setConfig({ ...config, mods: { ...config.mods, active_mods: activeMods.filter(id => id !== modId) } })
  }

  // ── Add by custom ID ───────────────────────────────────────────────────────
  const addCustom = async () => {
    const id = customId.trim()
    if (!id) return
    if (!/^\d+$/.test(id)) { setCustomError('Debe ser un ID numérico de CurseForge'); return }
    if (activeMods.includes(id)) { setCustomError('Ya está activo'); return }
    setCustomError('')
    setResolvingId(true)
    try {
      const mod = await invoke('get_curseforge_mod_by_id', { modId: id }) as CurseForgeMod | null
      if (mod) {
        // addToActive already handles the name-duplicate check + caching
        addToActive(mod)
      } else {
        if (!config) return
        setConfig({ ...config, mods: { ...config.mods, active_mods: [...activeMods, id] } })
      }
      setCustomId('')
    } catch {
      if (!config) return
      setConfig({ ...config, mods: { ...config.mods, active_mods: [...activeMods, id] } })
      setCustomId('')
    } finally {
      setResolvingId(false)
    }
  }

  // Categories from loaded mods
  const categories = ['All', ...Array.from(new Set(mods.flatMap(m => m.categories))).sort()]

  // Client-side category filter (search filter already applied server-side)
  const filtered = mods.filter(m =>
    selectedCategory === 'All' || m.categories.includes(selectedCategory)
  )

  const hasMore = mods.length < totalCount && !search

  // ─────────────────────────────────────────────────────────────────────────
  // Loading state (first time, before we know the key)
  // ─────────────────────────────────────────────────────────────────────────
  if (apiKey === null) {
    return (
      <div className="px-8 py-16 flex items-center justify-center">
        <div className="text-ark-cyan/50 text-sm">Verificando API key...</div>
      </div>
    )
  }

  // ─────────────────────────────────────────────────────────────────────────
  // API key setup screen
  // ─────────────────────────────────────────────────────────────────────────
  if (apiKey === '') {
    return (
      <div className="px-8 py-10 max-w-xl">
        <div className="ark-panel rounded-lg p-6 space-y-4">
          <h2 className="text-ark-cyan text-sm font-bold tracking-widest uppercase">API Key de CurseForge requerida</h2>
          <p className="text-ark-cyan/60 text-xs leading-relaxed">
            Para navegar mods directamente desde CurseForge necesitas una API key gratuita.
          </p>
          <ol className="text-ark-cyan/55 text-xs space-y-1 list-decimal list-inside">
            <li>Ve a <button onClick={() => invoke('open_external_url', { url: 'https://console.curseforge.com' })} className="text-ark-cyan/80 hover:text-ark-cyan underline">console.curseforge.com</button></li>
            <li>Inicia sesión y crea una nueva API key (el tier gratuito es suficiente)</li>
            <li>Pégala abajo</li>
          </ol>
          <div className="flex gap-2 items-start flex-col">
            <input
              type="password"
              value={apiKeyInput}
              onChange={e => { setApiKeyInput(e.target.value); setKeyError('') }}
              onKeyDown={e => e.key === 'Enter' && handleSaveKey()}
              placeholder="$2a$10$..."
              className="w-full bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/70 placeholder-ark-cyan/25 font-mono"
            />
            {keyError && <span className="text-red-400/80 text-xs">{keyError}</span>}
            <button
              onClick={handleSaveKey}
              disabled={savingKey}
              className="ark-action-btn text-xs px-5 py-1.5 disabled:opacity-40"
            >
              {savingKey ? 'GUARDANDO...' : 'GUARDAR Y CARGAR MODS'}
            </button>
          </div>
          <p className="text-ark-cyan/35 text-[10px]">
            La key se guarda localmente en %APPDATA%\ARK ASA Config Manager\ y solo se usa para llamar a la API de CurseForge.
          </p>
        </div>

        <div className="ark-panel rounded-lg p-4 mt-4">
          <h3 className="text-ark-cyan/80 text-xs font-bold tracking-widest uppercase mb-2">O agrega un mod por ID de CurseForge</h3>
          <div className="flex gap-2 items-center">
            <input
              type="text"
              value={customId}
              onChange={e => { setCustomId(e.target.value); setCustomError('') }}
              onKeyDown={e => e.key === 'Enter' && addCustom()}
              placeholder="ej: 928988"
              className="w-44 bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/70 placeholder-ark-cyan/30"
            />
            <button onClick={addCustom} disabled={resolvingId} className="ark-action-btn text-xs px-4 py-1.5 disabled:opacity-40">
              {resolvingId ? '...' : '+ ADD'}
            </button>
            {customError && <span className="text-red-400/80 text-xs">{customError}</span>}
          </div>
        </div>
      </div>
    )
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Main mod browser
  // ─────────────────────────────────────────────────────────────────────────
  return (
    <div className="px-8 py-6 pb-24 space-y-4">
      {/* Top bar */}
      <div className="ark-panel rounded-lg p-4 flex flex-wrap gap-4 items-end">
        <div>
          <h3 className="text-ark-cyan/80 text-xs font-bold tracking-widest uppercase mb-2">Agregar por ID de CurseForge</h3>
          <div className="flex gap-2 items-center">
            <input
              type="text"
              value={customId}
              onChange={e => { setCustomId(e.target.value); setCustomError('') }}
              onKeyDown={e => e.key === 'Enter' && addCustom()}
              placeholder="ej: 928988"
              className="w-44 bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/70 placeholder-ark-cyan/30"
            />
            <button onClick={addCustom} disabled={resolvingId} className="ark-action-btn text-xs px-4 py-1.5 disabled:opacity-40">
              {resolvingId ? 'BUSCANDO...' : '+ ADD'}
            </button>
            {customError && <span className="text-red-400/80 text-xs">{customError}</span>}
          </div>
        </div>

        <div className="ml-auto flex gap-2 items-center">
          <button
            onClick={() => { setSearch(''); fetchMods(0, true) }}
            disabled={loading}
            className="ark-action-btn text-xs px-3 py-1.5 disabled:opacity-40"
            title="Forzar recarga desde CurseForge (limpia caché)"
          >
            ↻ REFRESH
          </button>
          <button
            onClick={() => { storeApiKey(''); setApiKeyState(''); setMods([]); setApiKeyInput('') }}
            className="ark-action-btn text-xs px-3 py-1.5"
            style={{ color: 'rgba(239,68,68,0.65)', outlineColor: 'rgba(239,68,68,0.3)' }}
            title="Cambiar API key"
          >
            ⚙ API KEY
          </button>
          <button
            onClick={() => invoke('open_external_url', { url: 'https://www.curseforge.com/ark-survival-ascended/mods?sortField=2&sortOrder=desc' })}
            className="ark-action-btn text-xs px-4 py-1.5"
          >
            VER EN CURSEFORGE ↗
          </button>
        </div>
      </div>

      {/* Search + category filter */}
      <div className="flex flex-col gap-2">
        <input
          type="text"
          value={search}
          onChange={e => setSearch(e.target.value)}
          placeholder="Buscar mods por nombre, descripción o ID... (busca en CurseForge)"
          className="w-full bg-transparent border border-ark-cyan/25 text-ark-cyan/80 text-sm px-3 py-2 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/25"
        />
        {categories.length > 1 && (
          <div className="flex gap-1 flex-wrap">
            {categories.map(cat => (
              <button
                key={cat}
                onClick={() => setSelectedCategory(cat)}
                className={`text-[10px] px-2.5 py-1 rounded transition border ${
                  selectedCategory === cat
                    ? 'border-ark-cyan/60 bg-ark-cyan/15 text-ark-cyan font-bold'
                    : 'border-ark-cyan/20 text-ark-cyan/45 hover:border-ark-cyan/40 hover:text-ark-cyan/70'
                }`}
              >
                {cat}
              </button>
            ))}
          </div>
        )}
      </div>

      {fetchError && (
        <div className="text-red-400/80 text-xs bg-red-400/10 border border-red-400/30 px-4 py-2 rounded">
          {fetchError}
        </div>
      )}

      {/* Mod list */}
      <div className="ark-panel rounded-lg overflow-hidden">
        <div className="bg-ark-secondary/40 px-4 py-2 border-b border-ark-cyan/20 flex justify-between items-center">
          <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
            {loading && mods.length === 0
              ? 'Cargando mods desde CurseForge...'
              : `${filtered.length} mods${search ? ` para "${search}"` : ' de ARK Survival Ascended'}`}
          </span>
          <span className="text-ark-cyan/30 text-xs flex items-center gap-2">
            {fromCache && <span className="text-ark-cyan/25 italic text-[10px]">caché</span>}
            {totalCount > 0 && !search && <span>{totalCount.toLocaleString()} total en CurseForge</span>}
            <span>·</span>
            <span>{activeMods.length} activos</span>
          </span>
        </div>

        {loading && mods.length === 0 && (
          <div className="p-12 text-center">
            <div className="text-ark-cyan/50 text-sm animate-pulse">Obteniendo mods de CurseForge...</div>
          </div>
        )}

        {(!loading || mods.length > 0) && (
          <div className="ark-scroll overflow-y-auto" style={{ maxHeight: 'calc(100vh - 380px)' }}>
            {filtered.length === 0 && !loading ? (
              <div className="p-8 text-ark-cyan/40 text-sm text-center">
                {mods.length === 0 ? 'No se cargaron mods.' : 'Ningún mod coincide con tu búsqueda.'}
              </div>
            ) : (
              filtered.map(mod => {
                const isActive = activeMods.includes(mod.id)
                return (
                  <div
                    key={mod.id}
                    className={`flex items-center gap-4 px-4 py-3 border-b border-ark-cyan/10 hover:bg-ark-secondary/15 transition ${isActive ? 'bg-ark-cyan/5' : ''}`}
                  >
                    {mod.logo_url ? (
                      <img src={mod.logo_url} alt="" className="w-10 h-10 rounded object-cover flex-shrink-0 opacity-80" loading="lazy" />
                    ) : (
                      <div className="w-10 h-10 rounded flex-shrink-0 bg-ark-cyan/10 flex items-center justify-center text-ark-cyan/30 text-[9px] font-bold">MOD</div>
                    )}

                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-0.5 flex-wrap">
                        <span className="text-ark-cyan/90 text-sm font-semibold">{mod.name}</span>
                        {mod.client_only && (
                          <span className="text-red-400/90 text-[10px] border border-red-500/50 bg-red-500/10 px-1.5 py-0.5 rounded font-bold" title="No funciona en servidores cross-platform">⚠ SOLO PC</span>
                        )}
                        {mod.categories.slice(0, 2).map(cat => (
                          <span key={cat} className="text-ark-cyan/30 text-[10px] border border-ark-cyan/20 px-1.5 py-0.5 rounded">{cat}</span>
                        ))}
                        {isActive && (
                          <span className="text-ark-cyan text-[10px] border border-ark-cyan/50 bg-ark-cyan/10 px-1.5 py-0.5 rounded font-bold">✓ ACTIVO</span>
                        )}
                      </div>
                      <p className="text-ark-cyan/50 text-xs line-clamp-1">{mod.summary}</p>
                      <div className="flex gap-3 mt-0.5">
                        <span className="text-ark-cyan/25 text-[10px] font-mono">ID: {mod.id}</span>
                        {mod.download_count > 0 && (
                          <span className="text-ark-cyan/25 text-[10px]">↓ {formatDownloads(mod.download_count)}</span>
                        )}
                      </div>
                    </div>

                    <div className="flex gap-1.5 flex-shrink-0 items-center">
                      <button
                        onClick={() => invoke('open_external_url', { url: `https://www.curseforge.com/ark-survival-ascended/mods/${mod.slug || mod.id}` })}
                        className="text-ark-cyan/30 hover:text-ark-cyan/70 text-xs transition px-1.5 py-1"
                        title="Ver en CurseForge"
                      >↗</button>
                      {isActive ? (
                        <button
                          onClick={() => removeFromActive(mod.id)}
                          className="ark-action-btn text-[10px] px-3 py-1"
                          style={{ color: 'rgba(239,68,68,0.75)', outlineColor: 'rgba(239,68,68,0.4)' }}
                        >QUITAR</button>
                      ) : (
                        <button onClick={() => addToActive(mod)} className="ark-action-btn text-[10px] px-3 py-1">+ ADD</button>
                      )}
                    </div>
                  </div>
                )
              })
            )}

            {hasMore && (
              <div className="p-4 text-center">
                <button
                  onClick={() => fetchMods(currentPage + 1)}
                  disabled={loading}
                  className="ark-action-btn text-xs px-6 py-2 disabled:opacity-40"
                >
                  {loading ? 'Cargando...' : `CARGAR MÁS (${mods.length} / ${totalCount.toLocaleString()})`}
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
