# Changelog

All notable changes to this project are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/), and the project
adheres to [Semantic Versioning](https://semver.org/).

## [v2.1.0-alpha] — 2026-06-30 — INCOMPLETE / CONFIGURATION REQUIRED

> **Status: ALPHA — Not production-ready.**
> Convex OAuth + Vercel deploy + per-bot credentials are wired in code
> but **require operator secrets** before any channel works end-to-end.
> See `docs/PLUGINS.md` (TODO Hito 12) and `docs/INTEGRATIONS.md`.

### Added — desktop app

- **Plugin registry** (`src-tauri/src/plugins/mod.rs`): OpenClaw-style
  plugin trait with capability flags (`MessagesRecv`, `MessagesSend`,
  `RequiresOAuth`, `RequiresSecrets`) and a fail-soft startup pattern.
  Each external channel is a thin adapter implementing `start(ctx)`.
- **Loopback HTTP API** (`src-tauri/src/integrations/http_api.rs`):
  Axum v0.7 server bound **strictly** to `127.0.0.1:8765`. Endpoints:
  - `GET  /api/v1/health`
  - `GET  /api/v1/state`
  - `GET  /api/v1/logs?tail=N`
  - `GET  /api/v1/config`
  - `POST /api/v1/start` / `/stop` / `/restart`
  - `POST /api/v1/internal/dispatch` (Convex service-to-service)
  Bearer auth + JWT (`jsonwebtoken` HS256) + per-token rotation.
- **Public HTTP REST** on `127.0.0.1:8766` (`http_commands.rs`) for
  bearer-token access without bound loopback.
- **Command Router** (`integrations/command_router.rs`): single internal
  interface for all 8 adapters. Drafted with `authorize(ctx, cmd)` so
  every channel honors the same admin / viewer RBAC.
- **Convex publisher** (`integrations/convex_push.rs`): every 5 s pushes
  the cluster state (game/query ports, MOTD, map statuses) to Convex
  via HMAC-SHA256 signed body.
- **AuthState** (`src-tauri/src/auth/mod.rs`): JWT signing secret
  persisted in `~/.ark-asa/admin.jwt`; first-run token in
  `~/.ark-asa/admin.token`. New Tauri commands:
  - `admin_token`           → returns current bearer token
  - `rotate_admin_token`    → rotates secret + token
- **Telegram bot adapter** (`integrations/telegram.rs`): long-polling
  against `api.telegram.org/bot<token>/getUpdates`; per-chat-id allowlist,
  1 cmd / 5 s rate limit; maps `/start /stop /restart /status /logs /ip`
  to normalized `RemoteCommand`s.
- **Schema additions**: `NetworkConfig.no_battleye`,
  `NetworkConfig.fixed_port_assignment_per_map`,
  `NetworkConfig.allow_start_without_internet`.
- **Launcher change**: `ark/launcher.rs::build_launch_args` honours
  `fixed_port_assignment_per_map` via a new `ports_for_map_id(...)`
  helper using FNV-1a hashing so each map always lands on the same
  triplet regardless of boot order.
- **Internet-gated start** (`useInternetStatus` + ActionBar disabled
  state + bottom-right banner) prevents silent crashes from offline
  boots.
- **Single-cluster start bug fix**: `start_server_instance` now does a
  proper **UDP-port** check (not TCP-of-RCON) + a process-existence
  check on `ArkAscendedServer.exe` → fixed.

### Added — web admin (Vercel-hosted, Convex auth)

- `web/` Vite + React + TS scaffold (`/web/package.json` +
  `vercel.json` for auto-deploy).
- `docs/index.html` + `GitHub Pages` v2.1 banner updated.
- LoginPage with Anonymous (guest) provider; Google/GitHub/Email
  placeholders ready for OAuth secret env-vars.
- DashboardPage: 4 tabs (Estado/Mods/Logs/Usuarios), admin-only Start/Stop
  dispatch, role-gated Users tab.
- AuthGate + RoleContext using `useConvexAuth`.

### Added — Convex BaaS

- `convex/convex/schema.ts`: `users_aux`, `servers`, `command_log`,
  `integrations`, `auth_rejections` tables with `by_host`, `by_at`,
  `by_user`, `by_host_channel` indexes.
- `convex/convex/servers.ts`: HMAC-signed `upsert` action +
  `find_by_host`/`insert_state`/`update_state` internal helpers +
  frontend-facing `listServers`/`getServer`.
- `convex/convex/commands.ts`: `issue()` proxies a `CommandInput`
  to the Tauri loopback (configurable `TAURI_ADMIN_URL`) with HMAC
  body signature.
- `convex/convex/auth.ts`: re-exports `@convex-dev/auth`'s
  `signIn/signOut/isAuthenticated`.
- `convex/convex/auth.config.ts`: `convexAuth` with Anonymous provider
  ready; OAuth providers commented for operator secret setup.
- `convex/convex/authorization.ts`: `me`, `promote`, `listUsers`
  enforcing the admin/viewer tier model.

### Added — docs

- `docs/index.html` v2.1 footer + features blurb.
- `README.md` updated with v2.1 features and roadmap markers.
- `installer/setup.nsi` version bumped to 2.1.0; output filename
  `ARK-ASA-Full-Setup-2.1.exe`.
- `Cargo.toml` + `package.json` versions bumped to 2.1.0.
- `.gitignore` extended (Vercel, Convex, session artifacts).

### Changed

- Frontend dev dependencies aligned to **pnpm 11** workspaces via
  `pnpm-workspace.yaml`. Monorepo includes `frontend`, `web`,
  `convex`, and `packages/shared-types`.
- Settings UI: removed BACK button from Options → Actions; LOGS moved
  to Options → Actions; IP Info lives as a per-row icon button on the
  ConnectionManager.
- SettingRow now exposes a **show/hide** + **copy** action column on
  `secret` and `copyable` types, separated by a vertical line.

---

## ⚠ What's NOT done — manual setup required to ship v2.1.0 GA

These are the pieces that need a real account / CLI to enable. The code
exists but cannot run autonomously:

| Concern | What you must do |
|---------|------------------|
| Convex account | Sign up at convex.dev, create a project, set `CONVEX_URL` env var in your build. |
| Convex `INTEGRATIONS_SHARED_SECRET` | Set in Convex dashboard and `.env.local`. |
| Convex auth providers | Configure Google / GitHub OAuth client IDs (optional). |
| Vercel web deploy | `npm i -g vercel && vercel login && vercel --prod`. |
| Telegram bot | Create via `@BotFather`, store token in TOML `[plugins.telegram].secrets.bot_token`. |
| Discord bot | Create via Discord dev portal, enable Message Content Intent, store `bot_token` + `guild_id`. |
| WhatsApp Business | Set up WABA + Meta Graph API; store `phone_id` + `business_id` + `webhook_secret`. |
| Signal | Run `signal-cli` daemon; store `phone_e164` in plugins. |
| WeChat OA | Set up Official Account; store `app_id` + `app_secret`. |
| SSH | Generate `authorized_keys`; place in `[plugins.ssh].secrets.authorized_keys`. |
| HTTPS public webhook | Deploy a tiny /api/webhooks/{channel}/sink on Vercel so Convex can call the **Tauri** loopback via HTTPS. |
| NSIS sign | If signing Windows installer, configure `signtool` creds. |

### Code that's tagged as TODO/open work

- `src-tauri/src/integrations/telegram.rs::spawn_looper` — the long-poll
  loop's `Send` constraints still need polishing in Hito 12.
- `src-tauri/src/integrations/http_api.rs` — all `start/stop/restart`
  stubs return `RouterOutcome::Error` with reason "bridged to
  launcher in lib.rs at Hito 12". The bridge to the actual ARK launcher
  is the next implementation step.
- Each integration directory (`discord.rs`, `whatsapp.rs`, `signal.rs`,
  `wechat.rs`, `ssh.rs`, `http_commands.rs`) — currently `#[allow(dead_code)]`
  stubs or placeholders. The Plugin trait is the contract for plugging
  them in.
- Real log line streaming (`/api/v1/logs`) returns empty until
  `arc/logs.rs::tail_log` is wired into the HTTP layer.
- `convex/convex/servers.ts::upsert` audit log: today writes only state;
  command_log ride-along is in `convex/convex/commands.ts::issue`
  but the action-bound schema field `host_id: 'unknown'` should be
  resolved against the selected Tauri host later (Hito 12).

## [v2.0.0] — 2026-06 — release

Multi-server, form editor, undo/redo (commit f3a224b).

## [v1.4.0] — release

Undo/redo, i18n, tooltips, text truncation (commit b045698).

## [v1.3.0] — release

Connection Manager, Friend Contacts (commit 39b26ae).

## [v1.2.0] — release

On-demand notifications (commit 747bfec — partial mention).

## [v1.1.0] — release

Server management + cloud backup.

## [v1.0.0] — release

Core configuration management done.

[Unreleased]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/compare/v2.1.0-alpha...HEAD
[v2.1.0-alpha]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/tag/v2.1.0-alpha
[v2.0.0]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/tag/v2.0.0
