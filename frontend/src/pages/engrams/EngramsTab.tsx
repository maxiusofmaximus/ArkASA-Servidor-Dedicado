import React, { useState, useMemo } from 'react'
import { useConfigStore, type ConfigStore } from '../../stores/configStore'
import { useShallow } from 'zustand/react/shallow'
import type { AdvancedConfig } from '../../types'
import { ENGRAMS_DB } from './data/engrams-db'

const CATEGORIES = ['All', ...Array.from(new Set(ENGRAMS_DB.map(e => e.category)))]

export default function EngramsTab() {
  const { config, setConfig } = useConfigStore(useShallow((s: ConfigStore) => ({ config: s.config, setConfig: s.setConfig })))
  const [search, setSearch] = useState('')
  const [category, setCategory] = useState('All')
  const [customId, setCustomId] = useState('')
  const [customError, setCustomError] = useState('')

  const onlySpecific = config?.advanced?.only_allow_specific_engrams ?? false
  const autoUnlock = config?.advanced?.auto_unlock_engrams ?? []

  const updateAdvanced = (patch: Partial<AdvancedConfig>) => {
    if (!config) return
    setConfig({ ...config, advanced: { ...config.advanced, ...patch } })
  }

  const toggleOnlySpecific = () => {
    updateAdvanced({ only_allow_specific_engrams: !onlySpecific })
  }

  const addEngram = (id: number) => {
    if (autoUnlock.includes(id)) return
    updateAdvanced({ auto_unlock_engrams: [...autoUnlock, id] })
  }

  const removeEngram = (id: number) => {
    updateAdvanced({ auto_unlock_engrams: autoUnlock.filter(e => e !== id) })
  }

  const addCustom = () => {
    const id = parseInt(customId.trim(), 10)
    if (isNaN(id) || id <= 0) { setCustomError('Enter a valid engram ID'); return }
    if (autoUnlock.includes(id)) { setCustomError('Already in list'); return }
    setCustomError('')
    addEngram(id)
    setCustomId('')
  }

  const filteredDB = useMemo(() => {
    return ENGRAMS_DB.filter(e => {
      const matchCat = category === 'All' || e.category === category
      const matchSearch = !search || e.name.toLowerCase().includes(search.toLowerCase()) || String(e.id).includes(search)
      return matchCat && matchSearch
    })
  }, [search, category])

  // Enrich auto_unlock list with DB data where possible
  const enrichedAutoUnlock = autoUnlock.map(id => {
    const db = ENGRAMS_DB.find(e => e.id === id)
    return { id, name: db?.name, level: db?.level_required, category: db?.category }
  })

  return (
    <div className="flex gap-4 px-8 py-6 pb-24">
      {/* Left: DB browser */}
      <div className="flex-1 flex flex-col gap-3 min-w-0">
        {/* Only allow specific toggle */}
        <div className="ark-panel rounded-lg p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-ark-cyan/90 text-sm font-bold tracking-wide">Only Allow Specific Engrams</h3>
              <p className="text-ark-cyan/50 text-xs mt-0.5">When enabled, players can only unlock engrams in the auto-unlock list below</p>
            </div>
            <button
              onClick={toggleOnlySpecific}
              className={`relative w-12 h-6 rounded-full transition-all duration-300 border flex-shrink-0 ${
                onlySpecific
                  ? 'bg-ark-cyan/20 border-ark-cyan/60'
                  : 'bg-ark-secondary/50 border-ark-cyan/20'
              }`}
              title={onlySpecific ? 'Disable' : 'Enable'}
            >
              <span className={`absolute top-0.5 w-5 h-5 rounded-full transition-all duration-300 ${
                onlySpecific
                  ? 'left-6 bg-ark-cyan shadow-[0_0_8px_rgba(0,212,255,0.8)]'
                  : 'left-0.5 bg-ark-cyan/30'
              }`} />
            </button>
          </div>
        </div>

        {/* Search + filter */}
        <div className="flex gap-2 flex-wrap items-center">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search engrams..."
            className="flex-1 min-w-0 bg-transparent border border-ark-cyan/25 text-ark-cyan/80 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/25"
          />
          <div className="flex gap-1 flex-wrap">
            {CATEGORIES.map(cat => (
              <button
                key={cat}
                onClick={() => setCategory(cat)}
                className={`text-[10px] px-2 py-1 rounded transition border ${
                  category === cat
                    ? 'border-ark-cyan/60 bg-ark-cyan/15 text-ark-cyan font-bold'
                    : 'border-ark-cyan/20 text-ark-cyan/45 hover:border-ark-cyan/40 hover:text-ark-cyan/70'
                }`}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>

        {/* Engram DB table */}
        <div className="ark-panel rounded-lg overflow-hidden flex-1">
          <div className="bg-ark-secondary/40 px-4 py-2 border-b border-ark-cyan/20 flex justify-between">
            <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">Engram Database ({filteredDB.length})</span>
            <span className="text-ark-cyan/30 text-xs">Click + to auto-unlock on server start</span>
          </div>
          <div className="ark-scroll overflow-y-auto" style={{ maxHeight: 'calc(100vh - 400px)' }}>
            {filteredDB.length === 0 ? (
              <div className="p-6 text-ark-cyan/40 text-sm text-center">No engrams match your search.</div>
            ) : (
              filteredDB.map(engram => {
                const isUnlocked = autoUnlock.includes(engram.id)
                return (
                  <div key={engram.id} className={`flex items-center gap-3 px-4 py-2 border-b border-ark-cyan/10 hover:bg-ark-secondary/15 transition ${isUnlocked ? 'bg-ark-cyan/5' : ''}`}>
                    <span className="text-ark-cyan/25 text-xs w-8 text-right flex-shrink-0 font-mono">{engram.id}</span>
                    <div className="flex-1 min-w-0">
                      <span className="text-ark-cyan/80 text-xs font-medium">{engram.name}</span>
                    </div>
                    <span className="text-ark-cyan/30 text-[10px] flex-shrink-0">Lvl {engram.level_required}</span>
                    <span className="text-ark-cyan/25 text-[10px] flex-shrink-0 hidden xl:block">{engram.points_cost}pt</span>
                    <span className="text-ark-cyan/25 text-[10px] border border-ark-cyan/15 px-1 rounded flex-shrink-0 hidden xl:block">{engram.category}</span>
                    {isUnlocked ? (
                      <button
                        onClick={() => removeEngram(engram.id)}
                        className="text-ark-cyan text-xs font-bold w-6 text-center flex-shrink-0"
                        title="Remove from auto-unlock"
                      >✓</button>
                    ) : (
                      <button
                        onClick={() => addEngram(engram.id)}
                        className="text-ark-cyan/40 hover:text-ark-cyan text-sm font-bold w-6 text-center flex-shrink-0 transition"
                        title="Add to auto-unlock"
                      >+</button>
                    )}
                  </div>
                )
              })
            )}
          </div>
        </div>
      </div>

      {/* Right: auto-unlock list */}
      <div className="w-72 flex flex-col gap-3 flex-shrink-0">
        {/* Custom ID input */}
        <div className="ark-panel rounded-lg p-3">
          <h3 className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase mb-2">Add by ID</h3>
          <div className="flex gap-2">
            <input
              type="text"
              value={customId}
              onChange={(e) => { setCustomId(e.target.value); setCustomError('') }}
              onKeyDown={(e) => e.key === 'Enter' && addCustom()}
              placeholder="Engram ID"
              className="flex-1 bg-transparent border border-ark-cyan/25 text-ark-cyan/80 text-xs px-2 py-1.5 rounded focus:outline-none focus:border-ark-cyan/60 placeholder-ark-cyan/25"
            />
            <button onClick={addCustom} className="ark-action-btn text-[10px] px-3 py-1">ADD</button>
          </div>
          {customError && <p className="text-red-400/70 text-[10px] mt-1">{customError}</p>}
        </div>

        {/* Auto-unlock list */}
        <div className="ark-panel rounded-lg overflow-hidden flex-1">
          <div className="bg-ark-secondary/40 px-3 py-2 border-b border-ark-cyan/20">
            <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
              Auto-Unlock ({autoUnlock.length})
            </span>
          </div>
          <div className="ark-scroll overflow-y-auto" style={{ maxHeight: 'calc(100vh - 360px)' }}>
            {autoUnlock.length === 0 ? (
              <div className="p-6 text-ark-cyan/35 text-xs text-center">
                No engrams configured.<br/>
                Click + on any engram to auto-unlock it for all players.
              </div>
            ) : (
              enrichedAutoUnlock.map(({ id, name, level, category: _cat }) => (
                <div key={id} className="flex items-center gap-2 px-3 py-2 border-b border-ark-cyan/10 hover:bg-ark-secondary/15 transition">
                  <div className="flex-1 min-w-0">
                    {name ? (
                      <>
                        <p className="text-ark-cyan/80 text-xs font-medium truncate">{name}</p>
                        <p className="text-ark-cyan/35 text-[10px]">ID {id} · Lvl {level}</p>
                      </>
                    ) : (
                      <p className="text-ark-cyan/60 text-xs font-mono">ID: {id}</p>
                    )}
                  </div>
                  <button
                    onClick={() => removeEngram(id)}
                    className="text-red-400/50 hover:text-red-400 text-sm font-bold transition w-5 text-center flex-shrink-0"
                    title="Remove"
                  >×</button>
                </div>
              ))
            )}
          </div>
        </div>

        {onlySpecific && autoUnlock.length > 0 && (
          <div className="ark-panel rounded-lg p-3 border border-amber-400/30">
            <p className="text-amber-400/80 text-xs">
              <span className="font-bold">Restricted mode active</span> — players can only unlock these {autoUnlock.length} engram{autoUnlock.length !== 1 ? 's' : ''}.
            </p>
          </div>
        )}
      </div>
    </div>
  )
}
