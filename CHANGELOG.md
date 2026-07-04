# Changelog

All notable changes to this project are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/), and the project
adheres to [Semantic Versioning](https://semver.org/).

---

> ## v2.1.0-alpha is still INCOMPLETE.
>
> This branch is **untagged** alpha work toward CLI-bridge plugins
> (Convex, Vercel, hosting) and bot pipeline. **Do not ship it as
> v2.3.0 yet.**
>
> **v2.1 cannot be declared complete until:**
> 1. **Remote Convex backend** — deployable from the app (currently
>    requires operator secrets at runtime; no first-class `push schema`
>    button from the desktop UI to a tenant-owned Convex cloud).
> 2. **Hosted Vercel web admin** — the operator clicks **Deploy web**
>    and `vercel deploy --prod` produces a real URL they can share
>    with players on phones / other machines. Today this only stubs
>    a CLI-bridge plugin without a baked deploy flow.
> 3. **Self-hosted VPS bootstrap** — for users who don't want a BaaS
>    and just want to run the desktop on a Raspberry Pi 5 / spare PC.
>    The 7-provider CLI runners ship the *script* but there's no
>    end-to-end "I have an IPv4 → ARK running in 10 minutes" guide
>    validated on real hardware (Pi 4/5, Intel NUC, old workstation).
>
> Until those three land, this file's `[Unreleased]` section is the
> source of truth. Nothing in `main` deserves a version bump yet.

## [Unreleased] — alpha work toward v2.1.0

### Added — bot adapters & identity

- **`integrations::identity::Identity` 7-axis model** — every chat-bot
  inbound now resolves to a `Identity { platform, accountId, channelId,
  userId, agentId, sessionKey, runtimeClass }` snapshot before
  `Bridge::dispatch` is reached. Mirrors Agent Harness Core / OpenClaw
  contracts. `ChannelBinding::resolve()` is **fail-closed** (`AllowlistMiss`
  + `MalformedActorId` rejections).
- **ALL adapters** (Telegram, Discord, Slack) now emit the full 5-stage
  pipeline per inbound message via the new `receipt_emit::Emitter`:
  `ChannelIngress → IdentityCheck → [QueueEnqueue] → RuntimePipeline →
  ChannelDelivery`. Slack additionally emits `QueueEnqueue` because its
  Socket-Mode contract requires ACK in ≤3s.
- **`receipts::ReceiptLedger`** — JSONL append-only ledger
  under `${AppData}/receipts/YYYY-MM-DD.jsonl`, fsync-on-write, daily
  rotation. Three new Tauri commands: `receipts_probe`,
  `receipts_today_path`, `receipts_tail(n)`. Initial entry written at
  boot with `loopback_port=8765` and `loopback_host_id`.

### Added — chat adapters

- **Discord bot** (`integrations::discord`) — real Discord Gateway v10
  WebSocket client; identify/heartbeat/MESSAGE_CREATE; supports admins
  allowlist, AI-natural-language through `[COMMAND: ...]` tag (same
  protocol as Telegram).
- **Slack bot** (`integrations::slack`) — Socket Mode WebSocket, no
  public HTTPS endpoint needed. App-Level Token + Bot OAuth token.
- **Telegram bot upgrade** — same 7-axis pipeline; rate-limit per
  `chat_id` (3s) preserved.

### Added — hosting CLI runners (target for v2.1 completion)

- **`render_provider_run_script(target, bundle_url)`** Tauri command —
  wraps the cloud-init into a **single bash file** the operator copies
  onto their authenticated workstation. Each provider dispatches to its
  official CLI: `hcloud` / `doctl` / `aws` / `az` / `gcloud` / `oci` or
  `rsync+ssh` for self-hosted. We deliberately do NOT reimplement OAuth
  flows in Rust.
- New tests covering Hetzner, DigitalOcean, AWS, GCP, Azure, Oracle and
  Self-hosted path parsing.

### Added — hosting & database tabs (frontend)

- `frontend/src/components/options/HostingTab.tsx` — provider dropdown
  + form + generate script + generate CLI runner + copy-to-clipboard.
- `frontend/src/components/options/DatabaseTab.tsx` — backend dropdown
  + validate button + status indicator.
- New primitives `Select` and `TextArea` in
  `frontend/src/components/ui/OptionsUI.tsx`.
- 5-language i18n (EN/ES/DE/PT/FR) updated for new tabs.

### Added — Convex One-Click Deploy (alpha toward v2.1)

- **New Tauri command** `convex_deploy(deployment_url, deploy_key)`
  in `src-tauri/src/plugins/convex/mod.rs`. Composes the existing
  `paste_convex_deploy_key` (persists creds to secret store) followed
  by `convex_push_schema` (`npx convex deploy --prod`).
- **New DatabaseTab section** "Convex One-Click Deploy" — appears
  only when the backend is `Convex (BaaS)`. Operator pastes their
  deployment URL & deploy key, clicks **DEPLOY TO CONVEX**, and
  watches the schema push log stream into the panel.
- **i18n** added in EN/ES/DE/PT/FR for the new section.
- **New test** `plugins::convex::tests::test_convex_deploy_persists_credentials`
  asserts credentials are saved correctly and never panics. Test count
  is now **67 passing**.

### Added — Vercel One-Click Deploy (alpha toward v2.1)

- **New Tauri command** `vercel_deploy_one_click(token, project_id)`
  in `src-tauri/src/plugins/vercel/mod.rs`. Composes the existing
  `paste_vercel_token(token, project_id)` (persists token to secret
  store) followed by `vercel_deploy_web()` which runs
  `vercel deploy --prod --yes`.
- **VercelCard in GeneralTab.tsx** gains a collapsible
  "⚡ One-Click Deploy (paste a token)" section. Operator types the
  Vercel token + optional project id into `<input>` fields and
  clicks **DEPLOY**. The `vercel CLI` stdout streams into a
  scrollable `<pre>` panel; the production URL (`.vercel.app`) is
  auto-extracted and stored on success. Existing
  `Connect Vercel` / `Deploy web` CLI flow remains as a fallback
  for operators who already have `vercel` installed locally.
- **New tests**:
  - `plugins::vercel::tests::parse_vercel_url_from_output_works` —
    covers the URL extraction helper (positive + negative).
  - `plugins::vercel::tests::test_vercel_deploy_one_click_persists_token` —
    asserts the token + project_id are persisted to the secret store
    before any shell-spawn, and that no leak survives cleanup.
- **`docs/VERCEL.md`** — operator guide: prerequisites (token at
  vercel.com/account/tokens), flow internals, security rationale,
  pre-flighting panel + path mapping.
- Test count is now **69 passing**.

### Test coverage

- `cargo test --lib` — **69/69 passing**.
- `frontend tsc --noEmit` — clean.

### Backwards compatibility

- `RemoteCommandContext::desktop(actor, actor_name, role)` constructor
  populated with `identity: None`. Existing UI callers continue to
  flow through `authorize()` unchanged.
- `http_api.rs` still passes `identity: None` for now (next session
  will switch to `auth.rs` role mapping).
- Vercel CLI (`vercel login` + `vercel deploy --prod`) remains
  supported; the One-Click flow is additive, not a replacement.

### Open work to actually close `v2.1.0`

- ~~**Convex deploy flow**~~ ✅ **Shipped** — `convex_deploy` Tauri
  command registered in `lib.rs`. Operates in the `DatabaseTab.tsx`
  *Convex One-Click Deploy* section. Operator pastes their
  `CONVEX_URL` + `CONVEX_DEPLOY_KEY` once, clicks **DEPLOY TO CONVEX**,
  and the desktop app calls `paste_convex_deploy_key` →
  `convex_push_schema` → `npx convex deploy --prod`. Schema &
  functions land on the operator's tenant-owned Convex cloud. See
  `docs/CONVEX.md`. Test:
  `plugins::convex::tests::test_convex_deploy_persists_credentials`.
- ~~**Vercel deploy flow**~~ ✅ **Shipped** — `vercel_deploy_one_click`
  Tauri command registered in `lib.rs`. Operates in the
  `GeneralTab.tsx` *Vercel (web admin)* card under the collapsible
  "⚡ One-Click Deploy (paste a token)" disclosure. Operator pastes a
  token from `vercel.com/account/tokens`, optionally a `project_id`,
  clicks **DEPLOY**, and the desktop app runs
  `paste_vercel_token` → `vercel_deploy_web` →
  `vercel deploy --prod --yes`. The `.vercel.app` production URL is
  auto-parsed from stdout and persisted to the secret store. The
  legacy `Connect Vercel` / `Deploy web` CLI flow remains as
  fallback. See `docs/VERCEL.md`. Tests:
  `plugins::vercel::tests::{parse_vercel_url_from_output_works,
  test_vercel_deploy_one_click_persists_token}`.
- **VPS self-host guide** (`docs/HOSTING.md` + `supervisor.ps1` + the
  bash runners shipping in this commit) — test on:
  - a clean Raspberry Pi 5 bookworm install (8 GB RAM)
  - an Intel NUC i3 with Ubuntu Server 24.04
  - if the operator can't pay for a cloud, repurpose an old Win10 PC
    with WSL2 and the `Self-hosted` runner.
  Each path needs an **end-to-end 10-minute validation**: blank OS →
  ARK server reachable via DNS / public IP.
- **Network & Tailscale docs** — `docs/NETWORK_SETUP.md` exists but
  doesn't yet walk a fresh user through choosing Tailscale when they
  have a CGNAT-only ISP. Should cover the wizard in the existing
  `General → Network` tab.

## [v2.1.0-alpha.2] — 2026-06-30 — OAuth-removed, CLI-bridge plugin pattern

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
- **Plugin registry** (`src-tauri/src/plugins/`): each cloud service is
  implemented as a **CLI bridge plugin**, not a custom OAuth server.
  Reason: Convex and Vercel don't expose operators OAuth serverside for
  this kind of integration. The desktop app shells out to their
  first-party CLIs which already work.
  - `convex/` — `begin_convex_link` spawns `npx convex login`; once
    the CLI writes `~/.convex/credentials.json`, we ingest the
    deploy_key + deployment_url into our own secret store. `paste_convex_deploy_key`
    is the air-gapped fallback. `convex_push_schema` runs
    `npx convex deploy --prod`.
  - `vercel/` — same architecture: `vercel login` / `vercel deploy --prod`.
    Credentials read from `~/.vercel/auth.json`.
  - `secret_store.rs` — atomic disk write, 0600 perms on Unix.
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
| Convex account | Sign up at convex.dev, create a project. Modern flow: open the app, click **Connect Convex** (Options → General → Cloud Services), which spawns `npx convex login`. CLI opens browser automatically. |
| Convex `INTEGRATIONS_SHARED_SECRET` | Set in Convex dashboard and `.env.local`. |
| Convex auth providers | Configure Google / GitHub OAuth client IDs (optional). |
| Vercel web deploy | Click **Connect Vercel** in the desktop app. CLI-bridge invokes `vercel login` then `vercel deploy --prod`. |
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

## [v2.1.0-alpha.2] — 2026-06-30 — OAuth-removed, CLI-bridge plugin pattern

Replaced the fake-OAuth endpoints from `v2.1.0-alpha` with **CLI
bridges**. Convex ships no first-party OAuth server today, only the
official `npx convex login` CLI which authors against GitHub OAuth. Our
plugin shells out to that CLI, then reads `~/.convex/credentials.json`
to capture the result. Same for Vercel: `vercel login` writes
`~/.vercel/auth.json`. This keeps us off the maintenance burden of
running an OAuth server we don't need.

### What changed since v2.1.0-alpha
- All `begin_*_oauth` / `complete_*_oauth` commands in
  `src-tauri/src/plugins/convex/` and `src-tauri/src/plugins/vercel/`
  were removed.
- New `begin_convex_link` and `begin_vercel_link` commands call out
  to the CLI; both have a `paste_*_key` / `paste_*_token` fallback for
  air-gapped setups.
- `PluginDescriptor::oauth_url` is now `None` for both plugins.

[Any earlier v2.1.0-alpha without these changes is superseded.]

[Unreleased]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/compare/v2.1.0-alpha.2...HEAD
[v2.1.0-alpha.2]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/tag/v2.1.0-alpha.2
[v2.1.0-alpha]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/tag/v2.1.0-alpha
[v2.0.0]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/tag/v2.0.0
