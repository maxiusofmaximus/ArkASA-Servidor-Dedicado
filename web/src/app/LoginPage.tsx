/**
 * Login page for the ARK ASA admin web.
 *
 * Three sign-in buttons:
 *  1. Anonymous (dev/local convenience — auth provider `guest`)
 *  2. Email + password (Phase 2 — wires up a credential provider)
 *  3. Google OAuth (Phase 2 — wires up Google's provider)
 *
 * Phase 1 today ships just the Anonymous button so the dashboard can
 * be deployed end-to-end. Once the operator has provisioned a Convex
 * deployment + provider secrets (see `docs/CONVEX_SETUP.md`) the other
 * providers become live.
 */
import { useState } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { useAuthActions } from '@convex-dev/auth/react'
import { useConvexAuth } from 'convex/react'

export default function LoginPage() {
  const { signIn } = useAuthActions()
  const { isAuthenticated } = useConvexAuth()
  const nav = useNavigate()
  const loc = useLocation()
  const [busy, setBusy] = useState<string | null>(null)
  const [err, setErr] = useState<string | null>(null)

  if (isAuthenticated) {
    const dest = (loc.state as any)?.from?.pathname ?? '/dashboard'
    nav(dest, { replace: true })
    return null
  }

  async function signInAs(provider: 'guest' | 'google' | 'github' | 'email') {
    setBusy(provider)
    setErr(null)
    try {
      await signIn(provider)
      nav('/dashboard', { replace: true })
    } catch (e: any) {
      setErr(e?.message ?? String(e))
    } finally {
      setBusy(null)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-ark-dark">
      <div className="ark-panel rounded-lg p-8 max-w-md w-full">
        <h1 className="text-ark-cyan text-2xl font-bold tracking-widest uppercase text-center mb-2">
          ARK ASA Admin
        </h1>
        <p className="text-ark-cyan/40 text-xs text-center tracking-widest uppercase mb-6">
          Sign in to manage the server remotely
        </p>

        <div className="space-y-2">
          <button
            disabled={busy !== null}
            onClick={() => signInAs('guest')}
            className="w-full ark-action-btn py-2 text-xs disabled:opacity-40"
          >
            {busy === 'guest' ? 'Signing in…' : '⚡ Sign in as guest (no email)'}
          </button>
          <button
            disabled
            className="w-full ark-action-btn py-2 text-xs opacity-50"
            title="Phase 2 — requires Convex Email provider configured"
          >
            📧 Email + password
          </button>
          <button
            disabled
            className="w-full ark-action-btn py-2 text-xs opacity-50"
            title="Phase 2 — requires GOOGLE_CLIENT_ID env var in Convex dashboard"
          >
            🟦 Google
          </button>
        </div>

        {err && (
          <p className="mt-4 text-ark-accent text-xs text-center">{err}</p>
        )}

        <p className="mt-6 text-ark-cyan/30 text-xs text-center italic">
          First-login users default to the *viewer* tier. Ask an admin to promote you
          to *admin* via Options → General → Users.
        </p>
      </div>
    </div>
  )
}
