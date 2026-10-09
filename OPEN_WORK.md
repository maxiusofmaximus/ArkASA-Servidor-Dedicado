# Open Work — v2.1.0-rc.3 / v2.1.0 GA / v2.2

> Carried-over debt from `v2.1.0-rc.2`.  These are **not** in the
> rc.2 cart.  rc.2 was exclusively bug-fix-for-bug-fix; anything
> that adds surface, replaces documented behaviour, or is a feature
> belongs here.

## 2026-09 — Steam-install migration pass (closed)

Three production defects surfaced while migrating a real cluster from a
steamcmd install to the Steam-client install, plus the CLI that was
previously "documented but not shipped":

| ID  | Sev  | Title                                                                                         | Status |
|-----|------|-----------------------------------------------------------------------------------------------|--------|
| S1  | **High** | `ServerAdminPassword` in the `?`-option chain corrupts `GameUserSettings.ini` → RCON auth fails **and** in-game `enablecheats <password>` rejects the real password | **CLOSED** (`ark/launcher.rs`: passwords no longer in the URL; INI only) |
| S2  | High | `ARK_ASA_CONFIG_PATH` was never set → the multi-channel router ran every remote / loopback / chat command on `ServerConfig::default()` (wrong `server_dir`, ports, passwords) | **CLOSED** (`lib.rs::run` setup hook pins `commands::get_config_dir`) |
| S3  | Med  | Server spawned from a console front-end inherited the launcher's console (terminal flood + died with the parent shell) | **CLOSED** (`integrations/bridge.rs`: `Stdio::null()` on both spawn sites) |
| —   | —    | Standalone CLI                                                                                 | **SHIPPED** — real `ark-config` bin replaces the P3b "delete it" decision. See `docs/CLI.md`. |

Also in this pass: `bridge::ensure_ini` regenerates `Game.ini` /
`GameUserSettings.ini` before every launch, so the INI is authoritative for the
passwords on **all** start paths (UI, CLI, bots, Convex). The uncommitted WIP
`src-tauri/src/integrations/command_router/` directory — a partial P23 split
that collided with `command_router.rs` and broke every build — was parked out
of the tree.

**Still open from this pass (report only, not yet actioned):**
- ~~Single-server desktop~~ **DONE** — multi-server profiles shipped
  (`config::profiles` + `ServerSwitcher` UI + `ark-config servers` / `--server`).
- README roadmap items: ~~performance-analytics dashboard~~ **DONE** (ANALYTICS
  tab), ~~custom validator plugin system~~ **DONE**
  (`config::custom_validators` + `ark-config validate`), mobile companion app
  **still open**.
- **App signed self-updater not wired.** `tauri-plugin-updater` is neither a
  dependency nor configured in `tauri.conf.json` (no `plugins.updater` /
  `pubkey` / `endpoints`, no `bundle.createUpdaterArtifacts`). TODO.md F9
  assumed it shipped; it did not. Needs the operator's signing keys + a release
  feed to be testable. (The **server** update path *is* now verified —
  `ark-config update`.)
- New idea backlog from the deep-research pass: see
  [`docs/RESEARCH_NEW_IDEAS.md`](./docs/RESEARCH_NEW_IDEAS.md) (Prometheus
  exporter, config-drift detection, mod-health watchdog, self-healing
  guardian, status page, cluster integrity guard, …).

## Status summary

| ID  | Severity  | Title                                                              | Source                                          |
|-----|-----------|--------------------------------------------------------------------|-------------------------------------------------|
| P3  | **Med-Hi** | `src-tauri/src/cli.rs` dead but documented                          | Audit 2026-07 (this branch)                     | **CLOSED** (P3b, `0fc9933`) |
| P4  | Med       | `set_admin_feature_flag` silently no-ops                            | Audit 2026-07 (this branch)                     | **CLOSED** (`0fc9933`) |
| P5  | Med       | DB DAO only SQLite — Convex/Supabase/Mongo/Postgres are labels-only | Audit 2026-07 (this branch) | **CLOSED** (DAO retained + config persisted + UI wired) |
| P6  | Med       | `diagnostics::steam_validate` ignores `_repair` flag                | Audit 2026-07 (this branch)                     | **CLOSED** (`0fc9933`) |
| P7  | Med       | WeChat / SSH adapters lack real wire-up                             | Audit 2026-07 (this branch)                     | **CLOSED** — WeChat Msg-Crypt (verify + AES-256-CBC decrypt) implemented and wired into `/hooks/wechat`; SSH is the **sidecar** design by decision (Sesión 13), no russh. |
| P8  | Med       | `auth/admin.jwt` plaintext on disk, not keyring                     | Audit 2026-07 (this branch)                     | **CLOSED** (`fix/ga.1-secrets-keyring`) |
| P9  | Low-Med   | 33 dev-mode mock strings in `services/tauri.ts`                     | Audit 2026-07 (this branch)                     | **CLOSED** (`0fc9933`) |
| P10 | Low       | Plugin runtime mounts (signal-cli / russh)                         | `docs/ARCHITECTURE_AUDIT.md`                    | **CLOSED** (signal daemon gated on registry; boot mount report; transition watcher no longer dead code) — the russh *server* remains the sidecar approach (P7). |
| P11 | Low       | Tauri v2 capability-based permissions not enforced on adapters      | `docs/ARCHITECTURE_AUDIT.md`                    | **CLOSED** (`PluginPolicyFile` + `gate_router` wired to Telegram/Discord/Slack) |
| P12 | Low       | Plugin gateway / OTA update signature verification                  | `docs/ARCHITECTURE_AUDIT.md`                    |
| P13 | Info      | PluginHub `enable/disable` doesn't kill live `JoinHandle`s          | `docs/ARCHITECTURE_AUDIT.md`                    |
| P14 | Info      | Tailscale `--advertise-tags` UX gap                                 | `docs/ARCHITECTURE_AUDIT.md`                    |
| P15 | Info      | Frontend `services/tauri.ts` mocks should throw dev-mock-only errors | `docs/ARCHITECTURE_AUDIT.md`                    |
| P16 | Info      | Apple Silicon notarization pipeline                                 | `docs/ARCHITECTURE_AUDIT.md`                    |
| P17 | Info      | WhatsApp outbound reply via Graph API (today inbound-only)           | Audit 2026-07 (this branch)                     |
| P18 | **HIGH**   | `backup.rs` 1346 LOC god-module (Zip-I/O + S3 SigV4 + GDrive REST + OneDrive REST + log reader + PKCE OAuth x 2 mixed) | Audit 2026-07 (this branch) |
| P19 | **HIGH**   | `router_arc` (lib.rs) calls `tauri::async_runtime::block_on` inside an async closure; each consumer dances around with `tokio::task::spawn_blocking`.  Convert the type to async-aware so we drop the dance + deadlock footgun. | Audit 2026-07 (this branch) | **CLOSED** (`8f4dd23`) |
| P20 | **HIGH**   | CurseForge API key persisted to `<cfg>/curseforge_api_key.txt` plaintext; the same crate has `secret_store_v2` (keyring) ready for it. **Tracked this audit, not in OPEN_WORK previously.** | Audit 2026-07 (this branch) | **CLOSED** (`fix/ga.1-secrets-keyring`) |
| P21 | High     | `commands/integrations.rs::update_server` lives in the wrong module — it operates on the ARK server (SteamCmdInstaller::update), not on integrations. Move to `commands/server.rs`. | Audit 2026-07 (this branch) |
| P22 | Med      | `integrations/hosting.rs` (552 LOC) mixes 4 concerns. Split into `hosting/{types,cloud_init,bash_runner,provider_cli,self_hosted}.rs`. | Audit 2026-07 (this branch) |
| P23 | Med      | `integrations/command_router.rs` (571 LOC) mixes types / authorization / `run_with_receipts` pipeline. Split into `command_router/{types,policy,pipeline}.rs`. | Audit 2026-07 (this branch) |
| P24 | Med      | `Role` enum duplicated as `auth::Role` + `command_router::Role`. Replace with `pub use crate::auth::Role;` alias in command_router. | Audit 2026-07 (this branch) |
| P25 | Med      | `map_display_label` byte-identical 2x (`commands/server.rs:350`, `integrations/bridge.rs:12`), drift in 3 inline siblings. Promote to `ark::map_label()`. | Audit 2026-07 (this branch) |
| P26 | Med      | `bridge.rs:289` hardcodes `C:\\ASA\\server\\ShooterGame\\Saved\\Logs\\ShooterGame.log` for `CommandKind::Logs`. Re-route through `config.paths.server_dir`. | Audit 2026-07 (this branch) | **CLOSED** (`0fc9933`) |
| P27 | Med      | `admin_only_call` + `internal_dispatch` reimplement `run_with_receipts` inline. After P19 lands (async router), reuse the canonical pipeline. | Audit 2026-07 (this branch) |
| P28 | Med      | `commands/server.rs:113-152` 5 stub metric helpers (`process_uptime_seconds` always None, etc.) shipped in production JS payload as `null` keys. Either populate via `wmic` or strip the null keys. | Audit 2026-07 (this branch) |
| P29 | Med      | `lib.rs:53-67` 5-second busy-wait poll for `auth_holder`. Replace with `tokio::sync::watch` or `Notify`. | Audit 2026-07 (this branch) |
| P30 | Med      | Once-Lock proliferation (`AUTH_HOLDER`, `admin_state_holder`, `EMITTER`, `TEST_STORE`). Migrate to `app.manage(...)` pattern (Tauri 2 idiom). | Audit 2026-07 (this branch) |
| P31 | Med      | Receipts ledger (`integrations/receipt_emit.rs:51-58`) writes `rawText` per inbound chat message without a TTL or retention policy. GDPR Article 5 violation. Add `RetentionExpiresAt` to `Receipt` + boot-time janitor task. | Audit 2026-07 (this branch) | **CLOSED** (`8f4dd23`) |
| P32 | Med      | `Claims.sub = "tauri-app"` literal in `auth/mod.rs:103, 146`. 7-axis identity collapses to "the desktop app"; Convex/Vercel/loopback-bearer indistinguishable in receipts. | Audit 2026-07 (this branch) | **CLOSED** (`8f4dd23`) |
| P33 | Med      | `sha1_smol`/`constant_time_eq`/`parse_wechat_xml_loose` (~40 LOC) belong in `integrations/wechat.rs` not `http_api.rs`. Pure boundary move. | Audit 2026-07 (this branch) | **CLOSED** (`8f4dd23`) |
| P34 | Low      | Spanish/English error strings mixed at backup.rs:62 etc. (cosmetic) | Audit 2026-07 (this branch) |
| P35 | Info     | Frontend `any`/`as unknown` casts concentrated in dynamic setter pattern (SettingRow, ServerSettings, etc.). Replace with `ConfigField<T>` typing. | Audit 2026-07 (this branch) |
| P36 | Med      | Mobile companion app: Kotlin Multiplatform + Compose + Rust core via UniFFI. Pre-req = extract platform-agnostic `ark-core`. | `docs/MOBILE_DECISION.md` | **IN PROGRESS** - `ark-core` extraido, bridge UniFFI verificado (9/9 tests host-JVM), APK Android construido y verificado en dispositivo, capa offline Y su consumer del outbox DONE (`docs/MOBILE_DECISION.md` S3, S3b); UI Compose completa aun OPEN |

**Closed in `fix/rc.3-cleanup` (commit ff00a4c):**

  - H1 P-dead-file-1: `integrations/http_commands.rs` (125 LOC dead). DELETED.
  - H3 P-dead-trait-1: `pub trait CommandRouter` (no impls). REMOVED.
  - H4 P-verb-judo-1: Duplicate `CommandKind -> &str` match arms across 12+ sites. UNIFIED via `CommandKind::as_str/parse_slash`.
  - M7 P-routerfn-1: `RouterFn` type alias moved to `command_router.rs`. SINGLE SOURCE.

**Closed in `fix/ga.1-secrets-keyring` (commit tbd, branch on top of v2.1.0-rc.4):**

  - P8: `auth/mod.rs` rewritten so both the JWT HS256 secret and the
    derived admin token are stored as a single keyring entry `"auth_admin_v2"`
    under `secret_store_v2` (OS Credential Manager via `keyring` v4).
    Token + secret round-trip through `secret_store_v2::read` /
    `secret_store_v2::write` wrapped in `tokio::task::spawn_blocking`
    (the underlying keyring call is synchronous). On first boot, if the
    legacy plaintext pair `<cfg>/admin.jwt` + `<cfg>/admin.token` is
    still on disk, it is *lifted into the keyring first* and only then
    the plaintext files are deleted — a crash mid-migration cannot drop
    the only copy. Rotation (`AuthState::rotate`) generates a fresh
    HS256 secret + derived token (`derive_token` over the secret with
    `"v2.1-rotated-token"` domain) and writes through the keyring,
    with belt-and-braces plaintext best-effort writes also performed
    so a downgrade round-trip is recoverable. Per-test isolation is
    handled by `thread_local! TEST_KEYRING_OVERRIDE` (test-only, behind
    `cfg(test)`) + `install_test_keyring_id` / `clear_test_keyring_id` /
    `wipe` helpers; production builds compile the override out. New
    test `legacy_plaintext_is_lifted_and_deleted` exercises the full
    lift → delete → re-load round-trip. GDPR Article 32 (security of
    processing) is now satisfied for the admin credential on the
    supported desktop platforms (Windows Credential Manager, macOS
    Keychain, libsecret on Linux).
  - P20: `commands/mods.rs::read_api_key` and
    `commands/mods.rs::set_curseforge_api_key` rewritten to route the
    CurseForge API key through the same `secret_store_v2` keyring
    backend (slot `"curseforge_api_key_v2"`, field `"api_key"`).
    `read_api_key` probes the legacy plaintext
    `<config_dir>/curseforge_api_key.txt`; if present and non-empty,
    it writes the value through the keyring and then deletes the
    plaintext file before returning the key. `set_curseforge_api_key`
    rejects empty input, writes the new value to the keyring slot and
    also deletes the legacy file. Both code paths spawn_blocking on
    the synchronous `keyring` call. No new dependencies; `keyring` v4
    was already declared. 158 cargo test pass; `cargo build --release`
    emits the same binary cleanly.

**Closed in `fix/rc.4-layer-tighten` (commit tbd):**  - P21: `update_server` + `check_server_version` + `read_*_buildid` helpers moved out of `commands/integrations.rs` (the integrations adapter layer doesn't operate the ARK server lifecycle) into `commands/server.rs`.  `commands/integrations.rs` shed 124 LOC; `commands/server.rs` is now self-contained.  `events.rs::emit_version` and `lib.rs::generate_handler!` re-pointed accordingly.
  - P22: still OPEN — split of `integrations/hosting.rs` (552 LOC).  Deferred to single dedicated sub-PR.
  - P23: still OPEN — split of `integrations/command_router.rs` (571 LOC).  Deferred until P19 (async-router) lands.
  - P25: NEW `ark/labels.rs` exposes canonical `map_label(&str) -> String` and `map_key_stem(&str) -> &str`.  Removed 2 byte-identical `map_display_label` definitions (commands/server.rs:350, integrations/bridge.rs:12); updated 4 inline `trim_end_matches("_WP")` callsites in launcher.rs, stub.rs, bridge.rs, server.rs to use the canonical helper.  Centralised drift.
  - P27: still OPEN — `admin_only_call` / `internal_dispatch` not yet adopting `run_with_receipts`.  Waiting on P19 (async-router) so the call site collapses cleanly.
  - P28: `commands/server.rs::server_status` and `get_server_metrics` rewritten to omit the keys they cannot honestly report (`cpu_pct`, `uptime_seconds`, `udp_conns` on linux).  Five `_pid: u32` no-impl helpers removed (`process_uptime_seconds`, `process_cpu_percent_windows` ×2 cfg twins), and one real `process_memory_mb_windows` and `read_proc_status_mem_mb` retained.  UI no longer lies about zero/null fields the impl couldn't poll.
  - P29: `lib.rs::run()` auth-init wait switched from a 50-iteration mutex+lock+sleep poll to `tokio::sync::watch::<Option<Arc<AuthState>>>`.  Auth initialiser publishes, loopback server subscribes (`auth_rx.changed().await` + `borrow().clone()`), bounded by a 5-second `tokio::time::timeout`.  Zero polling, zero lock churn.  Static `auth_initial_holder` Mutex kept for callers `admin_token` + `rotate_admin_token`; conversion is non-trivial and tracked under OPEN_WORK P30.

**Closed in `fix/ga.1-secrets-keyring` (commit `c5fdae0` on the rc.3 consolidation stream):**

  - P3b: `pub mod cli;` removed from `src-tauri/src/lib.rs`; `src-tauri/src/cli.rs` deleted (-254 LOC); `docs/CLI.md` rewritten as the honest "NOT IMPLEMENTED in v2.1.0" banner pointing operators to the Desktop UI, loopback HTTP API, chat-bot, and Convex/Vercel channels. (commit `0fc9933`)
  - P4: `commands/integrations.rs` now persists `set_admin_feature_flag` to `<storage_dir>/admin_feature_flags.toml` (TOML parse-safe + atomic write); `false` deletes the key with a CLI error if no surrounding flags exist. Added `get_admin_feature_flag` + `admin_feature_flags_path` commands and 3 tests covering regex validation (`[A-Za-z0-9_.]`), persistence round-trip, and deletion. (commit `0fc9933`)
  - P6: `ark/diagnostics.rs::check_steam_validate` now honours the `repair` flag — on `repair && stale`, spawns `steamcmd +force_install_dir <server_dir> +login anonymous +app_update 2430930 validate +quit` returning the last 8 non-empty lines tagged `status="ok"`, `repaired=true` on success. New `ark::set_no_window_flag(cmd)` helper centralises `CREATE_NO_WINDOW (0x08000000)` across `installer.rs` and the diagnostics repair path (cfg_attr-guarded for non-Windows hosts). (commit `0fc9933`)
  - P9/P15: `frontend/src/services/tauri.ts` literal-mock branch (gated on `__TAURI_INTERNALS__` absence) now `throw new Error('dev-mock-only: no mock implementation for command ... Run pnpm run tauri:dev ...')`. Frontend can never again silently swallow a missing binding in non-`tauri dev` contexts; tsc clean. (commit `0fc9933`)
  - P19: `integrations::command_router::RouterFn` lifted to `AsyncRouterFn: Fn(AdminApiState, serde_json::Value) -> BoxFuture<'static, Result<…>>` (via `futures::future::BoxFuture`) re-exported from `command_router/mod.rs`. `AdminApiState.async_router: Arc<AsyncRouterFn>` registered at router install. `integrations::http_api` `admin_only_call` and `internal_dispatch` now do `tokio::time::timeout(Duration::from_secs(30), router(ctx, cmd).await)`; the prior `router_arc` synchronous shim is retained as a `tauri::async_runtime::handle().block_on` wrapper so `commands::server` callers don't churn yet. P23 (split of `command_router.rs`) becomes the natural follow-up now that the file's role is unambiguous. (commit `8f4dd23`)
  - P26: `integrations::bridge::CommandKind::Logs` now joins `config.paths.server_dir + "ShooterGame/Saved/Logs/ShooterGame.log"` instead of the hardcoded `C:\ASA\server\…`. (commit `0fc9933`)
  - P31: `src-tauri/src/receipts/mod.rs` adds `Receipt.retention_expires_at_ms` + `DEFAULT_RAW_TEXT_RETENTION_MS = 30 days`; `Ledger::sweep_expired()` rewrites expired `ChannelIngress` rows (removes `rawText`, sets `textLength=0`, `redactedAt`, `redactionReason="gdpr_art5_storage_limitation"`) via atomic `tmp+rename+fsync`. Boot-time invocation corrected to `ChannelIngress→Audit` staging. Three tests (`default_ttl_persists_on_write`, `sweep_removes_raw_text_and_keeps_metadata`, `sweep_is_idempotent`). New `receipts_sweep_expired` Tauri command. (commit `8f4dd23`)
  - P32: `auth::PrincipalKind { Desktop, Loopback, Convex, Vercel }` with `as_str()`; `Claims.principal: PrincipalKind` defaults to `Desktop` via `#[serde(default)]`. `sign_jwt(principal: PrincipalKind, role: Role)` rewritten to thread the principal through the token. `http_api::admin_only_call` tags Identity fields with the deserialised principal tag. New `principal_roundtrips_in_claims` test. (commit `8f4dd23`)
  - P33: `sha1_smol`, `constant_time_eq`, `wechat_handshake_sha1`, `parse_wechat_xml_loose` (CDATA-aware + `<?xml…?>` stripping) plus their 3 tests relocated from `http_api.rs` to `integrations/wechat.rs`. `http_api.rs` shed ~115 LOC; `wechat.rs` is now the single host for WeChat verification primitives. (commit `8f4dd23`)

**Closed in `feat/ark-core-extraction` (commit tbd):**

  - **P36 (partial) — `ark-core` platform-agnostic extraction.** New
    cargo crate `crates/ark-core` (version 2.1.0, edition 2021,
    `rust-version = "1.80"`; deps: `serde`, `serde_json`, `toml`,
    `async-trait`, `thiserror`, `log`, `tokio` (`fs`), `uuid`). It owns
    the whole `config` module (`schema`, `defaults`, `loader`,
    `persister`, `validator`, `profiles`, `custom_validators`, `drift`,
    `presets`) plus `error` (`Error` / `Result`). Measured before the
    move: `config` leaked only **3** `crate::error` references and
    nothing else, and `error.rs` had no internal deps, so the cut is
    cycle-free by construction.
  - `src-tauri/src/config/` and `src-tauri/src/error.rs` were deleted
    and `lib.rs` now has `pub use ark_core::{config, error};`. The
    historic `crate::config::*` / `crate::error::*` paths therefore keep
    resolving, so **all 24 consuming files and the 42 + 13 reference
    sites were left untouched**, and the `ark-config` CLI kept working
    for free. `src-tauri/Cargo.toml` gained
    `ark-core = { path = "../crates/ark-core" }`.
  - Verified: `cargo check --bins --tests` clean; `cargo test --lib`
    **168 passed** in `src-tauri` (was 186 — the 18 config/error tests
    now run in their own crate); `cargo test` in `ark-core` **18
    passed, 0 failed** (`ark-core` gained a `[dev-dependencies] tokio`
    with `macros` + `rt-multi-thread` because its tests use
    `#[tokio::test]`).
  - Still OPEN under P36: the `ark-mobile-facade` crate + UniFFI
    bindings, the Kotlin/Compose app, and the on-device verification
    flow (see `docs/MOBILE_STRATEGY.md`).

**Toolchain note (2026-09-28).** The mobile half needs a JDK: Eclipse
Temurin **17.0.20.101** was installed via `winget`
(`EclipseAdoptium.Temurin.17.JDK`). Still absent and required only for
actual device/emulator testing: the Android SDK + `adb`, plus
`rustup target add aarch64-linux-android armv7-linux-androideabi
x86_64-linux-android`, `cargo install cargo-ndk` and `uniffi-bindgen`.
`github.com/google/artemis` is **not** a JVM test harness — it is an
LLM-driven Android-automation agent (ADB + multimodal model, Python
3.12+/uv, optional MCP) aimed at driving a real device, so it is a
verification aid for later, not part of the build.

**Decision record (2026-09-28).** `docs/MOBILE_DECISION.md` supersedes the
phase list in `docs/MOBILE_STRATEGY.md` §3–5 as the execution plan. Decision:
**UniFFI 0.32.2 facade + Compose Multiplatform 1.12.1, Android-first**;
verify the bridge with **host-JVM JUnit tests before installing the Android
SDK**; adopt Artémis only as a later device-level acceptance layer. Rejected
for the record: Tauri 2 mobile (parity but weak on background/push/plugins),
Flutter/Dart (third language, non-standard bridge), hand-written JNI
(maintenance sink), Gobley (revisit at the iOS phase).

**Phase 2A delivered (2026-09-28).** `crates/ark-mobile-facade` exposes
`MobileCore` / `ProfileView` / `FacadeError` via UniFFI 0.32.2 and is verified by
`crates/ark-mobile-facade/jvmTest` (Gradle 9.8.0 wrapper checked in): **6/6 JVM
tests pass** against the real Rust `.dll`, no Android SDK needed. Workspace
tests: 168 + 18 + 4 Rust, all green. Regenerate bindings with
`scripts/generate-uniffi-bindings.ps1` (pinned `uniffi-bindgen` 0.32.2).

Three traps found and recorded in `MOBILE_DECISION.md` §3 (Phase 2A findings):
a global unpinned `uniffi-bindgen` drifts and fails with a checksum mismatch;
an error-enum field named `message` will not compile as Kotlin (collides with
`Throwable.message`) — ours is `detail`; and
`layout.projectDirectory.dir("../..")` does not normalise `..` and silently
yields `NO-SOURCE`.



**Closed in `fix/rc.4-layer-tighten` (commit tbd):**

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
