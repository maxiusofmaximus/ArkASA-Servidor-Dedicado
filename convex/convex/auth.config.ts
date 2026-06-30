/**
 * @convex-dev/auth configuration for the ark-asa admin web.
 *
 * Two providers are wired up:
 *   - Email + password  (with optional verification email)
 *   - Google OAuth      (set GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET env)
 *
 * Run `npx @convex-dev/auth --apply-convex-auth` once to seed the schema
 * additions; this file expects the auth tables (`users`, `sessions`,
 * `authAccounts`, etc) to already be present in `schema.ts`.
 */
import { convexAuth, getAuthUserId } from '@convex-dev/auth/server';
import { Anonymous } from '@convex-dev/auth/providers/Anonymous';

export const { auth, signIn, signOut, store, isAuthenticated } = convexAuth({
  providers: [
    Anonymous({ id: 'guest' }), // dev/lan convenience — no email required
    // Email/password and Google / GitHub OAuth ship as `@convex-dev/auth/providers/*`
    // — uncomment once the secret env vars are set in the Convex dashboard.
    //
    // Email: import { Email } from '@convex-dev/auth/providers/Email'
    // Google: import Google from '@convex-dev/auth/providers/Google'
    // GitHub:  import GitHub  from '@convex-dev/auth/providers/GitHub'
  ],
});

export { getAuthUserId };
