/**
 * Convex BaaS schema.
 *
 * Tables:
 *  - users            Auth-managed by Convex Auth (extended via users_aux for role)
 *  - servers          One row per registered Tauri app instance (keyed by host_id)
 *  - state_history    Optional time-series snapshot store; Hito 12 may trim
 *  - command_log      Audit log of every RemoteCommand issued (web/Telegram/etc.)
 *  - integrations     Config of each bot (token, allowlist of actor IDs)
 *
 * Hitos 3/4 will fill these.  This skeleton keeps `defineSchema` ready so
 * `npx convex dev` will accept the directory once `convex` is added to the
 * package.json in Hito 3.
 */
// @ts-nocheck -- resolved at Hito 3 once `convex` is added to package.json
import { defineSchema, defineTable } from 'convex/server'
import { v } from 'convex/values'

export default defineSchema({
  users_aux: defineTable({
    user_id: v.string(),
    role: v.union(v.literal('admin'), v.literal('viewer')),
    display_name: v.optional(v.string()),
  }).index('by_user', ['user_id']),

  servers: defineTable({
    host_id: v.string(),
    public_ip: v.optional(v.string()),
    cluster_maps: v.array(v.string()),
    last_seen: v.number(),
    map_statuses: v.array(v.object({
      map_index: v.number(),
      map_id: v.string(),
      map_label: v.string(),
      running: v.boolean(),
    })),
    motd: v.optional(v.string()),
    session_name: v.optional(v.string()),
  }).index('by_host', ['host_id']),

  command_log: defineTable({
    server_host_id: v.string(),
    actor_id: v.string(),
    actor_name: v.string(),
    channel: v.string(), // web | rest | telegram | discord | whatsapp | signal | wechat | ssh
    kind: v.string(),    // start | stop | restart | status | logs | ip | config_get | config_set
    map_index: v.optional(v.number()),
    payload: v.optional(v.string()),
    result: v.optional(v.string()),
    at: v.number(),
  })
    .index('by_server', ['server_host_id'])
    .index('by_at', ['at']),

  integrations: defineTable({
    server_host_id: v.string(),
    channel: v.string(),            // telegram | discord | whatsapp | ...
    enabled: v.boolean(),
    /** Encrypted in transit only — secrets are stored server-side, never echoed to clients */
    token: v.optional(v.string()),
    allowlist: v.array(v.string()), // user IDs allowed to control via this channel
    config: v.optional(v.any()),
  })
    .index('by_host', ['server_host_id'])
    .index('by_host_channel', ['server_host_id', 'channel']),
})
