import { useEffect, useState, createContext, useContext, type ReactNode } from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useConvexAuth } from 'convex/react'

type Role = 'admin' | 'viewer' | 'unknown'
const RoleContext = createContext<Role>('unknown')
export function useRole(): Role {
  return useContext(RoleContext)
}

/**
 * Authentication gate for the admin web.
 *
 * Reads the Convex Auth session via `useConvexAuth`. If unauthenticated,
 * routes to `/login`. Otherwise fetches the current tier (admin / viewer)
 * and exposes it through the `RoleContext`.
 *
 * Tier enforcement (admin vs viewer) for individual actions lives in
 * each tab / button — the gate only blocks unauthenticated access.
 *
 * NOTE: the real role fetch happens inside an `Effect` once `_generated/api.ts`
 * is present. Until that file exists (i.e. until you run `npx convex dev`
 * for the first time), we default to "viewer" so the dashboard still
 * loads in skeleton form.
 */
export function AuthGate({ children }: { children: ReactNode }) {
  const { isLoading, isAuthenticated } = useConvexAuth()
  const location = useLocation()
  const [role, setRole] = useState<Role>('unknown')

  useEffect(() => {
    if (!isAuthenticated) {
      setRole('unknown')
      return
    }
    // Async loader — once convex dev has generated `_generated/api.ts`,
    // we fetch the user's role. Until then we default to viewer.
    let cancelled = false
    ;(async () => {
      try {
        const api: any = await import('../../convex/_generated/api')
        const { useQuery }: any = await import('convex/react')

        // Pseudo-hook call: gets the current tier-as-promise via the
        // generated `api.authorization.me` selector.
        // We keep this lightweight so the gate renders immediately.
        const me = await fetchMe(api, useQuery)
        if (!cancelled) setRole(me?.role ?? 'viewer')
      } catch {
        if (!cancelled) setRole('viewer')
      }
    })()
    return () => { cancelled = true }
  }, [isAuthenticated])

  if (isLoading) {
    return <div className="min-h-screen flex items-center justify-center text-ark-cyan">Loading…</div>
  }
  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  return (
    <RoleContext.Provider value={role}>
      {children}
    </RoleContext.Provider>
  )
}

/**
 * Wrapper around `useQuery(api.authorization.me)` that resolves the role.
 * Defined as a plain function so the dynamic import can resolve
 * `useQuery` lazily without breaking the static build.
 */
async function fetchMe(api: any, _useQuery: any): Promise<{ role: 'admin' | 'viewer' } | null> {
  try {
    // We can't call hooks dynamically inside an async callback, so we
    // fall back to a fetch directly via the underlying Convex client.
    // Operators who have run `npx convex dev` will get the real role here.
    const { ConvexHttpClient }: any = await import('convex/browser')
    const url = (import.meta as any).env?.VITE_CONVEX_URL
    if (!url) return null
    const client = new ConvexHttpClient(url)
    return await client.query(api.authorization.me, {})
  } catch {
    return null
  }
}
