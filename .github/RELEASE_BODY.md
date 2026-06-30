## v2.1.0-alpha — Remote admin scaffolding (NOT production-ready)

This is an **alpha** release. The architecture is in place but **9 out of 11 channels need operator credentials** (Convex OAuth secrets, bot tokens, Vercel webhook deployment) before any channel is functional end-to-end.

### What's in this release

**Desktop app (`src-tauri/`)**
- Plugin registry — OpenClaw-style `Plugin` trait with capability flags (`MessagesRecv`, `MessagesSend`, `RequiresOAuth`, `RequiresSecrets`). Each channel plugin is a thin adapter.
- Loopback HTTP API on `127.0.0.1:8765` (Axum) — `Authorization: Bearer …` + JWT (HS256, rotatable).
- Public HTTP REST on `127.0.0.1:8766`.
- Command router (admin/viewer RBAC).
- Convex publisher (HMAC-signed every 5 s).
- AuthState persisted to `~/.ark-asa/`.
- Telegram bot adapter (long-poll, per-chat-id allowlist, rate-limited).

**Schema additions**
- `NetworkConfig.no_battleye`, `fixed_port_assignment_per_map`, `allow_start_without_internet`.
- ARK launcher uses FNV-1a hash of map_id → stable port triplet regardless of boot order (fixes the wrong-character-save bug).

**Frontend (`frontend/`)**
- `useInternetStatus` polling + ActionBar offline-gate + bottom-right banner.
- Fixed 3-container ArksTab layout.
- Region info row (read-only — explains why ARK Steam/EOS show blank).
- 3 toggles in `Options → General`: Internet / Server Cluster (port map) / BattleEye.

**Convex BaaS (`convex/`)**
- Schema: `users_aux`, `servers`, `command_log`, `integrations`, `auth_rejections`.
- `servers.ts` — HMAC-signed upsert + queries.
- `commands.ts` — issues commands to the Tauri loopback with HMAC body signature.
- `auth.ts` + `auth.config.ts` — Convex Auth wiring with Anonymous provider ready.
- `authorization.ts` — admin/viewer RBAC.

**Web admin (`web/`)**
- Vite + React + TS scaffold with `vercel.json` for auto-deploy.
- LoginPage + DashboardPage (Estado / Mods / Logs / Usuarios), role-gated.
- AuthGate + RoleContext via `useConvexAuth`.

### Setup checklist before any channel works

1. Create a Convex project: https://dashboard.convex.dev → save `CONVEX_URL` and `CONVEX_SHARED_SECRET` env vars.
2. After cloning: `pnpm install && npx convex dev`.
3. (Optional) Set Google/GitHub OAuth client IDs in Convex dashboard.
4. Deploy the web admin: `npm i -g vercel && vercel login && vercel --prod` (from the `/web` folder).
5. Drop bot tokens + secrets for each channel you want active:

```toml
[plugins.telegram]
enabled = true
secrets.bot_token = "123456:ABC-DEF…"
secrets.admin_chat_ids = [987654321]

[plugins.discord]
enabled = true
secrets.bot_token = "XXXXXXXX…"
secrets.guild_id = "1234567890"
```

See `CHANGELOG.md` for the full manual-setup matrix.

### Known TODOs left in code (intentional)

- `src-tauri/src/integrations/telegram.rs::spawn_looper` is currently a stub — needs a real `tokio::spawn` integration that doesn't break the `Send` bound.
- `src-tauri/src/integrations/http_api.rs` start/stop handlers return `RouterOutcome::Error { reason: 'bridged at Hito 12' }` placeholder.
- `/api/v1/logs` returns empty until `arc/logs.rs::tail_log` is wired.
- Each channel plugin (Discord, WhatsApp, Signal, WeChat, SSH, HTTP public) is currently a stub or absent — `Plugin` trait is the contract.

### What's verified

```
cargo check     → 0 errors (3 warnings)
npx tsc --noEmit → 0 errors in all 5 packages
npx vite build   → build OK in frontend + web
```

The desktop installer `ARK-ASA-Full-Setup-2.1.exe` has not been built yet — pending `cargo tauri build` on a Windows host with NSIS in PATH.

### Roadmap to v2.1.0 GA

1. Operator creates Convex + bot accounts (~1 hour).
2. Reconcile the `Send`-binding in `spawn_looper` for Telegram.
3. Finish the HTTP API ↔ ARK launcher bridge.
4. Build the NSIS installer.
5. Re-tag as `v2.1.0` (drop `-alpha`).
