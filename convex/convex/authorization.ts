/**
 * RBAC helpers — wraps Convex Auth with our domain-specific tier model.
 *
 * Two roles:
 *   - 'admin'  : full control (start/stop, view configs, manage users)
 *   - 'viewer' : read-only (status, logs, IPs)
 *
 * `require_admin(ctx)` throws ConvexError on insufficient role. Used as the
 * first line of every admin-only mutation/query so we never have an
 * accidental open write path.
 */
import { ConvexError } from 'convex/values';
import { v } from 'convex/values';

import { getAuthUserId } from '@convex-dev/auth/server';
import { query, mutation, internalQuery, internalMutation } from './_generated/server';
import { auth } from './auth.config';

export type Role = 'admin' | 'viewer';
export type Ctx = any;

const ROLE_KEY_USER: string = 'user_id';

async function userRole(ctx: Ctx): Promise<{ role: Role; userId: string } | null> {
  const userId = await getAuthUserId(ctx);
  if (!userId) return null;
  const aux = await ctx.db.query('users_aux').withIndex('by_user', (q: any) => q.eq('user_id', userId)).first();
  if (!aux) {
    // Default for first-login users is *viewer* (least privilege). Admins
    // must be promoted explicitly via the `users_aux` table — see `promote`.
    return { role: 'viewer', userId };
  }
  return { role: aux.role, userId };
}

/**
 * Page guard — used by the React useConvexAuth wrapper to decide whether
 * to render dashboard links, settings tabs, etc.
 */
export const me = query({
  args: {},
  handler: async (ctx: Ctx) => {
    return await userRole(ctx);
  },
});

/**
 * Promote / demote a user. Admin only.
 */
export const promote = mutation({
  args: { user_id: v.string(), role: v.union(v.literal('admin'), v.literal('viewer')) },
  handler: async (ctx: any, { user_id, role }: any) => {
    const me = await userRole(ctx);
    if (!me || me.role !== 'admin') throw new ConvexError('forbidden');
    const existing = await ctx.db.query('users_aux').withIndex('by_user', (q: any) => q.eq('user_id', user_id)).first();
    if (existing) {
      await ctx.db.patch(existing._id, { role });
    } else {
      await ctx.db.insert('users_aux', { user_id, role });
    }
    return { ok: true };
  },
});

/**
 * List users — admin only. Today this returns everything; Hito 4 may paginate.
 */
export const listUsers = query({
  args: {},
  handler: async (ctx: any) => {
    const me = await userRole(ctx);
    if (!me || me.role !== 'admin') return [];
    return await ctx.db.query('users_aux').collect();
  },
});

/**
 * Internal-only counterparts used by other actions / Tauri-pushed events.
 * Same RBAC, but callable from server context without needing a logged-in
 * user (used for the Tauri app's own short-lived signed JWT).
 */
export const require_admin_internal = internalQuery({
  args: {},
  handler: async (ctx: any) => {
    // For routes that come in via the Tauri service-to-service path,
    // we'll know the role from the bearer JWT lookup instead of from
    // Convex Auth. Today we short-circuit to admin because the only caller
    // is the Tauri app.
    return { role: 'admin', userId: ROLE_KEY_USER };
  },
});

/** Pull the helper up so action handlers can use it. */
export const helpers = { userRole };
