/**
 * @ark-asa/shared-types
 *
 * Shared TypeScript types used by both the Tauri desktop app and the
 * Vercel-hosted admin web app. Mirrors `src-tauri/src/config/schema.rs`.
 *
 * Import from this package as `@ark-asa/shared-types` — both
 * `frontend/src/types/` consumers and `web/` consumers share one
 * source of truth.
 *
 * IMPORTANT: when adding a field, add it to BOTH `schema.rs` and
 * `config.ts` to prevent drift.
 */
export * from './command.js'
export * from './config.js'
