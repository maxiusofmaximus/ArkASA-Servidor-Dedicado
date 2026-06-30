/**
 * Server state — inbound push from the Tauri app's `convex_push` module.
 *
 * Endpoint contract:
 *   POST /api/internal/servers/upsert
 *   body: StatePushPayload (JSON) with a `signature_hex` HMAC-SHA256 of the
 *         canonicalized body using a shared secret.
 *
 * Authentication: HMAC verified against the secret stored in the
 * `INTEGRATIONS_SHARED_SECRET` Convex env var (configure in Convex dashboard).
 * If the env var is unset the action refuses all writes — fail-secure default.
 *
 * The same canonicalization rules used by the Tauri publisher are inlined
 * here so signer and verifier agree on what is signed.
 */
'use node';

import { action, query, mutation } from './_generated/server';
import { v } from 'convex/values';
import { createHmac } from 'node:crypto';

const HMAC_FIELD = 'signature_hex';

function canonicalize(value: unknown): string {
  if (value === null) return 'null';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'number') return String(value);
  if (typeof value === 'string') return JSON.stringify(value);
  if (Array.isArray(value)) return '[' + value.map(canonicalize).join(',') + ']';
  if (typeof value === 'object' && value !== null) {
    const obj = value as Record<string, unknown>
    const keys = Object.keys(obj).sort()
    return '{' + keys.map((k) => JSON.stringify(k) + ':' + canonicalize(obj[k])).join(',') + '}'
  }
  throw new Error('unsupported canonicalize input')
}
function sign(body: any, secret: string): string {
  const mut = { ...body }
  delete mut[HMAC_FIELD]
  return createHmac('sha256', secret).update(canonicalize(mut)).digest('hex')
}

const MapStatus = v.object({
  map_index: v.number(),
  map_id: v.string(),
  map_label: v.string(),
  running: v.boolean(),
});

const PushInput = v.object({
  host_id: v.string(),
  session_name: v.string(),
  motd: v.optional(v.string()),
  cluster_maps: v.array(v.string()),
  map_statuses: v.array(MapStatus),
  last_seen_ms: v.number(),
  signature_hex: v.string(),
});

/**
 * Upsert one server row keyed by host_id. Called by the Tauri app every 5 s.
 *
 * Public-by-design (no user auth): accesses control via HMAC.  Treat the
 * shared secret like an API key — never log it, never echo it back.
 */
export const upsert = action({
  args: { body: v.any() },
  handler: async (ctx: any, { body }: { body: any }) => {
    const secret = process.env.INTEGRATIONS_SHARED_SECRET;
    if (!secret) throw new Error('INTEGRATIONS_SHARED_SECRET is not configured');
    const parsed = body as {
      host_id: string; session_name: string;
      motd?: string | null; cluster_maps: string[];
      map_statuses: Array<{ map_index: number; map_id: string; map_label: string; running: boolean }>;
      last_seen_ms: number; signature_hex: string;
    };
    if (sign(parsed, secret) !== parsed.signature_hex) {
      throw new Error('HMAC mismatch; refusing state push');
    }
    const existing = await ctx.runQuery((internalServers).find_by_host, { host_id: parsed.host_id });
    if (existing) {
      await ctx.runMutation((internalServers).update_state, {
        id: existing._id,
        session_name: parsed.session_name,
        motd: parsed.motd ?? null,
        cluster_maps: parsed.cluster_maps,
        map_statuses: parsed.map_statuses,
        last_seen_ms: parsed.last_seen_ms,
      });
    } else {
      await ctx.runMutation((internalServers).insert_state, {
        host_id: parsed.host_id,
        session_name: parsed.session_name,
        motd: parsed.motd ?? null,
        cluster_maps: parsed.cluster_maps,
        map_statuses: parsed.map_statuses,
        last_seen_ms: parsed.last_seen_ms,
      });
    }
    return { ok: true };
  },
});

// ── Internal queries / mutations ────────────────────────────────────────
// The two paths below are `internal*` so they are not callable from the
// web client directly — only from the action above.

import { internalQuery, internalMutation } from './_generated/server';

const internalServers = {
  find_by_host: internalQuery({
    args: { host_id: v.string() },
    handler: async (ctx: any, args: { host_id: string }) => {
      return await ctx.db.query('servers').withIndex('by_host', (q: any) => q.eq('host_id', args.host_id)).first();
    },
  }),
  insert_state: internalMutation({
    args: {
      host_id: v.string(),
      session_name: v.string(),
      motd: v.union(v.string(), v.null()),
      cluster_maps: v.array(v.string()),
      map_statuses: v.array(MapStatus),
      last_seen_ms: v.number(),
    },
    handler: async (ctx: any, args: any) => {
      await ctx.db.insert('servers', args);
      return null;
    },
  }),
  update_state: internalMutation({
    args: {
      id: v.id('servers'),
      session_name: v.string(),
      motd: v.union(v.string(), v.null()),
      cluster_maps: v.array(v.string()),
      map_statuses: v.array(MapStatus),
      last_seen_ms: v.number(),
    },
    handler: async (ctx: any, args: any) => {
      const { id, ...rest } = args;
      await ctx.db.patch(id, rest);
      return null;
    },
  }),
};

// ── Frontend-facing queries ─────────────────────────────────────────────

export const listServers = query({
  args: {},
  handler: async (ctx: any) => {
    return await ctx.db.query('servers').collect();
  },
});

export const getServer = query({
  args: { host_id: v.string() },
  handler: async (ctx: any, args: { host_id: string }) => {
    return await ctx.db
      .query('servers')
      .withIndex('by_host', (q: any) => q.eq('host_id', args.host_id))
      .first();
  },
});

export const get = query({
  args: { host_id: v.string() },
  handler: async (ctx, { host_id }) => {
    return await ctx.db
      .query('servers')
      .withIndex('by_host', (q) => q.eq('host_id', host_id))
      .first();
  },
});
