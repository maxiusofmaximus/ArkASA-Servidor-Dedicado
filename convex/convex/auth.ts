/**
 * Sign-in / sign-out / me HTTP actions.
 *
 * The React frontend calls these via the standard Convex React hooks:
 *   - `useConvexAuth()` to read state
 *   - `useAuthActions()` from `@convex-dev/auth/react` for sign-in/out
 *
 * This file exists so we can add **bespoke** wrappers later (e.g.
 * server-side rate limiting, audit log of login attempts, etc).
 */

export { signIn, signOut, isAuthenticated } from './auth.config';
