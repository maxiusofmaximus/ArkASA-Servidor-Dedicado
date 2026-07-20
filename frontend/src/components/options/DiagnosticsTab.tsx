/**
 * DiagnosticsTab — Options modal ➜ Diagnostics
 *
 * Surface the Rust `diagnose_server_list` panel here.  Three checks:
 *   • culture_en     – `[Internationalization] Culture=en` in
 *                       GameUserSettings.ini (required for ASA EOS
 *                       session registration; absent ⇒ server silent).
 *   • eos_cert       – Amazon RSA 2048 M02 trust root in the Windows
 *                       store (missing/expired ⇒ TLS handshake to
 *                       Epic Online Services fails).
 *   • steam_validate – steamcmd build-id vs running client (mismatched
 *                       ⇒ server filtered out of unofficial list).
 *
 * Repair mode patches the INI (writes `[Internationalization] Culture=en`)
 * and reinserts the EOS trust root.  Steam validate is information-only
 * — the user must invoke it manually for now.
 */
import { useMemo } from 'react'
import type { ServerConfig } from '../../types'
import { useI18n } from '../../i18n/useI18n'
import { useDiagnostics, type DiagCheck } from '../../hooks/useDiagnostics'

const STATUS_COLOR: Record<string, string> = {
  ok:        'rgba(74,222,128,0.85)',   // green
  missing:   'rgba(250,204,21,0.85)',   // amber
  stale:     'rgba(250,204,21,0.85)',
  fixed:     'rgba(0,200,255,0.85)',    // cyan
  error:     'rgba(248,113,113,0.85)',  // red
  skipped:   'rgba(148,163,184,0.7)',
  unknown:   'rgba(148,163,184,0.7)',
}

const STATUS_ICON: Record<string, string> = {
  ok:        '✓',
  missing:   '!',
  stale:     '!',
  fixed:     '✦',
  error:     '✕',
  skipped:   '›',
  unknown:   '?',
}

function statusColor(s: string): string {
  return STATUS_COLOR[s] ?? 'rgba(148,163,184,0.7)'
}

function statusIcon(s: string): string {
  return STATUS_ICON[s] ?? '?'
}

function CheckRow({ check }: { check: DiagCheck }) {
  const c = statusColor(check.status)
  return (
    <div
      className="rounded-md px-4 py-3"
      style={{
        background:  check.repaired ? 'rgba(0,200,255,0.05)' : 'rgba(255,255,255,0.03)',
        border:     `1px solid ${c}33`,
      }}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0">
          <span
            className="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0"
            style={{ background: `${c}22`, color: c, border: `1px solid ${c}66` }}
          >
            {statusIcon(check.status)}
          </span>
          <div className="min-w-0">
            <p className="text-xs font-bold tracking-wider uppercase" style={{ color: c }}>
              {check.label}
              {check.repaired && (
                <span className="ml-2 text-ark-cyan/70 normal-case font-normal tracking-normal">
                  · repaired
                </span>
              )}
            </p>
            {check.detail && (
              <p className="text-ark-cyan/45 text-[11px] mt-0.5 font-mono break-words">
                {check.detail}
              </p>
            )}
          </div>
        </div>
        <span
          className="text-[10px] font-bold tracking-widest uppercase px-2 py-1 rounded flex-shrink-0"
          style={{ color: c, background: `${c}15`, border: `1px solid ${c}33` }}
        >
          {check.status}
        </span>
      </div>
    </div>
  )
}

export default function DiagnosticsTab({ config }: { config: ServerConfig | null }) {
  const { tk } = useI18n()
  const paths = config?.paths ?? { server_dir: '', steam_cmd_dir: '' }
  const status = useMemo(() => ({
    serverDir:    paths.server_dir,
    steamCmdDir:  paths.steam_cmd_dir,
  }), [paths.server_dir, paths.steam_cmd_dir])

  const {
    report,
    running,
    repairing,
    error,
    run,
  } = useDiagnostics(status)

  const lastTickLabel = useMemo(() => {
    if (!report) return null
    return null
  }, [report])

  return (
    <div className="space-y-5">
      <div className="space-y-1.5">
        <p className="text-ark-cyan/60 text-[10px] font-bold tracking-widest uppercase">
          {tk('section_diagnostics', 'Server Diagnostics')}
        </p>
        <p className="text-ark-cyan/40 text-[11px] leading-relaxed">
          {tk('diagnostics_intro', 'Run fixes for the three failure modes that cause "server invisible in the in-game Unofficial PC list" on ARK Survival Ascended.')}
        </p>
      </div>

      <div className="space-y-1.5">
        <p className="text-ark-cyan/45 text-[10px] uppercase tracking-widest font-bold">Paths in use</p>
        <div className="font-mono text-[11px] text-ark-cyan/60 space-y-0.5 break-all">
          <p>
            <span className="text-ark-cyan/35">server_dir&nbsp;  </span>
            {paths.server_dir || <span className="text-red-400/70">∅ not configured</span>}
          </p>
          <p>
            <span className="text-ark-cyan/35">steam_cmd_dir&nbsp;</span>
            {paths.steam_cmd_dir || <span className="text-red-400/70">∅ not configured</span>}
          </p>
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => run({ repair: false })}
          disabled={running || repairing}
          className="ark-action-btn px-5 py-2 text-xs tracking-widest disabled:opacity-40"
        >
          {running
            ? tk('diagnostics_running', '◌ RUNNING…')
            : tk('diagnostics_run', '▶ RUN CHECKS')}
        </button>
        <button
          type="button"
          onClick={() => run({ repair: true })}
          disabled={running || repairing || !paths.server_dir}
          className="px-5 py-2 text-xs tracking-widest rounded transition-colors disabled:opacity-40"
          style={{
            background: 'rgba(250,204,21,0.08)',
            border: '1px solid rgba(250,204,21,0.35)',
            color: 'rgba(250,204,21,0.85)',
          }}
          title="Patches GameUserSettings.ini Culture=en and refreshes EOS trust root.  Steam validate is left manual."
        >
          {repairing
            ? tk('diagnostics_repairing', '✦ REPAIRING…')
            : tk('diagnostics_repair', '✦ REPAIR FIXABLE')}
        </button>
      </div>

      {error && (
        <div
          className="rounded-md px-4 py-3 text-[11px] font-mono break-words"
          style={{
            background: 'rgba(248,113,113,0.06)',
            border: '1px solid rgba(248,113,113,0.35)',
            color: 'rgba(248,113,113,0.85)',
          }}
        >
          {error}
        </div>
      )}

      {!report && !running && !repairing && !error && (
        <div
          className="rounded-md px-4 py-6 text-center text-[11px] text-ark-cyan/40 tracking-widest uppercase"
          style={{ background: 'rgba(255,255,255,0.02)', border: '1px dashed rgba(0,200,255,0.18)' }}
        >
          {tk('diagnostics_empty', 'No diagnostic report yet — press RUN CHECKS.')}
        </div>
      )}

      {report && (
        <div className="space-y-2">
          {report.checks.map((c) => <CheckRow key={c.key} check={c} />)}
          <div
            className="rounded-md px-4 py-3 text-[11px] leading-relaxed"
            style={{
              background:  report.overall_ok ? 'rgba(74,222,128,0.06)' : 'rgba(250,204,21,0.05)',
              border:    `1px solid ${report.overall_ok ? 'rgba(74,222,128,0.35)' : 'rgba(250,204,21,0.35)'}`,
              color:      report.overall_ok ? 'rgba(74,222,128,0.85)' : 'rgba(250,204,21,0.85)',
            }}
          >
            <p className="font-bold tracking-widest uppercase">
              {report.overall_ok ? '✓ ALL OK' : '! NEEDS ATTENTION'}
            </p>
            <p className="mt-1 font-mono normal-case tracking-normal text-[11px]">
              {report.summary}
            </p>
          </div>
        </div>
      )}

      {lastTickLabel && null}
    </div>
  )
}
