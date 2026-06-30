/**
 * ARK ASA remote admin command types
 *
 * Used by the 8 integrations (Convex/web, REST, Telegram, Discord,
 * WhatsApp, Signal, WeChat, SSH). Every adapter accepts a normalized
 * `RemoteCommand` and routes it through the same pipeline.
 */
export type Role = 'admin' | 'viewer'

/** Bus message carriers. The auth/identity layer tags each command. */
export type Channel =
  | 'web'
  | 'rest'
  | 'telegram'
  | 'discord'
  | 'whatsapp'
  | 'signal'
  | 'wechat'
  | 'ssh'

export type CommandKind =
  | 'start'
  | 'stop'
  | 'restart'
  | 'status'
  | 'logs'
  | 'ip'
  | 'config_get'
  | 'config_set'

export interface RemoteCommand {
  /** Logical kind of operation the operator wants to run */
  kind: CommandKind
  /** Which map instance to target, or undefined for "always-on" cluster */
  map_index?: number
  /** Free-form key/value patch (used by config_set for TOML fields) */
  config_patch?: Record<string, unknown>
  /** How many log lines to fetch (used by logs command) */
  tail?: number
}

export interface RemoteCommandContext {
  channel: Channel
  /** Stable id of the operator (resolved from BotFather, Discord id, status etc.) */
  actor_id: string
  /** Display name resolved by the channel adapter */
  actor_name: string
  /** Role for the ACL gate — must be remapped by the channel adapter
   *  (HMAC secret + commands database -> admin / viewer). */
  role: Role
}

export interface RemoteCommandResult {
  ok: boolean
  message: string
  data?: unknown
}
