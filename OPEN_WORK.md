# Open Work — v2.1.0-rc.3 / v2.1.0 GA / v2.2

> Carried-over debt from `v2.1.0-rc.2`.  These are **not** in the
> rc.2 cart.  rc.2 was exclusively bug-fix-for-bug-fix; anything
> that adds surface, replaces documented behaviour, or is a feature
> belongs here.

## Status summary

| ID  | Severity  | Title                                                              | Source                                          |
|-----|-----------|--------------------------------------------------------------------|-------------------------------------------------|
| P3  | **Med-Hi** | `src-tauri/src/cli.rs` dead but documented                          | Audit 2026-07 (this branch)                     |
| P4  | Med       | `set_admin_feature_flag` silently no-ops                            | Audit 2026-07 (this branch)                     |
| P5  | Med       | DB DAO only SQLite — Convex/Supabase/Mongo/Postgres are labels-only | Audit 2026-07 (this branch)                     |
| P6  | Med       | `diagnostics::steam_validate` ignores `_repair` flag                | Audit 2026-07 (this branch)                     |
| P7  | Med       | WeChat / SSH adapters lack real wire-up                             | Audit 2026-07 (this branch)                     |
| P8  | Med       | `auth/admin.jwt` plaintext on disk, not keyring                     | Audit 2026-07 (this branch)                     |
| P9  | Low-Med   | 33 dev-mode mock strings in `services/tauri.ts`                     | Audit 2026-07 (this branch)                     |
| P10 | Low       | Plugin runtime mounts (signal-cli / russh)                         | `docs/ARCHITECTURE_AUDIT.md`                    |
| P11 | Low       | Tauri v2 capability-based permissions not enforced on adapters      | `docs/ARCHITECTURE_AUDIT.md`                    |
| P12 | Low       | Plugin gateway / OTA update signature verification                  | `docs/ARCHITECTURE_AUDIT.md`                    |
| P13 | Info      | PluginHub `enable/disable` doesn't kill live `JoinHandle`s          | `docs/ARCHITECTURE_AUDIT.md`                    |
| P14 | Info      | Tailscale `--advertise-tags` UX gap                                 | `docs/ARCHITECTURE_AUDIT.md`                    |
| P15 | Info      | Frontend `services/tauri.ts` mocks should throw dev-mock-only errors | `docs/ARCHITECTURE_AUDIT.md`                    |
| P16 | Info      | Apple Silicon notarization pipeline                                 | `docs/ARCHITECTURE_AUDIT.md`                    |
| P17 | Info      | WhatsApp outbound reply via Graph API (today inbound-only)           | Audit 2026-07 (this branch)                     |
| P18 | **HIGH**   | `backup.rs` 1346 LOC god-module (Zip-I/O + S3 SigV4 + GDrive REST + OneDrive REST + log reader + PKCE OAuth x 2 mixed) | Audit 2026-07 (this branch) |
| P19 | **HIGH**   | `router_arc` (lib.rs) calls `tauri::async_runtime::block_on` inside an async closure; each consumer dances around with `tokio::task::spawn_blocking`.  Convert the type to async-aware so we drop the dance + deadlock footgun. | Audit 2026-07 (this branch) |
| P20 | **HIGH**   | CurseForge API key persisted to `<cfg>/curseforge_api_key.txt` plaintext; the same crate has `secret_store_v2` (keyring) ready for it. **Tracked this audit, not in OPEN_WORK previously.** | Audit 2026-07 (this branch) |
| P21 | High     | `commands/integrations.rs::update_server` lives in the wrong module — it operates on the ARK server (SteamCmdInstaller::update), not on integrations. Move to `commands/server.rs`. | Audit 2026-07 (this branch) |
| P22 | Med      | `integrations/hosting.rs` (552 LOC) mixes 4 concerns. Split into `hosting/{types,cloud_init,bash_runner,provider_cli,self_hosted}.rs`. | Audit 2026-07 (this branch) |
| P23 | Med      | `integrations/command_router.rs` (571 LOC) mixes types / authorization / `run_with_receipts` pipeline. Split into `command_router/{types,policy,pipeline}.rs`. | Audit 2026-07 (this branch) |
| P24 | Med      | `Role` enum duplicated as `auth::Role` + `command_router::Role`. Replace with `pub use crate::auth::Role;` alias in command_router. | Audit 2026-07 (this branch) |
| P25 | Med      | `map_display_label` byte-identical 2x (`commands/server.rs:350`, `integrations/bridge.rs:12`), drift in 3 inline siblings. Promote to `ark::map_label()`. | Audit 2026-07 (this branch) |
| P26 | Med      | `bridge.rs:289` hardcodes `C:\\ASA\\server\\ShooterGame\\Saved\\Logs\\ShooterGame.log` for `CommandKind::Logs`. Re-route through `config.paths.server_dir`. | Audit 2026-07 (this branch) |
| P27 | Med      | `admin_only_call` + `internal_dispatch` reimplement `run_with_receipts` inline. After P19 lands (async router), reuse the canonical pipeline. | Audit 2026-07 (this branch) |
| P28 | Med      | `commands/server.rs:113-152` 5 stub metric helpers (`process_uptime_seconds` always None, etc.) shipped in production JS payload as `null` keys. Either populate via `wmic` or strip the null keys. | Audit 2026-07 (this branch) |
| P29 | Med      | `lib.rs:53-67` 5-second busy-wait poll for `auth_holder`. Replace with `tokio::sync::watch` or `Notify`. | Audit 2026-07 (this branch) |
| P30 | Med      | Once-Lock proliferation (`AUTH_HOLDER`, `admin_state_holder`, `EMITTER`, `TEST_STORE`). Migrate to `app.manage(...)` pattern (Tauri 2 idiom). | Audit 2026-07 (this branch) |
| P31 | Med      | Receipts ledger (`integrations/receipt_emit.rs:51-58`) writes `rawText` per inbound chat message without a TTL or retention policy. GDPR Article 5 violation. Add `RetentionExpiresAt` to `Receipt` + boot-time janitor task. | Audit 2026-07 (this branch) |
| P32 | Med      | `Claims.sub = "tauri-app"` literal in `auth/mod.rs:103, 146`. 7-axis identity collapses to "the desktop app"; Convex/Vercel/loopback-bearer indistinguishable in receipts. | Audit 2026-07 (this branch) |
| P33 | Med      | `sha1_smol`/`constant_time_eq`/`parse_wechat_xml_loose` (~40 LOC) belong in `integrations/wechat.rs` not `http_api.rs`. Pure boundary move. | Audit 2026-07 (this branch) |
| P34 | Low      | Spanish/English error strings mixed at backup.rs:62 etc. (cosmetic) | Audit 2026-07 (this branch) |
| P35 | Info     | Frontend `any`/`as unknown` casts concentrated in dynamic setter pattern (SettingRow, ServerSettings, etc.). Replace with `ConfigField<T>` typing. | Audit 2026-07 (this branch) |

**Closed in `fix/rc.3-cleanup` (commit ff00a4c):**

  - H1 P-dead-file-1: `integrations/http_commands.rs` (125 LOC dead). DELETED.
  - H3 P-dead-trait-1: `pub trait CommandRouter` (no impls). REMOVED.
  - H4 P-verb-judo-1: Duplicate `CommandKind -> &str` match arms across 12+ sites. UNIFIED via `CommandKind::as_str/parse_slash`.
  - M7 P-routerfn-1: `RouterFn` type alias moved to `command_router.rs`. SINGLE SOURCE.

**Closed in `fix/rc.4-layer-tighten` (commit tbd):**

  - P21: `update_server` + `check_server_version` + `read_*_buildid` helpers moved out of `commands/integrations.rs` (the integrations adapter layer doesn't operate the ARK server lifecycle) into `commands/server.rs`.  `commands/integrations.rs` shed 124 LOC; `commands/server.rs` is now self-contained.  `events.rs::emit_version` and `lib.rs::generate_handler!` re-pointed accordingly.
  - P22: still OPEN — split of `integrations/hosting.rs` (552 LOC).  Deferred to single dedicated sub-PR.
  - P23: still OPEN — split of `integrations/command_router.rs` (571 LOC).  Deferred until P19 (async-router) lands.
  - P25: NEW `ark/labels.rs` exposes canonical `map_label(&str) -> String` and `map_key_stem(&str) -> &str`.  Removed 2 byte-identical `map_display_label` definitions (commands/server.rs:350, integrations/bridge.rs:12); updated 4 inline `trim_end_matches("_WP")` callsites in launcher.rs, stub.rs, bridge.rs, server.rs to use the canonical helper.  Centralised drift.
  - P27: still OPEN — `admin_only_call` / `internal_dispatch` not yet adopting `run_with_receipts`.  Waiting on P19 (async-router) so the call site collapses cleanly.
  - P28: `commands/server.rs::server_status` and `get_server_metrics` rewritten to omit the keys they cannot honestly report (`cpu_pct`, `uptime_seconds`, `udp_conns` on linux).  Five `_pid: u32` no-impl helpers removed (`process_uptime_seconds`, `process_cpu_percent_windows` ×2 cfg twins), and one real `process_memory_mb_windows` and `read_proc_status_mem_mb` retained.  UI no longer lies about zero/null fields the impl couldn't poll.
  - P29: `lib.rs::run()` auth-init wait switched from a 50-iteration mutex+lock+sleep poll to `tokio::sync::watch::<Option<Arc<AuthState>>>`.  Auth initialiser publishes, loopback server subscribes (`auth_rx.changed().await` + `borrow().clone()`), bounded by a 5-second `tokio::time::timeout`.  Zero polling, zero lock churn.  Static `auth_initial_holder` Mutex kept for callers `admin_token` + `rotate_admin_token`; conversion is non-trivial and tracked under OPEN_WORK P30.

> **Manifest contract.** Each entry below names the file(s), the
> bad behaviour, the recommended fix shape, and a `?` for "needs
> decision before code".

---

## P3 — `src-tauri/src/cli.rs` dead code with documented behaviour

- **Severity:** Med-Hi (operator-trust risk: `docs/CLI.md` documents
  fake responses — running `ark-config status` will print
  `"Server: RUNNING (PID: 1234, Uptime: 1h)"` while no process runs)
- **Files:** `src-tauri/src/cli.rs` (254 lines, 0 references in
  crate), `docs/CLI.md` (documents the fake behaves as if real),
  `src-tauri/Cargo.toml` (single `[[bin]] = ark-asa-config`
  — no `ark-config` bin ever built)
- **Action:** Choose one of:
  - **P3a** Wire `cli.rs::execute` to `integrations::bridge::dispatch`
    + `integrations::dispatch`, build a second `[[bin]] = "ark-config"`
    in `Cargo.toml`. Costs ~0.5 d (the skeleton is already 254 lines,
    it's the body that fakes).  Pro: matches the docs verbatim.
    Con: maintenance burden to keep parity with the GUI launcher.
  - **P3b** Delete `src-tauri/src/cli.rs` (or `#[allow(dead_code)]`
    the whole module) and rewrite `docs/CLI.md` as
    "Operator-facing CLI is currently delivered through the desktop
    app (`OptionsModal → Actions → ...`).  A standalone CLI bridge is
    on the v2.2 roadmap but not shipped today."  Cost ~30 min.
    Pro: clean.  Con: removes a documented operator-facing tool path.
  - Defer neither — at minimum, `docs/CLI.md` should carry a
    "not implemented in this release" banner.
- **Decision required:** P3a vs P3b.

## P4 — `set_admin_feature_flag` silent no-op

- **Severity:** Med (next hito that consumes a feature flag will
  appear to behave correctly while storing zero state)
- **File:** `src-tauri/src/commands/integrations.rs:42-49`
- **Action:** Implement a minimal key/value registry in
  `AuthState` (file-backed, or `~/.ark-asa/feature_flags.toml`)
  with a Tauri-friendly `Map<String,bool>` round-trip.
  ~0.5 d.

## P5 — DB DAO only SQLite; rest are labels

- **Severity:** Med (operator UI lets you pick Postgres/Supabase/Mongo
  and the system says "audit DAO ready" while quietly using SQLite)
- **Files:** `src-tauri/src/integrations/database/mod.rs:128`,
  matching `database/{postgres,supabase,mongodb,convex_rest,remote}.rs`
- **Action:** Either disable the non-SQLite backends in the catalog
  (status: not yet implemented) OR implement each HTTP DAO against
  the published REST contract. Lean toward disable-by-default
  (~0.5 d) for v2.1.0 GA.

## P6 — `diagnostics::steam_validate` ignores `_repair`

- **Severity:** Med
- **File:** `src-tauri/src/ark/diagnostics.rs:313-351`
- **Action:** Have the repair branch spawn `steamcmd +app_update 2430930
  validate` (creating the same `CREATE_NO_WINDOW` spawn that
  `installer.rs` uses).  ~30 min.  Or mark explicitly "manual only"
  in the Diagnostics UI.

## P7 — WeChat / SSH adapter wire-up

- **Severity:** Med
- **Files:** `integrations/wechat.rs`, `integrations/ssh.rs`,
  `runtime_hooks.rs:103-117`
- **Action:**
  - **WeChat:** Re-implement the official Msg-Crypt verification
    path (HMAC-SHA256 over iv + ciphertext, AES-CBC decryption).
    Use the existing `parse_wechat_xml_loose` as a fallback + emit
    a `MissingCryptSignature` receipt when the connector is
    misconfigured.  ~1 d.
  - **SSH:** Add an internal russh / russh-rust listener on
    `127.0.0.1:<port>` (operator-chosen, default 2222) gated by
    the `allowed_fingerprints` list already in
    `runtime_hooks.rs:103-117`.  ~2 d.  Caveat: requires a
    keypair shipped with the desktop app (operator-side setup).

## P8 — `auth/admin.jwt` plaintext

- **Severity:** Med (master admin token is the highest-priv
  credential on the machine, sitting next to plugin API tokens
  that *are* OS-keyring-backed)
- **File:** `src-tauri/src/auth/mod.rs:60-111`
- **Action:** Migrate the `auth/admin.jwt` content to
  `secret_store_v2` (the existing OS-keyring integration).  Re-
  derive the initial token via `into_secret_format`.  ~0.5 d.

## P9 — 33 dev-mode mock strings in `services/tauri.ts`

- **Severity:** Low-Med
- **File:** `frontend/src/services/tauri.ts:59-433`
- **Action:** Replace the literal-mock branch (gated by
  `__TAURI_INTERNALS__` absence) with one that throws a
  `'dev-mock-only'` `Error`.  Today the gate is solid; future
  refactors could break it.  ~0.5 d.

## P10 — Plugin runtime mounts left for "Sesión 10+"

- **Severity:** Low (architectural promise uncovered for runtime
  daemons, not a blocker for the desktop + 8-channel bot flow
  that ships today)
- **Files:** `plugins/runtime_hooks.rs:75-126`,
  `integrations/{signal,wechat,ssh}.rs`
- **Action:**
  - `signal.rs::spawn_looper` already wires `signal-cli daemon`.
  - `ssh.rs`: see P7.
  - `wechat.rs`: see P7.
  - Bring `plugins::runtime_hooks::start()` to actually mount each
    adapter's daemon at boot when enabled; today it returns a
    parked `Future`.  ~2 d to do it cleanly + tests.

## P11 — Capability enforcement not coupled to dispatch

- **Severity:** Low (today every adapter is internal; capability
  enforcement matters when third-party plugins land)
- **Files:** `plugins/mod.rs`, `integrations/command_router.rs`
- **Action:** Introduce a `gateway.rs` that wraps `router_arc`
  with `capabilities.matches(cmd.kind)` pre-dispatch, plus a
  policy file the operator can edit.  ~1 d + tests.

## P12 — Plugin OTA update signature verification

- **Severity:** Low (this is about v2.2 marketplace dynamics)
- **Action:** Deferred until a real plugin shipping format is
  chosen.  Track in `docs/ARCHITECTURE_AUDIT.md`.

## P13 — PluginHub enable/disable doesn't abort live JoinHandles

- **Severity:** Info
- **File:** `plugins/pluginhub.rs:144-156`
- **Action:** Track the spawned `JoinHandle` inside
  `PluginEntry::handle: Mutex<Option<JoinHandle>>`; `disable_plugin`
  aborts it.  ~0.5 d.

## P14 — Tailscale `--advertise-tags` UX gap

- **Severity:** Info
- **File:** `src-tauri/src/integrations/tailscale.rs:127-134`
- **Action:** Document in `docs/HOSTING_SELFHOSTED.md` how to pass
  `tag:arkasa-prod` directly.  ~30 min.

## P15 — Frontend dev-mock error hardener

- **Severity:** Info
- **File:** `frontend/src/services/tauri.ts:59-433`
- **Action:** See P9.  Inverting the gate to "throw, do not return
  fake success" is the same work.

## P16 — Apple Silicon macOS notarization CI

- **Severity:** Info
- **Files:** missing — would need `.github/workflows/release-macos.yml`
- **Action:** Wire `tauri-apps/tauri-action@v1` with
  `signingIdentity` + `appleId` secrets, store notarization
  credentials in Keychain.  ~2 d (requires Apple Developer Program
  credentials).

## P17 — WhatsApp outbound reply

- **Severity:** Info (today the operator sends /start, the server
  starts, but no "✓ Server started" message goes back to the user.
  They have to look at the desktop UI to know if it worked.)
- **File:** `src-tauri/src/integrations/whatsapp.rs:170-200` (the
  Graph API call site already exists in `WhatsAppBot::render_outcome`)
- **Action:** When `accepted=true` is decided at the end of the
  webhook handler, spawn a `tokio::spawn` task that calls
  `https://graph.facebook.com/v18.0/<phone_number_id>/messages`
  with the bearer token from `secret_store_v2::read("whatsapp")`.
  ~0.5 d including rate-limit smoke testing.

---

## How to use this file

When picking up an item:

1. Create a branch named `fix/<short-id>-<slug>` (e.g.
   `fix/p3-purge-cli-rs`).
2. Reference this file in the commit body.
3. When the branch lands, delete the row from the table above and
   link the commit SHA in `docs/CHANGELOG.md`'s `[Unreleased]`.

When a release candidate is cut (rc.3, GA, etc.), the table at the
top of this file should typically be empty or only carry items
tagged `Info`.  Anything Med+ should not be in a GA.
