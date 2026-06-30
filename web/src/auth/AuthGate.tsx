import { useEffect } from 'react'
import { Navigate, useLocation } from 'react-router-dom'

/**
 * Placeholder AuthGate — will be replaced in Hito 4 once the Convex Auth
 * provider is plumbed.  Today it simply routes anonymous users to /login.
 */
export function AuthGate({ children }: { children: React.ReactNode }) {
  const location = useLocation()
  // TODO H4: get auth state from useConvexAuth()
  const signedIn = false

  useEffect(() => {
    if (!signedIn) {
      // placeholder, no-op
    }
  }, [signedIn])

  return signedIn ? <>{children}</> : <Navigate to="/login" state={{ from: location }} replace />
}
