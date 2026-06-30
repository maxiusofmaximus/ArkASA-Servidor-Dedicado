/**
 * Commands — issued by web/Telegram/Discord/etc and dispatched through
 * the Tauri loopback HTTP API on 127.0.0.1:8765.
 *
 * Auth + freshness:
 *   1. Tauri loopback HTTP requires `Authorization: Bearer <token>`.
 *   2. This backend adds HMAC over the body using the shared secret.
 *   3. The Tauri side recomputes the HMAC and rejects mismatches.
 *
 * Because Convex cannot reach the Tauri app directly (the API is loopback),
 * Hito 11's Vercel proxy bridges them. Until then we expose the API here
 * for early integration tests.
 *
 * Rate limiting is handled in `rate_limit.ts` (Hito 4).
 */
'use node';

import { action } from './_generated/server';
import { v } from 'convex/values';
import { createHmac } from 'node:crypto';

/** Same canonicalization rules as `servers.ts` so HMAC inputs match. */
function canonicalize(value: any): string {
  if (value === null) return 'null';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'number') return String(value);
  if (typeof value === 'string') return JSON.stringify(value);
  if (Array.isArray(value)) return '[' + value.map(canonicalize).join(',') + ']';
  if (typeof value === 'object' && value !== null) {
    const obj = value as Record<string, any>;
    const keys = Object.keys(obj).sort();
    return '{' + keys.map((k) => JSON.stringify(k) + ':' + canonicalize(obj[k])).join(',') + '}';
  }
  throw new Error('unsupported canonicalize input');
}

const CommandInput = v.object({
  kind: v.union(
    v.literal('start'),
    v.literal('stop'),
    v.literal('restart'),
    v.literal('status'),
    v.literal('logs'),
  ),
  map_index: v.optional(v.number()),
  tail: v.optional(v.number()),
  patch: v.optional(v.any()),
  issued_by: v.object({
    actor_id: v.string(),
    actor_name: v.string(),
    channel: v.string(),
  }),
});

const CommandOutput = v.object({
  ok: v.boolean(),
  message: v.string(),
  data: v.optional(v.any()),
});

/**
 * Issue one command. Catches HMAC/sign failures and writes the audit log.
 * The actual HTTP call to Tauri is performed by Hito 5 (scaffolded here
 * with `fetch` to `process.env.TAURI_ADMIN_URL`).
 */
export const issue = action({
  args: {
    body: CommandInput,
  },
  handler: async (ctx: any, { body }: any) => {
    // 0) Sanity — the `v` validator already typed `body`; nothing to check.

    // 1) Audit-log first so every issue is observable even if the Tauri
    //    side is unreachable.
    await ctx.runMutation(internalCommands.append_log, {
      host_id: 'unknown', // Hito 4: bind to the selected server.
      actor_id: body.issued_by.actor_id,
      actor_name: body.issued_by.actor_name,
      channel: body.issued_by.channel,
      kind: body.kind,
      map_index: body.map_index ?? null,
      payload: JSON.stringify({ tail: body.tail, patch: body.patch }),
    });

    // 2) Sign the command body
    const secret = process.env.INTEGRATIONS_SHARED_SECRET;
    if (!secret) return { ok: false, message: 'shared secret missing' };
    const sig = createHmac('sha256', secret).update(canonicalize(body)).digest('hex');

    // 3) Either call Tauri directly via fetch (Hito 5 wires this), or
    //    return a bridge-able outcome.
    const tauriBase = process.env.TAURI_ADMIN_URL ?? 'http://127.0.0.1:8765';
    const tauriToken = process.env.TAURI_ADMIN_TOKEN ?? '';
    try {
      const resp = await fetch(`${tauriBase}/api/v1/internal/dispatch`, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          authorization: `Bearer ${tauriToken}`,
          'x-ark-asa-signature': sig,
        },
        body: JSON.stringify({ cmd: body, tauri_signed: false }),
      });
      if (!resp.ok) {
        const text = await resp.text();
        return { ok: false, message: `Tauri ${resp.status}: ${text.slice(0, 200)}` };
      }
      const data = await resp.json();
      return { ok: true, message: 'forwarded to Tauri', data };
    } catch (err) {
      return { ok: false, message: `network error to Tauri: ${err}` };
    }
  },
});

import { internalMutation } from './_generated/server';
import { internal } from './_generated/server';

const internalCommands = {
  append_log: internalMutation({
    args: {
      host_id: v.string(),
      actor_id: v.string(),
      actor_name: v.string(),
      channel: v.string(),
      kind: v.string(),
      map_index: v.union(v.number(), v.null()),
      payload: v.string(),
    },
    handler: async (ctx: any, args: any) => {
      await ctx.db.insert('command_log', { ...args, at: Date.now() });
      return null;
    },
  }),
};
