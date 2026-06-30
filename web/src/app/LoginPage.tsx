/**
 * Login page
 *
 * In Hito 4 we'll fill this with Convex Auth providers (email/password + Google).
 * For now, the page just confirms the panel exists and links to the dashboard.
 */
export default function LoginPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-ark-dark">
      <div className="ark-panel rounded-lg p-8 max-w-md w-full text-center">
        <h1 className="text-ark-cyan text-2xl font-bold tracking-widest uppercase mb-4">
          ARK ASA Admin
        </h1>
        <p className="text-ark-cyan/60 text-sm mb-6">
          Sign in to manage your ARK server remotely.
        </p>
        <p className="text-ark-cyan/30 text-xs italic">
          Login providers will be wired in Hito 4 (Convex Auth).
        </p>
        <a
          href="/dashboard"
          className="inline-block mt-6 ark-action-btn px-6 py-2 text-xs tracking-widest"
        >
          Continue to Dashboard
        </a>
      </div>
    </div>
  )
}
