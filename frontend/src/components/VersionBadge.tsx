/**
 * VersionBadge
 *
 * One-line pill that shows the ARK server's local buildid next to a
 * green/red dot so the operator instantly knows whether the install
 * is in sync with Steam.
 *
 *  🟢 Up to date     Version 8822334
 *  🔴 Outdated       Version 8811722 → latest 8822334
 *  ⚪ Unknown        No SteamCMD output / manifest missing
 */
import { useState } from 'react'
import { versionStatus } from '../utils/versionStatus'
import type { ServerVersionInfo } from '../types'

interface Props {
  info:     ServerVersionInfo | null
  loading:  boolean
  updating: boolean
  onRefresh: () => void
  onUpdate:  () => Promise<void>
}

export default function VersionBadge({ info, loading, updating, onRefresh, onUpdate }: Props) {
  const [confirming, setConfirming] = useState(false)
  const status  = versionStatus(info)
  const local  = info?.local_buildid  ?? null
  const latest = info?.latest_buildid ?? null

  if (loading && !info) {
    return (
      <span className="text-[10px] italic text-ark-cyan/40">checking buildid…</span>
    )
  }

  const { color, label, dot } = pick(status)

  return (
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onRefresh}
        title="Re-check buildid against Steam"
        className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono"
        style={{ color, borderColor: color + '40', background: color + '15' }}
      >
        <span>{dot}</span>
        <span>{label}</span>
        {local ? <span className="opacity-60">· v{local}</span> : null}
        {latest && status !== 'unknown' ? (
          <span className="opacity-40">→ {latest}</span>
        ) : null}
      </button>

      {status === 'outdated' || status === 'unknown' ? (
        confirming ? (
          <div className="flex items-center gap-1">
            <button
              type="button"
              disabled={updating}
              onClick={async () => {
                try { await onUpdate() } finally { setConfirming(false) }
              }}
              className="text-[10px] px-2 py-0.5 rounded bg-ark-cyan/20 text-ark-cyan hover:bg-ark-cyan/30 disabled:opacity-50"
            >
              {updating ? 'updating…' : 'yes, update'}
            </button>
            <button
              type="button"
              onClick={() => setConfirming(false)}
              className="text-[10px] px-2 py-0.5 rounded text-ark-cyan/50 hover:text-ark-cyan"
            >
              cancel
            </button>
          </div>
        ) : (
          <button
            type="button"
            disabled={updating}
            onClick={() => setConfirming(true)}
            className="text-[10px] px-2 py-0.5 rounded border border-ark-cyan/30 text-ark-cyan/80 hover:text-ark-cyan hover:border-ark-cyan disabled:opacity-50"
          >
            ⤓ update now
          </button>
        )
      ) : null}
    </div>
  )
}

function pick(status: ReturnType<typeof versionStatus>): { color: string; label: string; dot: string } {
  switch (status) {
    case 'current':  return { color: 'rgb(74,222,128)',  label: 'Up to date',  dot: '🟢' }
    case 'outdated': return { color: 'rgb(248,113,113)', label: 'Outdated',    dot: '🔴' }
    case 'unknown':  return { color: 'rgb(148,163,184)', label: 'No manifest', dot: '⚪' }
  }
}
