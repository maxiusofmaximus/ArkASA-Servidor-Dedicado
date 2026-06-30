import { useState } from 'react'

type Tab = 'status' | 'mods' | 'logs' | 'users'

/**
 * Dashboard — tab container for the admin views.
 *
 * Tabs:
 *  - status  → live server state + Start/Stop buttons (admin only)
 *  - mods    → installed mods + safe-toggle (admin only)
 *  - logs    → tail logs (any role)
 *  - users   → user & role admin (admin only — *not visible to viewers*)
 *
 * Today this renders the layout skeleton only; Hito 5 will plug live data
 * from Convex subscriptions + the Hito 1 shared types.
 *
 * NOTE: role-based render gates are already present so Hito 5 doesn't have
 * to retro-wire ACL.
 */
export default function DashboardPage() {
  const [tab, setTab] = useState<Tab>('status')
  // TODO H4/H5: read role from Convex Auth; default until then
  const role: 'admin' | 'viewer' = 'admin'

  return (
    <div className="min-h-screen bg-ark-dark text-ark-cyan/90">
      <Header role={role} />

      <nav className="flex items-center gap-2 px-6 pt-4 border-b border-ark-cyan/15">
        <TabButton active={tab === 'status'} onClick={() => setTab('status')}>
          📊 Estado
        </TabButton>
        <TabButton active={tab === 'mods'} onClick={() => setTab('mods')}>
          🧩 Mods
        </TabButton>
        <TabButton active={tab === 'logs'} onClick={() => setTab('logs')}>
          📋 Logs
        </TabButton>
        {role === 'admin' && (
          <TabButton active={tab === 'users'} onClick={() => setTab('users')}>
            👥 Usuarios
          </TabButton>
        )}
      </nav>

      <main className="p-6">
        {tab === 'status' && <StatusTab role={role} />}
        {tab === 'mods' && <ModsTab role={role} />}
        {tab === 'logs' && <LogsTab />}
        {tab === 'users' && role === 'admin' && <UsersTab />}
      </main>
    </div>
  )
}

function Header({ role }: { role: 'admin' | 'viewer' }) {
  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-ark-cyan/15">
      <span className="text-ark-cyan tracking-widest uppercase font-bold text-sm">
        ARK ASA Admin · v2.1
      </span>
      <span className="text-ark-cyan/50 text-xs">
        Role: <strong>{role}</strong>
      </span>
    </header>
  )
}

function TabButton({
  active, onClick, children,
}: {
  active: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      onClick={onClick}
      className="px-4 py-2 text-xs font-bold tracking-widest uppercase transition-colors"
      style={{
        color: active ? 'rgba(0,200,255,0.9)' : 'rgba(0,200,255,0.35)',
        borderBottom: active ? '2px solid rgba(0,200,255,0.8)' : '2px solid transparent',
      }}
    >
      {children}
    </button>
  )
}

// Tab placeholders — replaced in Hito 5 with live data.
function StatusTab({ role }: { role: 'admin' | 'viewer' }) {
  return (
    <div className="ark-panel rounded-lg p-6">
      <p className="text-ark-cyan/60">Live server state will render here.</p>
      {role === 'admin' && (
        <div className="mt-4">
          <button className="ark-action-btn px-5 py-2 mr-2">▶ Start Server</button>
          <button className="ark-action-btn px-5 py-2">■ Stop Server</button>
        </div>
      )}
      {role === 'viewer' && (
        <p className="text-ark-cyan/40 text-xs italic mt-2">
          (Viewer role — Start/Stop hidden)
        </p>
      )}
    </div>
  )
}

function ModsTab({ role }: { role: 'admin' | 'viewer' }) {
  return (
    <div className="ark-panel rounded-lg p-6">
      <p className="text-ark-cyan/60">Installed mods will render here.</p>
      {role === 'admin' && (
        <p className="text-ark-cyan/40 text-xs italic mt-2">
          Admin can toggle / add / remove mods.
        </p>
      )}
    </div>
  )
}

function LogsTab() {
  return (
    <div className="ark-panel rounded-lg p-6">
      <p className="text-ark-cyan/60">Tail logs will render here (read-only).</p>
    </div>
  )
}

function UsersTab() {
  return (
    <div className="ark-panel rounded-lg p-6">
      <p className="text-ark-cyan/60">User & role admin will render here (admin-only).</p>
    </div>
  )
}
