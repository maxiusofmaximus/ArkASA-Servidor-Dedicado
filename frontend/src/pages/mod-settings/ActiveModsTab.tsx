import React, { useState, useEffect } from 'react'
import { useConfigStore } from '../../stores/configStore'
import { useModsStore } from '../../stores/modsStore'
import { useBackupStore } from '../../stores/backupStore'
import { invoke } from '../../services/tauri'
import { useI18n } from '../../i18n/useI18n'

type RemovePending = { modId: string; modName: string; index: number }

export default function ActiveModsTab() {
  const { config, setConfig } = useConfigStore()
  const { modCache, setModInfo, getModInfo } = useModsStore()
  const store = useBackupStore()
  const { tk } = useI18n()
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [checkingPcOnly, setCheckingPcOnly] = useState(false)
  const [removePending, setRemovePending] = useState<RemovePending | null>(null)
  const [isBackingUp, setIsBackingUp] = useState(false)
  const [backupMsg, setBackupMsg] = useState<string | null>(null)
  const mods = config?.mods?.active_mods ?? []

  // Fetch names for any mod IDs not yet in the cache
  useEffect(() => {
    const unknown = mods.filter(id => !modCache[id])
    if (unknown.length === 0) return
    unknown.forEach(async (id) => {
      try {
        const mod = await invoke('get_curseforge_mod_by_id', { modId: id }) as any
        if (mod?.name) {
          setModInfo(id, {
            name: mod.name,
            summary: mod.summary ?? '',
            logoUrl: mod.logo_url ?? null,
            slug: mod.slug ?? '',
            downloadCount: mod.download_count ?? 0,
            categories: mod.categories ?? [],
            clientOnly: mod.client_only ?? false,
          })
        }
      } catch {
        // silently ignore — ID stays as display fallback
      }
    })
  }, [mods.join(',')]) // eslint-disable-line react-hooks/exhaustive-deps

  // Check PC-only status
  useEffect(() => {
    if (mods.length === 0) return
    const toCheck = mods.filter(id => modCache[id]?.clientOnly !== false)
    if (toCheck.length === 0) return
    setCheckingPcOnly(true)
    invoke('check_client_only_mods', { modIds: toCheck })
      .then((clientOnlyIds: unknown) => {
        const pcOnlySet = new Set(clientOnlyIds as string[])
        toCheck.forEach(id => {
          const cached = modCache[id]
          if (cached) setModInfo(id, { ...cached, clientOnly: pcOnlySet.has(id) })
        })
      })
      .catch(() => {})
      .finally(() => setCheckingPcOnly(false))
  }, [mods.join(',')]) // eslint-disable-line react-hooks/exhaustive-deps

  const updateMods = (next: string[]) => {
    if (!config) return
    setConfig({ ...config, mods: { ...config.mods, active_mods: next } })
  }

  // Duplicate name detection
  const nameCounts: Record<string, number> = {}
  for (const id of mods) {
    const name = modCache[id]?.name?.toLowerCase()
    if (name) nameCounts[name] = (nameCounts[name] ?? 0) + 1
  }
  const duplicateNames = new Set(Object.keys(nameCounts).filter(n => nameCounts[n] > 1))
  const hasDuplicates = duplicateNames.size > 0

  const pcOnlyIds = mods.filter(id => modCache[id]?.clientOnly === true)
  const hasPcOnly = pcOnlyIds.length > 0

  const removeDuplicates = () => {
    const seenNames = new Set<string>()
    const seenIds = new Set<string>()
    const deduped = mods.filter(id => {
      if (seenIds.has(id)) return false
      seenIds.add(id)
      const name = modCache[id]?.name?.toLowerCase()
      if (name) {
        if (seenNames.has(name)) return false
        seenNames.add(name)
      }
      return true
    })
    updateMods(deduped)
    setSelectedIdx(null)
  }

  const removeAllPcOnly = () => {
    updateMods(mods.filter(id => !pcOnlyIds.includes(id)))
    setSelectedIdx(null)
  }

  const removeMod = (i: number) => {
    updateMods(mods.filter((_, idx) => idx !== i))
    setSelectedIdx(null)
  }

  // Request removal — shows warning if the mod has been used in server launches
  const requestRemoveMod = (i: number) => {
    const modId = mods[i]
    const usage = store.modUsageHistory[modId]
    if (usage && usage.serverLaunches > 0) {
      setRemovePending({ modId, modName: modCache[modId]?.name ?? modId, index: i })
      setBackupMsg(null)
    } else {
      removeMod(i)
    }
  }

  const isBackupConfigured = () => {
    switch (store.provider) {
      case 's3':           return !!(store.s3Endpoint && store.s3Bucket && store.s3AccessKey && store.s3SecretKey)
      case 'gdrive':       return !!store.gdriveAccessToken
      case 'onedrive':     return !!store.onedriveAccessToken
      case 'icloud':       return !!store.icloudPath
      case 'local_folder': return !!store.localFolderPath
      default:             return false
    }
  }

  const handleBackupAndRemove = async () => {
    if (!removePending || !config) return
    setIsBackingUp(true)
    setBackupMsg(null)
    try {
      const token = store.provider === 'gdrive' ? store.gdriveAccessToken
        : store.provider === 'onedrive' ? store.onedriveAccessToken
        : undefined
      const metadataJson = JSON.stringify({
        server_name: config.identification.session_name,
        mod_ids: config.mods.active_mods,
        scope: store.backupScope,
        backed_up_at: new Date().toISOString(),
        note: `Backup before removing mod ${removePending.modId}`,
      })
      let configToml: string | null = null
      try { configToml = await invoke<string>('config_to_toml', { config }) } catch { /* non-fatal */ }

      const filename = await invoke<string>('backup_saves', {
        serverDir: config.paths.server_dir,
        map: config.cluster_maps?.[0] || 'TheIsland_WP',
        scope: store.backupScope,
        provider: store.provider,
        s3Endpoint: store.s3Endpoint || null,
        s3Bucket: store.s3Bucket || null,
        s3AccessKey: store.s3AccessKey || null,
        s3SecretKey: store.s3SecretKey || null,
        s3Region: store.s3Region || null,
        accessToken: token || null,
        icloudPath: store.icloudPath || null,
        localFolderPath: store.localFolderPath || null,
        metadataJson,
        configToml,
      })
      store.addBackupEntry({ filename, size_bytes: 0, created_at: new Date().toISOString(), provider: store.provider })
      removeMod(removePending.index)
      setRemovePending(null)
    } catch (e) {
      setBackupMsg(`❌ Error al hacer backup: ${String(e)}`)
    } finally {
      setIsBackingUp(false)
    }
  }

  const moveUp = (i: number) => {
    if (i === 0) return
    const next = [...mods];
    [next[i - 1], next[i]] = [next[i], next[i - 1]]
    updateMods(next)
    setSelectedIdx(i - 1)
  }

  const moveDown = (i: number) => {
    if (i === mods.length - 1) return
    const next = [...mods];
    [next[i], next[i + 1]] = [next[i + 1], next[i]]
    updateMods(next)
    setSelectedIdx(i + 1)
  }

  // Filter by search query (name or ID)
  const q = searchQuery.trim().toLowerCase()
  const filteredIndices = mods
    .map((id, i) => ({ id, i }))
    .filter(({ id }) => {
      if (!q) return true
      const name = modCache[id]?.name?.toLowerCase() ?? ''
      return name.includes(q) || id.includes(q)
    })

  const selectedId = selectedIdx !== null ? mods[selectedIdx] : null
  const selectedInfo = selectedId ? getModInfo(selectedId) : null

  // ── Mod remove warning modal ──────────────────────────────────────────────────
  const usage = removePending ? store.modUsageHistory[removePending.modId] : null
  const canBackup = store.provider !== 'none' && isBackupConfigured()

  const RemoveWarningModal = removePending ? (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: 'rgba(0,0,0,0.8)', backdropFilter: 'blur(4px)' }}
    >
      <div
        className="ark-panel rounded-xl w-full max-w-md mx-4 flex flex-col"
        style={{ border: '1px solid rgba(239,68,68,0.4)', boxShadow: '0 0 40px rgba(239,68,68,0.15)' }}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-5 pt-5 pb-4 border-b border-red-500/25">
          <span className="text-red-400 text-xl">⚠</span>
          <span className="text-red-300/90 font-bold tracking-widest text-sm uppercase">{tk('mod_remove_warning_title', 'Warning: Server Data')}</span>
        </div>

        {/* Body */}
        <div className="px-5 py-4 space-y-3">
          <p className="text-white/80 text-sm">
            {tk('the_mod', 'The mod')} <span className="text-ark-cyan font-semibold">"{removePending.modName}"</span>{' '}
            <span className="text-white/50 font-mono text-xs">#{removePending.modId}</span> {tk('mod_started_with_server', 'has been started with the server')}{' '}
            <span className="text-red-300 font-bold">{usage?.serverLaunches ?? 0} {(usage?.serverLaunches ?? 0) === 1 ? tk('times_once', 'time') : tk('times_plural', 'times')}</span>.
          </p>

          {usage && (
            <div className="rounded-md px-3 py-2 space-y-0.5" style={{ background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)' }}>
              <p className="text-ark-cyan/50 text-[10px]">
                {tk('first_use', 'First use')}: <span className="text-ark-cyan/70">{new Date(usage.firstSeen).toLocaleDateString()}</span>
                {' · '}{tk('last_active_short', 'Last')}: <span className="text-ark-cyan/70">{new Date(usage.lastActive).toLocaleDateString()}</span>
              </p>
            </div>
          )}

          <div className="rounded-md px-3 py-2.5 space-y-1" style={{ background: 'rgba(239,68,68,0.08)', border: '1px solid rgba(239,68,68,0.3)' }}>
            <p className="text-red-300/90 text-xs font-bold">{tk('remove_causes_loss', 'Removing it can cause permanent loss of:')}</p>
            <ul className="text-red-300/65 text-[11px] space-y-0.5 ml-3">
              <li>• {tk('loss_structures', 'Structures placed with this mod (tables, machines, etc.)')}</li>
              <li>• {tk('loss_items', 'Items and crafts from the mod in inventories and chests')}</li>
              <li>• {tk('loss_save_data', 'Saved data associated with the mod in the .ark file')}</li>
            </ul>
          </div>

          <p className="text-ark-cyan/40 text-[10px]">
            {tk('mod_restore_hint_text', 'To restore it later: Options → Backup → Restore a previous backup that had the mod active.')}
          </p>

          {backupMsg && (
            <p className="text-red-400/80 text-xs">{backupMsg}</p>
          )}
        </div>

        {/* Buttons */}
        <div className="px-5 pb-5 flex gap-2 flex-wrap">
          <button
            onClick={() => { setRemovePending(null); setBackupMsg(null) }}
            className="ark-action-btn px-4 py-2 text-xs tracking-widest flex-1"
          >
            {tk('cancel', 'CANCEL')}
          </button>
          <button
            onClick={() => { removeMod(removePending.index); setRemovePending(null) }}
            className="px-4 py-2 text-xs font-bold tracking-widest rounded transition-all flex-1"
            style={{ background: 'rgba(239,68,68,0.15)', color: 'rgba(239,68,68,0.85)', border: '1px solid rgba(239,68,68,0.4)' }}
          >
            {tk('remove_anyway', 'REMOVE ANYWAY')}
          </button>
          {canBackup && (
            <button
              onClick={handleBackupAndRemove}
              disabled={isBackingUp}
              className="px-4 py-2 text-xs font-bold tracking-widest rounded transition-all w-full"
              style={{ background: 'rgba(0,200,255,0.15)', color: 'rgba(0,200,255,0.9)', border: '1px solid rgba(0,200,255,0.4)', opacity: isBackingUp ? 0.6 : 1 }}
            >
              {isBackingUp ? tk('creating_backup', '⏳ CREATING BACKUP...') : tk('backup_and_remove', '💾 CREATE BACKUP AND REMOVE')}
            </button>
          )}
          {!canBackup && (
            <p className="text-ark-cyan/30 text-[10px] w-full text-center">
              {tk('configure_backup_hint', 'Configure a provider in Options → Backup to create a backup before removing.')}
            </p>
          )}
        </div>
      </div>
    </div>
  ) : null

  return (
    <>
    {RemoveWarningModal}
    <div className="flex gap-4 px-8 py-6 pb-24">
      {/* Left: mod list */}
      <div className="flex-1 flex flex-col gap-3">

        {/* Search/filter input */}
        <div className="ark-panel rounded-lg p-3 flex gap-2 items-center">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={tk('search_active_mods', 'Search active mods by name or ID…')}
            className="flex-1 bg-transparent border border-ark-cyan/30 text-ark-cyan/90 text-sm px-3 py-1.5 rounded focus:outline-none focus:border-ark-cyan/70 placeholder-ark-cyan/30"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="text-ark-cyan/40 hover:text-ark-cyan/80 text-sm px-2 transition"
            >×</button>
          )}
        </div>

        {/* PC-only warning banner */}
        {hasPcOnly && (
          <div className="flex items-center gap-3 bg-red-500/10 border border-red-500/40 text-red-300/80 text-xs px-4 py-2.5 rounded-lg">
            <span className="text-red-400 text-sm">⚠</span>
            <div className="flex-1">
              <p className="font-bold text-red-300/90 mb-0.5">{tk('pc_only_detected', 'PC-only mods detected — they will crash the server')}</p>
              <p className="text-red-300/60 text-[11px]">
                {pcOnlyIds.map(id => modCache[id]?.name ?? id).join(', ')}
              </p>
            </div>
            <button
              onClick={removeAllPcOnly}
              className="ark-action-btn text-[10px] px-3 py-1 flex-shrink-0"
              style={{ color: 'rgba(248,113,113,0.85)', outlineColor: 'rgba(248,113,113,0.4)' }}
            >
              {pcOnlyIds.length > 1 ? tk('remove_all_btn', 'REMOVE ALL') : tk('remove_btn', 'REMOVE')}
            </button>
          </div>
        )}

        {/* Duplicate warning banner */}
        {hasDuplicates && (
          <div className="flex items-center gap-3 bg-yellow-400/10 border border-yellow-400/40 text-yellow-300/80 text-xs px-4 py-2 rounded-lg">
            <span className="flex-1">
              {tk('duplicate_mods_warning', '⚠ Duplicate mods:')} {[...duplicateNames].map(n => {
                const id = mods.find(i => modCache[i]?.name?.toLowerCase() === n)
                return id ? modCache[id]?.name : null
              }).filter(Boolean).join(', ')}
            </span>
            <button
              onClick={removeDuplicates}
              className="ark-action-btn text-[10px] px-3 py-1 flex-shrink-0"
              style={{ color: 'rgba(251,191,36,0.8)', outlineColor: 'rgba(251,191,36,0.4)' }}
            >
              {tk('remove_duplicates', 'REMOVE DUPLICATES')}
            </button>
          </div>
        )}

        {checkingPcOnly && (
          <p className="text-ark-cyan/30 text-[10px] px-1 -mt-1 animate-pulse">{tk('checking_mod_compat', 'Checking mod compatibility...')}</p>
        )}

        {/* Mod list */}
        <div className="ark-panel rounded-lg overflow-hidden flex-1">
          <div className="bg-ark-secondary/40 px-4 py-2 border-b border-ark-cyan/20 flex items-center justify-between">
            <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
              {tk('active_mods_title', 'Active Mods')} ({q ? `${filteredIndices.length}/` : ''}{mods.length})
            </span>
            <span className="text-ark-cyan/40 text-xs">{tk('load_order_hint', 'Load order ↑ = higher priority')}</span>
          </div>
          <div className="ark-scroll overflow-y-auto" style={{ maxHeight: 'calc(100vh - 320px)' }}>
            {mods.length === 0 ? (
              <div className="p-8 text-ark-cyan/40 text-sm text-center">
                <div className="text-2xl mb-2 opacity-40">📦</div>
                {tk('no_active_mods', 'No active mods. Go to the mod search tab to add them.')}
              </div>
            ) : filteredIndices.length === 0 ? (
              <div className="p-8 text-ark-cyan/40 text-sm text-center">
                {tk('no_mods_match', 'No mods match "{{query}}".').replace('{{query}}', searchQuery)}
              </div>
            ) : (
              filteredIndices.map(({ id: modId, i }) => {
                const info = getModInfo(modId)
                const displayName = info?.name ?? modId
                const isResolved = !!info?.name
                const isDuplicate = info?.name ? duplicateNames.has(info.name.toLowerCase()) : false
                const isPcOnly = info?.clientOnly === true

                return (
                  <div
                    key={`${modId}-${i}`}
                    onClick={() => setSelectedIdx(i === selectedIdx ? null : i)}
                    className={`flex items-center gap-2 px-4 py-2.5 border-b border-ark-cyan/10 cursor-pointer transition hover:bg-ark-secondary/20
                      ${selectedIdx === i ? 'bg-ark-secondary/35 border-l-2 border-l-ark-cyan/60' : ''}
                      ${isPcOnly ? 'bg-red-500/5' : isDuplicate ? 'bg-yellow-400/5' : ''}`}
                  >
                    {/* Priority controls */}
                    <div className="flex flex-col gap-0.5">
                      <button
                        onClick={(e) => { e.stopPropagation(); moveUp(i) }}
                        disabled={i === 0}
                        className="text-ark-cyan/50 hover:text-ark-cyan disabled:opacity-20 text-[10px] leading-none px-0.5 transition"
                      >▲</button>
                      <button
                        onClick={(e) => { e.stopPropagation(); moveDown(i) }}
                        disabled={i === mods.length - 1}
                        className="text-ark-cyan/50 hover:text-ark-cyan disabled:opacity-20 text-[10px] leading-none px-0.5 transition"
                      >▼</button>
                    </div>

                    <span className="text-ark-cyan/30 text-xs w-5 text-right flex-shrink-0">{i + 1}</span>

                    {isPcOnly ? (
                      <span className="w-5 h-5 flex items-center justify-center border border-red-500/70 text-red-400 text-[9px] font-bold flex-shrink-0" title="Solo PC">PC</span>
                    ) : isDuplicate ? (
                      <span className="w-5 h-5 flex items-center justify-center border border-yellow-400/60 text-yellow-300 text-[10px] font-bold flex-shrink-0" title="Duplicado">!</span>
                    ) : (
                      <span className="w-5 h-5 flex items-center justify-center border border-ark-cyan/50 text-ark-cyan text-[10px] font-bold flex-shrink-0">A</span>
                    )}

                    {info?.logoUrl ? (
                      <img src={info.logoUrl} alt="" className="w-6 h-6 rounded object-cover flex-shrink-0 opacity-70" />
                    ) : null}

                    <div className="flex-1 min-w-0">
                      <span className={`text-sm tracking-wide ${
                        isPcOnly ? 'text-red-300/80' :
                        isDuplicate ? 'text-yellow-300/80' :
                        isResolved ? 'text-ark-cyan/90 font-medium' : 'text-ark-cyan/60 font-mono'
                      }`}>
                        {displayName}
                      </span>
                      {isResolved && (
                        <span className="text-ark-cyan/30 text-[10px] font-mono ml-2">#{modId}</span>
                      )}
                      {isPcOnly && (
                        <span className="text-red-400/60 text-[10px] ml-2 italic">{tk('pc_only_tag', 'PC only')}</span>
                      )}
                    </div>

                    <button
                      onClick={(e) => { e.stopPropagation(); invoke('open_external_url', { url: `https://www.curseforge.com/ark-survival-ascended/mods/${info?.slug || modId}` }) }}
                      className="text-ark-cyan/30 hover:text-ark-cyan/70 text-xs transition flex-shrink-0"
                      title="Ver en CurseForge"
                    >↗</button>

                    <button
                      onClick={(e) => { e.stopPropagation(); requestRemoveMod(i) }}
                      className="text-red-400/50 hover:text-red-400 text-sm font-bold transition flex-shrink-0 w-5 text-center"
                      title="Quitar mod"
                    >×</button>
                  </div>
                )
              })
            )}
          </div>
        </div>
      </div>

      {/* Right: preview panel */}
      <div className="w-72 ark-panel rounded-lg flex flex-col overflow-hidden" style={{ minHeight: 'calc(100vh - 260px)', maxHeight: 'calc(100vh - 260px)' }}>
        <div className="bg-ark-secondary/40 px-4 py-2 border-b border-ark-cyan/20">
          <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">{tk('mod_details', 'Mod Details')}</span>
        </div>
        <div className="flex-1 flex items-center justify-center p-6 overflow-y-auto">
          {selectedId ? (
            <div className="text-center space-y-3 w-full">
              {selectedInfo?.logoUrl ? (
                <img src={selectedInfo.logoUrl} alt="" className="w-20 h-20 rounded object-cover mx-auto opacity-80" />
              ) : (
                <div className="text-ark-cyan/30 text-4xl">📦</div>
              )}
              <div>
                <p className="text-ark-cyan text-sm font-semibold">
                  {selectedInfo?.name ?? selectedId}
                </p>
                <p className="text-ark-cyan/40 text-xs font-mono mt-0.5">ID: {selectedId}</p>
                {selectedInfo?.clientOnly && (
                  <p className="text-red-400/80 text-[10px] mt-1 border border-red-500/40 bg-red-500/10 px-2 py-0.5 rounded inline-block">
                    {tk('pc_only_crash_warning', '⚠ PC-Only — will crash the server')}
                  </p>
                )}
              </div>
              {selectedInfo?.summary && (
                <p className="text-ark-cyan/50 text-xs leading-relaxed text-left">{selectedInfo.summary}</p>
              )}
              {selectedInfo?.categories && selectedInfo.categories.length > 0 && (
                <div className="flex flex-wrap gap-1 justify-center">
                  {selectedInfo.categories.map(cat => (
                    <span key={cat} className="text-[10px] border border-ark-cyan/20 text-ark-cyan/40 px-1.5 py-0.5 rounded">{cat}</span>
                  ))}
                </div>
              )}
              {selectedInfo?.downloadCount ? (
                <p className="text-ark-cyan/30 text-xs">↓ {selectedInfo.downloadCount.toLocaleString()} {tk('downloads', 'downloads')}</p>
              ) : null}
              <div className="text-ark-cyan/40 text-xs">Orden: #{(selectedIdx ?? 0) + 1}</div>
              <button
                onClick={() => invoke('open_external_url', { url: `https://www.curseforge.com/ark-survival-ascended/mods/${selectedInfo?.slug || selectedId}` })}
                className="inline-block ark-action-btn text-xs px-3 py-1 mt-2"
              >
                VER EN CURSEFORGE ↗
              </button>
            </div>
          ) : (
            <span className="text-ark-cyan/30 text-xs font-bold tracking-widest text-center whitespace-pre-line">
              {tk('select_mod_hint', 'Select a mod\nto view details')}
            </span>
          )}
        </div>
      </div>
    </div>
    </>
  )
}
