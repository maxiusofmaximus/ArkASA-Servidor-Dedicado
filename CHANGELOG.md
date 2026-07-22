# Changelog

All notable changes to this project are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/), and the project
adheres to [Semantic Versioning](https://semver.org/).

---

> ## v2.1.0-rc.2 — ship-blocker patch cut
>
> Branch `fix/tauri-invoke-acl-2.11.5` merged 7 bug-fix commits onto
> the rc.1 baseline (all derived from real operator smoke tests on
> the live `C:\ASA\` install).  None of them add features; every one
> of them makes the existing advertised behavior actually work:
>
> * **c74a810** — `capabilities/main.json` granted the 105 vendor-
>   generated `allow-*` permissions that `tauri-build` had produced
>   as `.toml` files but never referenced.  Without this, every
>   `invoke('start_server')` from React was rejected with "Command
>   X not allowed by ACL".
> * **060f918** + **a008698** — top-bar `VersionBadge` mounted,
>   `ErrorBoundary` wraps render-error-prone trees so the flat blue
>   screen stops being the failure mode, plugin catalog rewired for
>   the camelCase Rust output.
> * **de4ae3e** — SteamCMD spawn no longer pops a CMD box for every
>   probe.  `read_remote_buildid` now carries `CREATE_NO_WINDOW`;
>   the React `useServerVersion` hook coalesced into single-flight
>   + 30s min-gap so the polling fallback no longer fans steamcmd
>   out four times in a row.
> * **e243960** — `server_status`, `get_server_logs`, `get_server_metrics`
>   were stubs returning hard-coded values.  Real implementations:
>   `tasklist` probe for live PID, `backup::read_server_log` for the
>   actual `ShooterGame.log` tail, `extract_last_stat_fps` parsing
>   for fps, `tasklist` KB→MB for memory.  Three duplicate
>   `backup_config`/`list_backups`/`restore_backup` stubs deleted.
> * **4702fd2** — `Options → Diagnostics` tab wired to the existing
>   `diagnose_server_list` backend; `useDiagnostics` hook + panel
>   with RUN/REPAIR buttons + 5-language i18n.
> * **54560ca** — `/hooks/whatsapp` was answering 200 OK but doing
>   nothing (stub `_bot` + `noop` task).  Now walks every message,
>   admin-allowlist-checks via `accept_message`, parses slash
>   commands, builds a `RemoteCommand`, dispatches through the
>   same `router_arc` closure Telegram/Discord/Slack already use.
>   `internal_dispatch` and admin POSTs to `/api/v1/start|stop|restart`
>   received the same treatment (they were returning "bridged to
>   launcher in lib.rs at Hito 12").
>
> **Honest disclosure (unchanged from rc.1):** runtime hooks for
> plugin daemons (signal-cli subprocess, russh SSH listener, MQTT
> ingress for WeChat) remain parked-future.  See `OPEN_WORK.md`
> for what's still open; everything in this section can ship now.

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

### Added — Self-host on Pi / NUC / WSL2 / macOS (alpha toward v2.1)

- **New module** `src-tauri/src/integrations/local_provision.rs`
  with `LocalTargetClass { DebianPi5, DebianX86, UbuntuX86,
  Wsl2Debian, Wsl2Ubuntu, MacosArm, MacosIntel }` plus
  `build_local_plan(class, ssh_user, ssh_host, bundle_url, disk_gb)`
  that mints a complete `LocalProvisionPlan` with bundled script,
  inline one-liner, and stage-by-stage checklist.
- **`local_provision.rs` patches** the upstream `provision_script`
  output for non-Linux platforms:
  - **macOS** → swap `apt-get` for `brew install`, replace
    `/home/arkasa` with `$SERVER_HOME`, drop the systemd unit and
    `systemctl daemon-reload/enable` calls, and add a final
    `screen -dmS arkasa` fallback that streams logs to
    `/var/log/arkasa.log`.
  - **WSL2** → append a hardening tail that warns the operator
    `systemd` must be enabled in `/etc/wsl.conf` first, and fall
    back to a manual launch command if systemd won't come up on
    that Windows build.
  - **Pi 5 / NUC** → pass through (`apt-get` + `systemctl`).
- **New Tauri command** `render_local_provision_plan(class, …)`
  registered in `lib.rs` and exposed in the React UI. Returns the
  full `LocalProvisionPlan` so the panel can render the bundled
  script + inline command + per-stage checklist.
- **`HostingTab.tsx`** gains a new `<Section>` "Run on your own
  hardware" with a `<Select>` picker for the 7 hardware classes,
  fields for `ssh_user` / `ssh_host` / `disk_gb` (mirroring the
  cloud provider form), and a GENERATE LOCAL PLAN button that
  invokes the Tauri command. Result exposes 3 sub-areas: inline
  copy (operator-friendly), bundled bash (preferred for the
  archive), and a stage-by-stage checklist showing what
  stdout looks like at each step. Backed by
  `tk()` so all 5 languages (EN/ES/DE/PT/FR) have translations.
- **New tests** (5 in `local_provision::tests`):
  - `pi5_plan_uses_systemd_and_apt`
  - `ubuntu_x86_plan_no_cooling_note`
  - `wsl2_plan_includes_tail_warning`
  - `macos_drops_systemd_and_apt_switches_to_brew`
  - `plan_render_doesnt_panic_for_any_class`
  Asserts that the **macOS patch removes all `systemctl` and
  `i386`-style lines**, replaces `/home/arkasa` with `$SERVER_HOME`,
  and keeps the steamcmd / bundle-install steps intact.
- **`docs/HOSTING_SELFHOSTED.md`** — operator-facing manual with:
  - Unified flow overview
  - **Playbook 1**: Raspberry Pi 5 (Bookworm), with cooling fan
    notes, port 7777 validation, ARK-specific Pi 5 gotchas
  - **Playbook 2**: Debian / Ubuntu on Intel NUC or x86 server,
    disable desktop environment, swap size notes
  - **Playbook 3**: Windows 10/11 + WSL2, with `wsl --install`,
    systemd-in-wsl.conf setup, ports-proxy through Windows firewall
  - **Apple Silicon / Intel Mac** section flagging that ARK on
    macOS is **not officially supported by Studio Wildcard** and
    is for personal-test-rig use only
  - Operator validation checklists for each playbook.
- Test count is now **74 passing**.

### Added — Public Network & Tailscale wizard (alpha toward v2.1)

- **New module** `src-tauri/src/integrations/tailscale.rs` with
  - `TailscaleStatus { installed, up, ip, hostname, cgnat_suspect, public_ip, hint }`
  - `detect_tailscale_cli()` — `which tailscale` on Unix; checks
    `C:\Program Files\Tailscale\tailscale.exe` on Windows
  - `tailscale_install_hint()` — returns the platform download URL
    (Windows / macOS / Linux / generic)
  - `cgnat_suspect(public, tailscale)` — pure heuristic, no IO
  - `tailscale_up(auth_key, hostname, dns_label)` — async spawn
    of `tailscale up --authkey <key> --hostname <host> [--advertise-tags]`
    with re-poll of `tailscale ip -4` to surface the new IP
  - Helpers `is_tailscale_ip(ip)` (100.64/10) and
    `detect_tailscale_ip_after_up()` (re-poll helper)
- **New Tauri commands** in `src-tauri/src/lib.rs`:
  - `tailscale_installed()` — boolean
  - `tailscale_download_url()` — string URL for the operator's platform
  - `tailscale_status_combined()` — combined IP probe + Tailscale
    probe + CGNAT heuristic in one call
  - `tailscale_setup(auth_key, hostname, publicly_dns_label)` —
    one-shot `tailscale up` invocation
- **New UI**: `GeneralTab.tsx::TailscaleWizard()` — a sub-component
  with a 4-row status table + amber-bordered hint box when CGNAT is
  suspected / Tailscale missing + auth-key + hostname `<input>`
  + SET UP TAILSCALE button + green success box showing the new
  `100.x.x.x` IP (select-all for sharing). Backed by `tk()` with
  English translations added; Spanish translations added; DE/PT/FR
  fall back to English defaults (consistent with the existing
  pattern for the rest of GeneralTab).
- **Existing `detect_ips` is untouched** — the new
  `tailscale_status_combined` reuses its helpers
  (`detect_public_ip`, `detect_tailscale_ip`, `is_tailscale_range`)
  so we don't fork any probe logic.
- **`docs/NETWORK_TAILSCALE.md`** — operator-facing wizard recipe:
  what CGNAT-suspect means, auth-key minting at
  <https://login.tailscale.com/admin/settings/keys>, the friend flow
  (`<100.x.x.x>:7777/UDP`), why we deliberately don't OAuth,
  iPhone/Android Tailscale connection path, common pitfalls (auth-key
  rejected by ACL, captive portal, missing `--advertise-tags`).
- **New tests** (`integrations::tailscale::tests`, **+6**):
  - `cgnat_suspect_when_no_public_ip`
  - `not_suspect_when_public_ip_present`
  - `is_tailscale_ip_100_range`
  - `install_hint_returns_some_url`
  - `tailscale_up_rejects_empty_inputs`
  - `status_default_is_empty`
- Test count is now **80 passing**.

### Sesión 6 / P5 — Deuda técnica pre-tag

- `cargo build` ahora compila con **0 warnings** (was 5 pre-existentes).
- `src-tauri/src/auth/mod.rs`: new `validate_with_claims()` returns full
  identity claims; existing `validate()` is now a thin wrapper. The
  previously-unused `claims` field is now read by the new method.
- `src-tauri/src/integrations/http_api.rs`: admin endpoint now binds a
  real 7-axis `Identity` derived from validated claims (was
  `identity: None`). Receipts ledger can correlate admin ingress to the
  operator who proved bearer / JWT.
- `src-tauri/src/integrations/slack.rs`: `handle_inbound` takes new
  `envelope_id` param threaded through from the Socket-Mode WS frame,
  instead of a `trace_id` placeholder. `queue_enqueued` now emits the
  real envelope id.
- `src-tauri/src/integrations/discord.rs`, `telegram.rs`,
  `http_commands.rs`: clean up the remaining dead-code warnings
  (`resume_url` removed / `User.id` marked allow-dead / `map_index` marked
  allow-dead + stub handlers log `{body:?}`).
- Sesión 6 prelude closes P5 (deuda) before expanding product surface.

### Sesión 6 / P1 — Plugin Hub core (dyn add/remove)

- `src-tauri/src/plugins/mod.rs`: `PluginRegistry` rewritten from
  hardcoded `Option<ConvexPlugin>` / `Option<VercelPlugin>` to
  `Vec<PluginEntry>` + `BTreeSet<String> enabled`. New trait `AnyPlugin`
  (object-safe adapter over the existing associated-function `Plugin`
  trait) so heterogeneous plugin types can sit in the same catalog.
  Methods: `register()`, `enable()`, `disable()`, `enable_id_no_start()`,
  `catalog_iter()`, `enabled_ids()`.
- `src-tauri/src/plugins/registry.rs` (new): registry.toml persistence
  under `~/.ark-asa/plugins/registry.toml`. Functions `read()`,
  `write()`, `enable_id()`, `disable_id()`. 2 tests.
- `src-tauri/src/plugins/pluginhub.rs` (new): 4 Tauri commands —
  `list_plugin_catalog`, `enable_plugin`, `disable_plugin`,
  `plugin_registry_snapshot`. Shared `OnceLock<Mutex<PluginRegistry>>`
  for state. 3 tests.

### Sesión 6 / P2 — Connection plugins catalog

- `src-tauri/src/plugins/connection.rs` (new): 7 VPS providers as a
  discoverable catalog (`Oracle Always-Free`, `Hetzner`, `DigitalOcean`,
  `Self-hosted`, `AWS EC2`, `Azure VM`, `GCP Compute`) with
  `description`, `free_tier`, `requires_cli`, `requires_credentials`,
  `docs_url`. 6 tests.
- 2 Tauri commands: `list_connection_plugins`, `get_connection_plugin`.
- 100% backwards compatible — existing `HostProvider` enum and the
  `provision_script` / `render_provider_run_script` paths are
  untouched.

### Sesión 6 / P3 — AI model plugins (8 adapters)

- `src-tauri/src/plugins/model.rs` (new): 8 OpenAI-API-compatible
  adapters as a discoverable catalog: OpenAI, Cerebras, NVIDIA NIM,
  llama.cpp, Ollama, vLLM, LM Studio, Custom. Each entry pins
  `defaultBaseUrl`, `defaultModel`, `requires_api_key`, `is_local`,
  `install_hint`, `docs_url`. 6 tests.
- 2 Tauri commands: `list_model_plugins`, `get_model_plugin`.
- Cero cambios al runtime AI — `integrations::ai::AiClient` ya habla
  OpenAI Chat Completions con cualquier endpoint.

### Sesión 6 / P4 — UI Plugin Hub tab

- `frontend/src/components/options/PluginsTab.tsx` (new): three
  sections in one tab — Plugin Hub (toggleable Convex/Vercel with
  missing-secrets amber pills), Connection Providers (declarative
  metadata list), AI Model Plugins (8 adapters with LOCAL/API KEY
  pills). Toggle is `enable_plugin` / `disable_plugin` Tauri calls —
  operator changes apply without restarting the app.
- `frontend/src/components/OptionsModal.tsx`: new `plugins` Tab.
- `frontend/src/i18n/translations.ts`: 12 new i18n keys (EN full,
  ES full, DE/PT/FR partial with English default fallback).

### Sesión 6 / P6 — Architecture audit

- `docs/ARCHITECTURE_AUDIT.md` (new): honest gap analysis vs
  OpenClaw / Hermes Agent / Agent Harness Core / Mastra / OpenSAGE /
  Bullwork. Identifies:
  * 0 critical risks today.
  * 3 high-risk gaps for v2.1.1+ (capability enforcement, slack
    parser mismatch, telegram per-user message limit).
  * 3 medium risks (CGNAT false positives, plugin-abort lifecycle,
    WSL2 portproxy firewall interactions).
  * 3 low risks (AI prompt injection, hardcoded Tailscale
    --advertise-tags, Apple Silicon notarization latency).
  * 6-item roadmap (P1: PluginGateway, P2: OTLP, P3: provider-schema
    TOML, P4: per-user rate-limit, P5: WASM runtime, P6: macOS
    notarization CI).
  Recommend: v2.1.0 can ship as RC today; v2.1.1+ takes the open
  gaps. Conclusión: **sin urgencia de feature-parity con OpenClaw**

### Sesión 7 — WhatsApp Business Cloud

- `src-tauri/src/integrations/whatsapp.rs` (new): WhatsAppConfig
  with phone_number_id / business_id / webhook_secret / api_token /
  admin_e164s. Full Meta Graph v18.0 webhook payload structs.
  Constant-time HMAC-SHA256 verify, send_text_reply via
  graph.facebook.com/v18.0/{phone_id}/messages. slash command
  parser, allowlist, render_outcome. 9 tests.
- `src-tauri/src/plugins/whatsapp_bridge.rs` (new): 3 Tauri
  commands mirroring Convex/Vercel pattern.
- `src-tauri/src/plugins/mod.rs`: register_default_plugins now
  adds WhatsAppPlugin; AnyPlugin impl.
- The actual webhook route mount happens at `lib::run()` in
  v2.1.x — for now the PluginHub shows the secret-store check.

### Sesión 8 — Signal, WeChat, SSH, REST

- 4 new adapters under `src-tauri/src/integrations/`:
  * `signal.rs`   — signal-cli JSON daemon inbound (5 tests)
  * `wechat.rs`   — WeCom XML webhook inbound (5 tests)
  * `ssh.rs`      — russh-style public-key dispatcher (5 tests)
  * `rest.rs`     — Bearer-auth POST /api/v1/* ingest (6 tests)
  Each ships with a parse_action / handlers / DESCRIPTOR /
  Plugin impl — same factory as Convex / Vercel / WhatsApp.
- `src-tauri/src/plugins/extra_bridges.rs` (new): 8 commands —
  paste_*_credentials + *_status for the four plugins.
- `src-tauri/src/plugins/mod.rs`: register_default_plugins now
  appends all 5 new entries (whatsapp/signal/wechat/ssh/rest).
- The runtime mount for Signal / WeChat / SSH (subprocess spawn,
  webhook axum route, russh server) is left to the operator's
  `lib::run()` glue. Today's repository only ships the
  parsing/registry/UI surface; the runtime loop is documented in
  `docs/OPEN_WORK.txt` (S10+).

### Sesión 9 — Plugin runtime hooks (honest disclosure)

- **DISCLOSURE**: Plugins enabled through the PluginHub today show
  ● connected on the UI, BUT their `start()` returns a parked
  future. The full runtime mount (signal-cli subprocess, axum
  WhatsApp route, russh SSH server, Convex periodic push,
  Vercel deploy-on-demand, Tailscale wizard) is left to operator-
  side wiring. The PluginHub does NOT spawn those processes yet —
  it only stores credentials and surfaces read-state.
- `src-tauri/src/plugins/runtime_hooks.rs` (new): 5-state machine
  `Disabled / PendingCredentials / Running / EventDriven /
  Failed` keyed off registry + secret-store. The new
  `runtime_status()` Tauri command surfaces the truth to the UI:
  * Convex  → event_driven (we trigger upserts from the UI)
  * Vercel  → event_driven
  * REST    → event_driven (the existing /api/v1/* endpoints
                        already handle this)
  * WhatsApp/Signal/WeChat/SSH → pending_credentials until the
                                  operator pastes the secrets, then
                                  running once the registry AND
                                  secret-store agree.
- The runtime mount itself (when each Plugin's `start()` actually
  spawns its subprocess / axum route / russh server) is the
  truthful session-10+ work; the current behaviour is **operator-
  honest**: no fake "running" badge for plugins that aren't
  actually wired.
- 4 tests for runtime_hooks: state matrix never panics on unknown
  ids; secret-store empty path returns PendingCredentials.

### Sesión 10 — WhatsApp + WeChat webhooks mounted + boot-time probes

- `src-tauri/src/integrations/http_api.rs` gains two POST axum
  routes on the existing `127.0.0.1:8765` loopback:
  * `POST /hooks/whatsapp` — verifies `X-Hub-Signature-256`
    against the operator's `webhook_secret`, drops the
    payload through `WhatsAppBot::accept_message` allowlist,
    surfaces accepted-count to Meta.
  * `POST /hooks/wechat` — accepts application/xml or
    JSON-already-parsed bodies (`x-wecom-token` header gate),
    runs `WeChatBot::accept_message` filter. Includes a
    hand-rolled pull-XML helper (`parse_wechat_xml_loose`) for
    the common case without pulling in a full XML crate.
- Both routes silently return `{ "status": "noop, plugin not
  configured" }` when the operator hasn't yet pasted secrets —
  Meta won't keep retrying pointlessly.
- `src-tauri/src/lib.rs::run()` gains two boot-time probes:
  * Convex periodic-push placeholder (logs presence/absence of
    `CONVEX_URL` env or secret-store entry). The full
    `convex_push::spawn_publisher` task is not yet wired to run
    autonomously; the operator can still trigger pushes via the
    existing `convex_push_schema` Tauri command.
  * `tailscale` CLI detection (one-shot log line on whether the
    binary is on PATH). The wizard remains UI-driven.
- `docs/OPEN_WORK.txt` (new) enumerates everything between
  `v2.1.0-alpha.3` and a future `v2.1.0` GA: signal-cli daemon
  spawn, russh SSH server bind, WeChat handshake URL verification,
  Convex periodic interval, capability enforcement, WeChat CDATA
  parse, pre-flight toggle gating.

### Sensación 11 — docs/INDEX.md

- New `docs/INDEX.md` (73 lines) replaces the previous 342-line
  block. Operator-facing scannable TOC with three sections:
  * Quick links (12 docs, one line each).
  * Plugin-level status legend (✅ real-loop wired, 🟡 parked).
  * Version timeline table ending at `v2.1.0-alpha.3` and the next
    honest step toward GA.
- No code touched; nothing else moves.

### Sesión 12 — Runtime hooks wire-up (real, partial)

The S9 honest disclosure ("plugins return parked futures")
is partially retired on three plugins:

  - **Convex** — the existing `holder.lock().await.clone()` arm
    spawned at α.2 already calls `integrations::convex_push::
    spawn_publisher`. No additional wiring needed. The redundant
    placeholder I added at S10 is removed.

  - **WeChat Work handshake** — `GET /hooks/wechat` now
    responds to the WeCom URL verification
    (`?msg_signature=...&timestamp=...&nonce=...&echostr=...`).
    WeChat mandates SHA-1 for this handshake (deliberately
    mandated by the protocol, not a security choice). The
    `sha1_smol` crate is added to Cargo.toml to do the
    lex-sorted token+timestamp+nonce SHA-1 hex. Constant-time
    compare rejects forged signatures.

  - **Signal CLI** — `integrations::signal::maybe_spawn_subprocess`
    spawns `signal-cli daemon --json -u <phone>` if the binary is
    discoverable on PATH and the operator has configured
    `phone_e164`. `lib::run()` calls it at boot. Falls back
    gracefully with an INFO log if signal-cli isn't installed.
    stdout JSON-lines are parsed via the existing
    `SignalJsonLine` shape.

- Tests added:
  + 2 in `integrations::signal::tests` for `maybe_spawn_subprocess`
    (no-phone returns None, find_signal_cli doesn't panic on
    clean PATH).
  + 2 in `integrations::http_api::tests` for `constant_time_eq`
    (the helper backing the WeChat handshake).
- Tests bumped from 131 → **135** passing.
- Cargo.toml: `sha1_smol = "1"` (zero-dependency SHA-1 for the
  WeChat handshake — SHA-1 is mandated by WeCom's URL
  verification protocol; SHA-3 would break compatibility).
- **SSH server (`russh`) and WeChat CDATA remain parked.**
  See `docs/OPEN_WORK.txt`. Pulling `russh` crate is out of scope
  for Sesión 12 (would touch many transitive deps); we document
  the sidecar option (`sshd` listening on :2222 with operator's
  authorized_keys) in `docs/OPEN_WORK.txt` §1.1.

### Test coverage

- `cargo test --lib` — **135/135 passing** (was 131; +2 signal +
  +2 constant_time_eq).
- `cargo build` — 0 errors, **0 warnings**.
- `frontend tsc --noEmit` — clean.
- SHA-1 soundness: WeChat Work mandates SHA-1 in its URL
  handshake. `corporate_secret` is a 256+ bit shared key,
  runs once per operator setup, no reproducible artefacts
  leak into the receipts ledger. SHA-3 is unsuitable here.

### Sesión 13 — SSH sidecar approach + cleanup

The OPEN_WORK.txt §1.1 SSH status item is now resolved via
the **sidecar approach** rather than pulling the `russh`
crate. The desktop app probes TCP port 2222 via
`TcpStream::connect_timeout` instead of pulling its own
sshd library, which cuts ~50 transitive deps from the
lockfile and keeps the audit surface narrow.

Rust
  - src-tauri/src/plugins/runtime_hooks.rs
    + sshd_sidecar_alive(listen_port) -> bool: TCP probe
      against 127.0.0.1:<port> with 300ms timeout.
    + runtime_state_for("ssh") now distinguishes:
        * Secrects missing → PendingCredentials
        * Secrects present + sshd reachable → Running
        * Secrects present + sshd unreachable → PendingCredentials
          ("operator pasted credentials but didn't start sshd yet")
    + 2 new tests covering port=0 always-false and unbound-port
      always-false.

Docs
  - docs/SSH_SETUP.md (new): 130-line operator guide
    covering sshd `-p 2222` setup on Linux/Windows/macOS,
    SHA-256 fingerprint retrieval, sidecar rationale
    ("why not russh"), failure-mode table, promotion
    checklist.
  - docs/OPEN_WORK.txt §1.1 marked RESOLVED (sidecar note).
  - docs/INDEX.md entries added for SSH_SETUP.md.

Tests
  - cargo test --lib → 137/137 passing (was 135; +2 tests).
  - cargo build      → 0 errors, 0 warnings.
  - frontend tsc --noEmit clean.

Honest assessment
  The sidecar approach is *operator-friendlier* than an
  in-process russh: it repackages an existing sshd that the
  operator already maintains. If an operator runs their own
  hardened OpenSSH policy set, the desktop app inherits that
  policy automatically. The cost is that the runtime_state
  is best-effort — `ssh_status` returns 'running' iff a
  TCP connect succeeds within 300 ms.

Tag plan
  No new tag. The sshd sidecar approach moves OPEN_WORK.txt
  §1.1 from "pending" to "resolved". The remaining items in
  OPEN_WORK.txt (capability enforcement, WeChat CDATA, etc)
  continue to gate v2.1.0 GA in line with the rc.1 → GA
  promotion checklist already documented.



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
- ~~**Self-host on Pi / NUC / WSL2 / macOS**~~ ✅ **Shipped (code
  side)** — `render_local_provision_plan` Tauri command registered
  in `lib.rs`. `HostingTab.tsx` "Run on your own hardware" lets the
  operator pick one of 7 hardware classes (Pi 5 / Debian NUC /
  Ubuntu NUC / WSL2 Debian / WSL2 Ubuntu / Apple Silicon / Intel
  Mac), paste the same backup-bundle URL they'd use for a cloud
  VPS, and generate a platform-tailored bash + inline one-liner +
  stage-by-stage checklist. The Rust module
  `src-tauri/src/integrations/local_provision.rs` contains the
  patches so that the macOS variant swaps `apt-get` for `brew
  install`, replaces `/home/arkasa` with `$SERVER_HOME`, drops the
  systemd unit and uses `screen -dmS arkasa` instead; the WSL2
  variant appends a hardening tail warning the operator to enable
  systemd in `/etc/wsl.conf` first. Tests:
  `integrations::local_provision::tests::{pi5_plan_uses_systemd_and_apt,
  ubuntu_x86_plan_no_cooling_note, wsl2_plan_includes_tail_warning,
  macos_drops_systemd_and_apt_switches_to_brew,
  plan_render_doesnt_panic_for_any_class}`.
  See `docs/HOSTING_SELFHOSTED.md` — three operator playbooks:
  Pi 5 (Bookworm arm64), Debian/Ubuntu on Intel NUC, WSL2 on
  Windows 10/11, with hardware-validation checklists. **For v2.1.0
  closure** the operator still needs to **physically validate** the
  three playbooks on real hardware (Pi 5, NUC, WSL2) — once those
  pass, this becomes fully shipped.
- ~~**Network & Tailscale wizard**~~ ✅ **Shipped** —
  `tailscale_status_combined` + `tailscale_setup` Tauri
  commands registered in `lib.rs`. Operates in the
  `GeneralTab.tsx` *Public network & Tailscale* section
  under the new `TailscaleWizard` sub-component. The wizard
  detects the public IPv4 + Tailscale availability + CGNAT
  heuristic on mount, flags CGNAT-suspected on a 4-row
  status table, and offers the operator an inline form
  (auth-key + hostname inputs) that calls
  `tailscale up --authkey <key> --hostname <host>`. The new
  `100.x.x.x` IP is surfaced as a select-all success box.
  Tests:
  `integrations::tailscale::tests::{cgnat_suspect_when_no_public_ip,
  not_suspect_when_public_ip_present,
  is_tailscale_ip_100_range,
  install_hint_returns_some_url,
  tailscale_up_rejects_empty_inputs,
  status_default_is_empty}`.

## [v2.1.0-rc.2] — 2026-07-22 — ship-blocker patch cut

Seven bug-fix commits rolled up from `fix/tauri-invoke-acl-2.11.5`.
Every item below makes an **already-advertised behaviour actually
work**; none of them adds a new feature.  See the preamble at the
top of this file for the commit-by-commit digest; each item links
to the in-house commit hash.

### Fixed

- **Capability ACL shipped `allow-*` grants.** Tauri 2.11 build.rs
  was emitting `permissions/autogenerated/<cmd>.toml` files for all
  105 `#[tauri::command]` handlers, but `capabilities/main.json`
  never listed them.  Every `invoke('start_server')` from React was
  rejected with `not allowed by ACL`.  The fix materialises all
  `allow-*` identifiers + keeps the 12 baseline grants, sorted.
  (c74a810)

- **SteamCMD no longer pops a CMD box per probe.** `read_remote_buildid`
  now carries the canonical Windows `CREATE_NO_WINDOW (0x08000000)`
  flag (parity with `installer.rs:46`).  The React `useServerVersion`
  hook added a single-flight lock + 30s min-gap debounce so the
  polling fallback can no longer fork four steamcmd subprocesses
  in a row.  Backend-driven `server://version` events bypass the
  cache.  (de4ae3e)

- **`UpdateNowCard` no longer fires `update_server` twice per click.**
  Was calling `invoke('update_server', ...)` independently of the
  hook, with no busy guard.  Now delegates to `version.runUpdate()`
  and disables both REFRESH and UPDATE NOW while a probe or update
  is in flight.  (de4ae3e)

- **`server_status`, `get_server_logs`, `get_server_metrics` now
  return real data.** Was silently returning hard-coded values
  (`{running:false}`, `["[INFO] Server started"]`, `{cpu:0,...}`).
  Real implementations: `tasklist` probe for live PID, `backup::
  read_server_log` for the per-map `ShooterGame.log` tail,
  `extract_last_stat_fps` scraping the latest `Stat FPS=` line,
  `tasklist` KB→MB for memory.  Three duplicate
  `backup_config`/`list_backups`/`restore_backup` stubs in
  `commands/server.rs` deleted; the real `backup::*` family is
  the source of truth.  (e243960)

- **`Options → Diagnostics` tab is now wired to the real backend.**
  The `diagnose_server_list` Tauri command and the per-check
  repair logic for `culture_en` + `eos_cert` lived in
  `ark/diagnostics.rs` for months without a UI hook.  This adds:
  * `useDiagnostics` hook with 5-language i18n
  * `DiagnosticsTab` component with status-coded checks and
    RUN/REPAIR buttons
  * `tab_diagnostics` / `section_diagnostics` / `diagnostics_*`
    keys across EN/ES/DE/PT/FR.  (4702fd2)
  NOTE: `steam_validate` repair remains intentionally manual
  (`repair=true` is ignored) — see OPEN_WORK.md.

- **WhatsApp webhook now actually dispatches commands.** `/hooks/whatsapp`
  was answering 200 OK `accepted` but doing nothing — `_bot` was
  constructed only as `let _bot = ...`, dispatch went to a
  `spawn_blocking(|| noop)`.  Now walks every inbound message,
  admin-allowlist-checks via `accept_message`, parses slash
  commands (`/start /stop /restart /status /logs /ip`), builds
  a `RemoteCommand`, and pushes it through the same `router_arc`
  closure Telegram / Discord / Slack already use.  Reception
  response now reports `{messages_dispatched, messages_rejected}`
  for observability.  `/api/v1/internal/dispatch` and admin POSTs
  to `/api/v1/{start,stop,restart}` received the same treatment
  — they were returning the literal string `"bridged to launcher
  in lib.rs at Hito 12"`.  (54560ca)

### Changed

- **`AdminApiState` now carries the same `router_arc` closure as the
  channel adapters.**  `lib.rs` reordered so `router_fn` is built
  first and `Arc::clone()`'d into both `AdminApiState::new(...)`
  and the bot spawners, instead of being constructed twice.  This
  keeps the `<hook> → dispatcher → launcher` path uniform across
  every adapter surface.  (54560ca)

### UX restoration (no behavior change to backend)

- **`Options → General` reorders so "Mundo & Inventario" is the
  first section right after Language.**  The Apilable (stack-size)
  multiplier `X1..X10` and the per-resource override grid (15
  presets: Raw Meat, Raw Prime Meat, Stone, Wood, Thatch, Metal,
  Obsidian, Crystal, Flint, Berries, ...) were always present in
  `MultipliersConfig` + `stack_overrides`, but were the third
  block of a long Options modal.  Surfacing them first eliminates
  the "where did the stack controls go?" report.  (de4ae3e)

- **`App.tsx` mounts `<VersionBadge>` + `<ErrorBoundary>`.**
  VersionBadge shows local→latest buildid with `🟢 current` /
  `🔴 outdated` colour coding and listens to `server://version`
  events for live updates.  ErrorBoundary replaces the flat
  blue surface when a render-time exception blows up the tree.
  (060f918, a008698)

[v2.1.0-rc.2]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/tag/v2.1.0-rc.2
[v2.1.0-rc.1]: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/tag/v2.1.0-rc.1

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
