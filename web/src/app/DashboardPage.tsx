import { useState } from 'react'
import { useQuery, useAction } from 'convex/react'
import { api } from '../../convex/_generated/api'
import { useNavigate } from 'react-router-dom'
import { useAuthActions } from '@convex-dev/auth/react'
import { useRole } from '../auth/AuthGate'

type Tab = 'status' | 'mods' | 'logs' | 'users'

/**
 * Dashboard tab container.
 *
 * Three clusters of tabs:
 *   - status / mods / logs → read-only or admin-write, depends on tab
 *   - users                → admin-only (hidden for viewer tier)
 *
 * Real data source is `useQuery(api.servers.listServers)` once
 * `_generated/api.ts` has been written by `npx convex dev`.
 */
export default function DashboardPage() {
  const [tab, setTab] = useState<Tab>('status')
  const role = useRole()
  const nav = useNavigate()
  const { signOut } = useAuthActions()

  // Subscription to the Tauri-pushed state. Until `_generated/api.ts`
  // is present this throws — the catch below falls back gracefully so
  // the rest of the dashboard still renders.
  let servers: any[] = []
  try {
    servers = (useQuery(api.servers.listServers, {}) as any) ?? []
  } catch {
    servers = []
  }

  return (
    <div className="min-h-screen bg-ark-dark text-ark-cyan/90">
      <Header role={role} onSignOut={async () => { await signOut(); nav('/login') }} />

      <nav className="flex items-center gap-2 px-6 pt-4 border-b border-ark-cyan/15">
        <TabButton active={tab === 'status'} onClick={() => setTab('status')}>📊 Estado</TabButton>
        <TabButton active={tab === 'mods'}   onClick={() => setTab('mods')}>🧩 Mods</TabButton>
        <TabButton active={tab === 'logs'}   onClick={() => setTab('logs')}>📋 Logs</TabButton>
        {role === 'admin' && (
          <TabButton active={tab === 'users'} onClick={() => setTab('users')}>👥 Usuarios</TabButton>
        )}
      </nav>

      <main className="p-6">
        {tab === 'status' && <StatusTab servers={servers} role={role} />}
        {tab === 'mods'   && <ModsTab role={role} />}
        {tab === 'logs'   && <LogsTab />}
        {tab === 'users'  && role === 'admin' && <UsersTab />}
      </main>
    </div>
  )
}

// ── Header ──────────────────────────────────────────────────────────────
function Header({ role, onSignOut }: { role: string; onSignOut: () => void }) {
  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-ark-cyan/15">
      <span className="text-ark-cyan tracking-widest uppercase font-bold text-sm">
        ARK ASA Admin · v2.1
      </span>
      <div className="flex items-center gap-3 text-xs">
        <span className="text-ark-cyan/50">Role: <strong>{role}</strong></span>
        <button onClick={onSignOut} className="ark-action-btn px-3 py-1 text-[10px]">
          Sign out
        </button>
      </div>
    </header>
  )
}

// ── Tabs ────────────────────────────────────────────────────────────────
function TabButton({ active, onClick, children }: {
  active: boolean; onClick: () => void; children: React.ReactNode
}) {
  return (
    <button
      onClick={onClick}
      className="px-4 py-2 text-xs font-bold tracking-widest uppercase transition-colors"
      style={{
        color:      active ? 'rgba(0,200,255,0.9)' : 'rgba(0,200,255,0.35)',
        borderBottom: active ? '2px solid rgba(0,200,255,0.8)' : '2px solid transparent',
      }}
    >
      {children}
    </button>
  )
}

// ── Tab Bodies ─────────────────────────────────────────────────────────
function StatusTab({ servers, role }: { servers: any[]; role: string }) {
  const issue = useAction(api.commands.issue)
  if (servers.length === 0) {
    return (
      <div className="ark-panel rounded-lg p-6">
        <p className="text-ark-cyan/60 text-sm mb-2">
          No servers registered yet. Until the Tauri app has emitted at
          least one state push (every 5 s) the Convex <code>servers</code> table
          will be empty.
        </p>
        <p className="text-ark-cyan/30 text-xs italic">
          When Hito 5 is fully wired, the table below updates in real-time
          whenever the desktop app reports a new status.
        </p>
      </div>
    )
  }
  return (
    <div className="space-y-3">
      {servers.map((s: any) => (
        <div key={s._id} className="ark-panel rounded-lg p-4">
          <header className="flex items-center justify-between mb-3">
            <span className="text-ark-cyan font-semibold tracking-widest text-sm uppercase">
              {s.session_name ?? s.host_id}
            </span>
            <span className="text-ark-cyan/40 text-xs">
              {s.cluster_maps?.length ?? 0} map(s) · last seen {new Date(s.last_seen).toLocaleTimeString()}
            </span>
          </header>
          <ul className="space-y-1.5">
            {(s.map_statuses ?? []).map((m: any) => (
              <li key={m.map_index} className="flex items-center justify-between text-xs">
                <span className="text-ark-cyan/80 font-mono">{m.map_label}</span>
                <span className="font-mono">
                  <StatusBadge running={m.running} />
                </span>
              </li>
            ))}
          </ul>

          {role === 'admin' && (
            <div className="mt-3 flex gap-2">
              <button
                onClick={() => issue({
                  body: { kind: 'start', issued_by: { actor_id: 'web-ui', actor_name: 'dashboard', channel: 'web' } },
                })}
                className="ark-action-btn px-3 py-1 text-[10px]"
              >
                ▶ Start
              </button>
              <button
                onClick={() => issue({
                  body: { kind: 'stop', issued_by: { actor_id: 'web-ui', actor_name: 'dashboard', channel: 'web' } },
                })}
                className="ark-action-btn px-3 py-1 text-[10px]"
              >
                ■ Stop
              </button>
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

function StatusBadge({ running }: { running: boolean }) {
  return (
    <span
      className="text-[10px] uppercase tracking-widest px-2 py-0.5 rounded"
      style={{
        color: running ? 'rgba(74,222,128,0.9)' : 'rgba(239,68,68,0.9)',
        background: running ? 'rgba(74,222,128,0.1)' : 'rgba(239,68,68,0.1)',
        border: running ? '1px solid rgba(74,222,128,0.4)' : '1px solid rgba(239,68,68,0.4)',
      }}
    >
      {running ? '● running' : '○ stopped'}
    </span>
  )
}

function ModsTab({ role }: { role: string }) {
  return (
    <div className="ark-panel rounded-lg p-6 text-ark-cyan/60 text-sm">
      <p>Active mods list will stream from <code>api.servers.listServers</code> per host.</p>
      {role === 'admin' && (
        <p className="text-ark-cyan/40 text-xs italic mt-2">
          Admins can toggle / add / remove mods (Hito 5 fills in).
        </p>
      )}
    </div>
  )
}

function LogsTab() {
  return (
    <div className="ark-panel rounded-lg p-6 text-ark-cyan/60 text-sm">
      <p>Tail logs will render here (read-only).</p>
    </div>
  )
}

function UsersTab() {
  // `listUsers` is admin-gated server-side — returns [] for non-admins.
  let users: any[] = []
  try {
    users = (useQuery(api.authorization.listUsers, {}) as any) ?? []
  } catch {
    users = []
  }
  return (
    <div className="ark-panel rounded-lg p-6">
      <p className="text-ark-cyan/60 text-sm mb-3">User & role admin (admin-only).</p>
      {users.length === 0 ? (
        <p className="text-ark-cyan/30 text-xs italic">
          No users yet — invite yourself through the Convex dashboard or
          via Options → General → Users in the desktop app.
        </p>
      ) : (
        <ul className="space-y-1.5">
          {users.map((u: any) => (
            <li key={u._id} className="flex items-center justify-between text-xs">
              <span className="font-mono text-ark-cyan/80">{u.user_id}</span>
              <span className="ark-action-btn px-2 py-0.5 text-[10px]">{u.role}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
