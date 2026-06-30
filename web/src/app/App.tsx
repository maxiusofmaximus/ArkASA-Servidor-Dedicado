import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { ConvexProvider, ConvexReactClient } from 'convex/react'
import LoginPage from './LoginPage'
import DashboardPage from './DashboardPage'
import { AuthGate } from '../auth/AuthGate'

/**
 * Convex client. Reads `VITE_CONVEX_URL` at build time. In Vercel the
 * env var is configured under "Project Settings → Environment Variables".
 *
 * For local dev, drop a `.env.local` with:
 *   VITE_CONVEX_URL=https://your-convex-deployment.convex.cloud
 */
const convexUrl = (import.meta.env.VITE_CONVEX_URL as string | undefined) ?? ''
const convex = convexUrl ? new ConvexReactClient(convexUrl) : null

export default function App() {
  if (!convex) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-ark-dark text-ark-cyan">
        <div className="ark-panel p-8 max-w-md text-center rounded-lg">
          <h1 className="text-xl font-bold mb-2">Ark ASA Admin — configuration required</h1>
          <p className="text-sm text-ark-cyan/60 mb-1">
            Set <code className="text-ark-accent">VITE_CONVEX_URL</code> to deploy.
          </p>
          <p className="text-xs text-ark-cyan/30">
            See <code>docs/VERCEL_DEPLOY.md</code> (Hito 12).
          </p>
        </div>
      </div>
    )
  }

  return (
    <ConvexProvider client={convex}>
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route
            path="/dashboard"
            element={
              <AuthGate>
                <DashboardPage />
              </AuthGate>
            }
          />
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </BrowserRouter>
    </ConvexProvider>
  )
}
