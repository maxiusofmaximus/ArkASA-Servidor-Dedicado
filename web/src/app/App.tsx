import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import LoginPage from './LoginPage'
import DashboardPage from './DashboardPage'
import { AuthGate } from '../auth/AuthGate'

/**
 * Root component for the ARK ASA Admin web app.
 *
 * Routing:
 *  - /login       Email / OAuth login (Convex Auth)
 *  - /dashboard   Auth-gated tabs (Estado, Servidores, Mods, Logs)
 *  - /users       Admin-only user & role management
 *  - everything else: redirect to /dashboard
 */
export default function App() {
  return (
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
  )
}
