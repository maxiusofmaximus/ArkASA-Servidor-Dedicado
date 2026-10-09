# Mobile companion app — strategy & stack decision

> Research pass (2026-09), three parallel investigations: (1) what makes a
> *professional* mobile app, (2) stack comparison for a high-performance,
> maintainable companion, (3) how to share this project's Rust core with
> Android/iOS. This file is the durable synthesis + recommendation.
>
> **Constraint from the product owner:** parity, efficiency, speed and
> performance are *examples* — the app must meet the full professional bar
> (maintainability, usability, offline-first, room for many features, security,
> i18n, accessibility, testing, distribution). React Native is explicitly
> rejected (bridge/JS perf).

---

## 0. TL;DR

> **Execution plan lives in [`MOBILE_DECISION.md`](./MOBILE_DECISION.md)**, which
> also records the verified tool versions, the options that were rejected and
> why, and the go/no-go gates. This document stays as the research and
> quality-model reference.

1. **Architecture comes before the UI toolkit.** The single biggest decision is
   *embedded core* (run the Rust core **on the phone** → true offline, no
   desktop needed) vs *thin client* (phone → desktop/host HTTP API). The
   requested features (offline, notifications, background, config editing)
   point at **embedded core**, with thin-client as a fallback.
2. **Extract a platform-agnostic `core` crate first** (no Tauri/desktop
   assumptions). This is required for *every* option and is the real work.
   ✅ **DONE (2026-09-28)** — see §7.
3. **Recommended stack: Kotlin Multiplatform + Compose Multiplatform, Rust core
   via UniFFI.** It is the only option that is simultaneously production-grade
   on mobile today, native-class in performance, excellent at offline /
   background / push / secure-storage, and able to reuse our Rust core via the
   canonical (Mozilla-proven) path.
4. **Runner-up: Tauri 2 mobile** — unbeatable *parity* (same Rust core **and**
   the existing React UI) but its mobile gaps are exactly our requirements
   (background Rust execution, push, plugin coverage, webview perf). Good for a
   fast sprint, weak as a long-term platform.
5. Non-negotiable, ARK-specific: **classic Source RCON is unauthenticated
   plaintext.** The mobile app must reach the server over a tunnel
   (Tailscale/WireGuard) or a TLS-terminating agent — never raw RCON over the
   internet.

---

## 1. What "professional mobile app" means (criteria we will be held to)

Condensed from the quality-model research. **M** = must ship, **S** = should,
**L** = later. Targets are verifiable.

### Architecture & maintainability (M)
- [ ] Layers: UI / domain / data; data layer is the single source of truth (SSOT); UI never touches DB/network/socket directly.
- [ ] Unidirectional data flow; UI state as one immutable state object.
- [ ] MVVM or MVI, documented. MVI pays off for many async streams (status, log tail, players).
- [ ] State holders are lifecycle-agnostic (no `Context`/`Activity` in domain).
- [ ] **One shared contract schema** (JSON Schema / protobuf / OpenAPI) → generated clients for Rust, TS and mobile. Never hand-copy DTOs across three languages.
- [ ] Versioned remote interfaces (`/v1/`) + backward-compatible additive changes + a published deprecation window (≥180 d beta, ~1 y stable).

### Offline-first (M)
- [ ] Local DB is the SSOT, not a cache (SQLDelight/Room on Android, GRDB/SwiftData on iOS).
- [ ] **Durable outbox** for writes + exponential backoff + **idempotency keys** (so a retried RCON command can't apply twice). Persist across process death.
- [ ] Conflict policy **per data type** (never wall-clock-only LWW on critical admin data); destructive admin actions (ban/whitelist/shutdown) → server-authoritative or manual.
- [ ] Optimistic UI + rollback with a human reason; **staleness indicator** (last-synced + offline), never a blocking spinner.
- [ ] Reachability = "is the *server* reachable" (RCON/agent heartbeat), not just link state.
- [ ] Respect platform background limits: Android WorkManager; iOS BGTaskScheduler is a **hint, not a guarantee** — design to work without background sync.
- [ ] Data budget: heavy work Wi-Fi/unmetered only, with a user toggle.

### Performance & efficiency (M)
- [ ] Cold start **TTID < 2 s** mid-tier (aim < 500 ms for a lean admin app); no I/O or SDK init on the launch path.
- [ ] Frame budget: 16.7 ms @60 Hz / 11 ms @90 / 8.3 ms @120; track janky-frame % and frozen frames separately.
- [ ] Virtualized lists; images sized + cached.
- [ ] Download size target **< 50 MB** (every ~6 MB costs ~1 % install rate); App Bundle/R8/WebP/locale stripping; iOS thinning.
- [ ] No polling loops; push or long-lived connection with backoff; gzip/brotli/ETag.
- [ ] CI performance gates via **Macrobenchmark** (Android) + **MetricKit/Instruments** (iOS), compared on a rolling window to kill flake.

### Usability & accessibility (M)
- [ ] Material 3 / Apple HIG conformance; native back/gesture handling.
- [ ] Touch targets **≥48 dp / ≥44 pt**; screen-reader labels; Dynamic Type at 200 %; WCAG AA contrast (4.5:1 / 3:1); never color alone.
- [ ] Zero hardcoded strings; logical start/end (RTL-safe); locale-aware dates/numbers; pseudo-locale tested early.
- [ ] Designed empty / loading / error / offline / partial states; dark mode; first-run onboarding (ideally a QR/deep-link from the desktop app).
- [ ] Human, actionable error copy (never raw stack traces).

### Security & privacy (M)
- [ ] Secrets (RCON password, admin tokens) in **iOS Keychain / Android Keystore**, preferably hardware-backed + biometric-gated; never in prefs/logs/crash reports.
- [ ] TLS everywhere + certificate pinning for the agent API; no cleartext traffic. **RCON over tunnel/TLS-agent only.**
- [ ] Short-lived tokens + refresh rotation; revoke on logout.
- [ ] Documented **admin threat model**; least-privilege per-server tokens; audit log.
- [ ] Data map + consent before analytics; accurate store privacy labels; ATT on iOS.

### Reliability & ops (M)
- [ ] Crash reporting with symbolication; **crash-free sessions ≥ 99.5 %** (aim 99.9 %); track ANRs/hangs, not just crashes; stay under Play's 1.09 % crash / 0.47 % ANR thresholds.
- [ ] **Remote kill-switch** (disable a broken feature without a store update) + local cache with defensive defaults.
- [ ] **Staged rollout** (1 %→100 %) with monitoring; TestFlight / Play tracks.
- [ ] OTA only for web content/config/assets — Apple 2.5.2 / 3.3.1 forbid shipping new native code post-review.

### QA, distribution, team (M/S)
- [ ] Test pyramid; regression test per bug fix; a11y + localization QA on real builds; device matrix + farm.
- [ ] CI: lint + unit on PR; instrumentation + perf nightly; reproducible release builds; fastlane.
- [ ] minSdk/deployment-target policy (Android target API 36 by 31 Aug 2026); signing/notarization; forced-upgrade strategy via protocol version check.
- [ ] **SBOM + license compliance** (we are OSS, MIT — copyleft deps in a store build are a problem); `SECURITY.md`; ADRs.

**Top failure modes to avoid** (real anecdotes): WebView-wrapper rejection (Apple 4.2), secrets in SharedPreferences, offline writes without an outbox, wall-clock LWW, expecting timely iOS background sync, dumping init into `onCreate`, 90 MB APK, feature-flag debt, "Wi-Fi is up but the server is down", testing on one flagship, a11y as an afterthought, aggregate-only stability dashboards, no kill-switch.

---

## 2. Stack comparison

Scores 1–5 (higher better; **Risk** = *low risk*; **Dev effort** = *least effort* for this team).

| Option | Parity | Perf | Maintainability | Dev effort | Ecosystem | Risk |
|---|---|---|---|---|---|---|
| **Tauri 2 mobile** | **5** | 3 | 4 | **5** | 4 | 3 |
| **KMP + Compose (UniFFI)** ⭐ | 4 | 4–5 | **5** | 3 | **5** | 4 |
| Kotlin+Swift native (UniFFI) | 4 | **5** | 4 | 2 | **5** | **5** |
| Flutter + `flutter_rust_bridge` | 4 | 4 | **5** | 3 | **5** | 4 |
| Rust + Dioxus (mobile) | 3 | 3 | 3 | 3 | 3 | 2 |
| Rust + Slint | 2 | 4 | 3 | 2 | 2 | 3 |
| Rust + egui/eframe | 2 | 3 | 3 | 2 | 2 | 2 |
| .NET MAUI / Avalonia | 2 | 3 | 3 | 2 | 4 | 3 |
| C++/Qt | 3 | 4 | 3 | 2 | 3 | 3 |

Key facts behind the scores:
- **Tauri 2 mobile:** system WebView (no engine shipped → tiny binaries; same Rust core + same React UI). But mobile DX is admittedly below desktop; **background Rust execution is an open gap**; push is community/in-progress; not all plugins ported; Android WebView is slower than Chrome and single-threaded.
- **KMP + Compose:** own Skia/Metal renderer (native-class startup/scroll, +~9 MB), **Compose-for-iOS Stable since 1.8 (May 2025)** with a strong production roster (Wrike, Instabee, Respawn 96 % shared, Feres 1 M+, Markaz 5 M+). First-class offline/background/push/secure-storage. Rust via **UniFFI** (Mozilla's Firefox mobile path).
- **Fully native (Kotlin+Compose / SwiftUI):** best possible perf/UX/OS integration; cost = two codebases (~2× UI effort).
- **Flutter:** near-native, most mature cross-platform SDK, best Rust bridge (`flutter_rust_bridge` v2, Flutter Favorite); Dart lock-in.
- **Rust-GUI trio:** maximal language purity, weakest mobile services/ecosystem. **egui is the weakest** consumer phone UI (IME/keyboard/safe-area issues); **Slint** is tiny/fast but mobile is explicitly "work in progress" + tri-license (GPL/RF/commercial); **Dioxus** renders via the platform WebView on mobile (so Tauri-like limits) but UI in Rust.

**Reject:** WASM-in-webview for anything but pure config parsing — no raw TCP (RCON), no arbitrary FS, no listener, no OS keychain.

### Recommendation

**Primary — Kotlin Multiplatform + Compose Multiplatform, Rust core via UniFFI.**
Only option that is *all* of: production-grade on mobile today, native-class perf,
excellent at the exact named features (offline, background, push, secure
storage), and cleanly reuses our Rust core through the canonical path. Costs:
a UI rewrite (React → Compose), a second runtime (Kotlin/Native ↔ Rust), and
build/signing complexity.

**Runner-up — Tauri 2 mobile.** Wins outright if **parity + minimal cost + reuse
the existing UI** is the top constraint (it reuses the Rust core *and* the React
components). Loses on background execution, push and webview perf — i.e. on our
own requirements.

**Third — Flutter + `flutter_rust_bridge`** if we accept Dart: near-native, huge
ecosystem, the best non-KMP Rust integration.

### What would flip the decision
- "Parity + reuse the existing UI, ship fast" → **Tauri 2 mobile**.
- Native feel/a11y/store polish paramount, or Android-first only → **fully native + UniFFI**.
- Team already knows Dart, or wants the broadest plugin ecosystem → **Flutter**.
- Thin-client architecture acceptable (phone → desktop/host API) → the toolkit
  question mostly evaporates; then optimize for UX (KMP/Flutter) over reuse.

---

## 3. Sharing the Rust core

Our core does **sockets (RCON), filesystem (TOML/INI/logs), keyring and hosts an
axum HTTP server** — that shapes the answer.

**Do:** extract a platform-agnostic `core` crate and expose it through a thin
**facade** crate annotated with UniFFI (`#[uniffi::export]`, `Record`/`Enum`/
`Error`), which generates **Kotlin *and* Swift** from one definition.

```
crates/
  core/      # pure domain: schema, TOML/INI, profiles, dispatcher, RCON, logparse, metrics
  facade/    # the ONLY crate with #[uniffi::export]; crate-type = ["lib","staticlib","cdylib"]
  cli/       # ark-config (already exists as a bin) — not linked into mobile
  daemon/    # axum loopback API + Prometheus exporter (desktop only)
```

Move every `main()`, clap parsing, tracing init and the axum server **out** of
the library. Keep the **axum loopback API for desktop/embedded clients only** —
on iOS a localhost listener drags in ATS exceptions/local-network permission and
can't run in the background anyway.

**Android packaging** (`cargo ndk -o app/src/main/jniLibs build --release`,
ABIs `arm64-v8a`/`armeabi-v7a`/`x86_64`) + **16 KB page alignment** (AGP ≥ 8.5.1,
NDK r28+ or the explicit linker flags) — a Play requirement.

**iOS packaging:** build `aarch64-apple-ios` (+ sim slices), `lipo`,
`uniffi-bindgen-swift`, `xcodebuild -create-xcframework`; prefer **`staticlib`**
so the linker dead-strips; ship as a Swift Package `binaryTarget` with a
checksum.

**Secrets on mobile:** desktop `keyring` backends don't exist on Android. Use
`apple-native-keyring-store` (Keychain) on iOS and `android-native-keyring-store`
(Android Keystore; **requires app-side `initializeNdkContext(...)`** before any
Rust secret call) on Android. Store a single master key in the OS store; keep
bulk config encrypted at rest.

**Networking/TLS:** `reqwest` + `rustls` needs `rustls-platform-verifier`'s
Kotlin component on Android (and Android blocks cleartext CRL fetches — a real
gotcha). Prefer **direct UniFFI calls over loopback HTTP on iOS**.

**Top pitfalls:** no future cancellation in UniFFI (expose a `Cancel` object);
tokio must be an explicit, long-lived global runtime (`async_runtime = "tokio"`
is awkward/version-sensitive); `Send + Sync` on shared `Arc` objects; never block
the UI thread (use `spawn_blocking` for log parsing); **iOS kills sockets seconds
after backgrounding** — design foreground-first with silent-push wake + fast
reconnect, don't promise persistent connections.

**Size/startup:** `lto = true`, `codegen-units = 1`, `opt-level = "z"`,
`strip = true` (keep an unstripped archive + dSYM for symbolication),
`panic = "abort"` (decide deliberately), trim default features, ship only real
ABIs. A lean Rust lib should land ~1–4 MB per ABI stripped; lazy-init the
runtime/keychain, and don't pull the axum/Prometheus deps into mobile.

**CI/symbolication:** GitHub Actions matrix (`cargo-ndk` + Android SDK/NDK 28;
`xcodebuild` + XCFramework on macOS), pin toolchain/NDK/AGP/Xcode, commit
`Cargo.lock`. Upload dSYMs (iOS) and `.so` + `mapping.txt` (Android) to Sentry
so native Rust crashes symbolicate.

---

## 4. Suggested phased plan

1. **Extract `ark-core`** (pure, no Tauri/desktop deps) + keep `ark-config` CLI
   and the desktop app consuming it. *No behavior change.* ← first, regardless of UI stack.
2. **Facade crate + UniFFI**; expose the read paths first (`profiles`, `metrics`,
   `status`) and a smoke Android/iOS sample app.
3. **Offline layer** in the shared core (local SQLite SSOT + durable outbox +
   idempotency keys); wire RCON/dispatch writes through it.
4. **Chosen UI** (KMP+Compose): dashboard, server profiles, console/logs, config
   editor, notifications. Ship an internal beta (Play internal / TestFlight).
5. **Hardening**: crash reporting + symbolication, staged rollout, kill-switch,
   a11y + i18n + RTL, size/startup budgets in CI, SBOM/license check.

**Thin-client fallback** if we need something in weeks: reuse the existing
loopback HTTP API from a small native app (read-only status + notifications),
then migrate to the embedded core.

---

## 5. Open decisions (need product input)

- **Primary platform:** both Android+iOS, or Android-first? (Android-first makes
  fully-native + UniFFI a cheap winner and halves the UI work.)
- **Embedded core vs thin client for v1.** (Recommend embedded, but thin client
  ships far sooner.)
- **RCON exposure model:** require Tailscale/WireGuard, or build a TLS agent the
  phone talks to? (Recommend the agent; it also gives cheap pinning + audit.)
- **Budget for the UI rewrite** (only Tauri avoids it).

---

## 6. Sources (selected)**Quality model:** developer.android.com (app startup, slow rendering, macrobenchmark, architecture, offline-first, background tasks, keystore, target API 36), developer.apple.com (App Review Guidelines 2.5.2/3.3.1/4.2/5.1, HIG accessibility, BGTaskScheduler, ATT, privacy manifests), owasp.org MASVS/MASTG/MASWE, w3.org WCAG 2.2 + wcag2mobile, firebase Crashlytics crash-free metrics, Google AIP-185.

**Stacks:** v2.tauri.app/blog/tauri-20 (+ mobile DX caveats), tauri-apps discussions 15222/11688, blog.erikhorton.com (4 Tauri mobile apps), jetbrains.com/compose-multiplatform + Kotlin 1.8 Stable blog, kotlinlang.org KMP FAQ, pub.dev/flutter_rust_bridge, docs.slint.dev platforms/mobile, dioxuslabs.com mobile, github.com/emilk/egui (eframe README, #5848), learn.microsoft.com/dotnet/maui, doc.qt.io Qt for Android/iOS.

**Rust↔mobile:** mozilla.github.io/uniffi-rs (futures, swift overview, async internals; issues #1726/#2391/#2771), docs.rs/cargo-ndk, stadiamaps.com Ferrostar parts 1–2, ac-opensource.github.io "one Rust core", techblog.lycorp.co.jp (naive iOS static lib = 25 MB), docs.rs/keyring + github.com/open-source-cooperative/android-native-keyring-store, rustls-platform-verifier (Android) + matrix-rust-sdk #6319, rustdesk discussions/13019 (iOS background sockets), developer.android.com 16 KB page sizes, github.com/johnthagen/min-sized-rust, docs.sentry.io (iOS/Android debug files).

---

## 7. Phase 1 status — `ark-core` extracted (2026-09-28)

Step 1 of §4 is **done and committed to the working tree**. The new crate
`crates/ark-core` (v2.1.0) owns:

```
crates/ark-core/src/
  lib.rs        re-exports + crate-level contract (no tauri/axum/reqwest/keyring)
  error.rs      Error / Result (no internal deps)
  config/
    mod.rs
    schema.rs
    defaults.rs
    loader.rs
    persister.rs
    validator.rs
    profiles.rs          (multi-server registry)
    custom_validators.rs (declarative rule engine)
    drift.rs
    presets.rs
```

**Why the cut is safe:** measured before moving, `config` referenced exactly
**3** symbols outside itself (all `crate::error`), and `error.rs` referenced
nothing internal. There is consequently no dependency cycle, and the extracted
crate does not need to "know about" the desktop app at all.

**How `src-tauri` still works unchanged:** `src-tauri/src/lib.rs` replaced the
two `mod` declarations with a re-export:

```rust
pub use ark_core::{config, error};
```

Because `ark_core::config` is now a module *named* `config` reachable as
`crate::config`, **all 24 consumers and the 42 + 13 reference sites kept
compiling with zero edits**, and so did the `ark-config` CLI.

**Verification:** `cargo check --bins --tests` clean; `src-tauri`
`cargo test --lib` → **168 passed, 0 failed** (the 18 config/error tests now
live in `ark-core` and run there: `cargo test` in `crates/ark-core` → **18
passed**).

### Still to do (phases 2–5)

- `crates/ark-mobile-facade` — the only crate with `#[uniffi::export]`,
  `crate-type = ["lib", "staticlib", "cdylib"]`; expose read paths first
  (`profiles`, `status`, `metrics`) then RCON/dispatch writes.
- Kotlin/Compose app (Android-first) consuming the generated bindings.
- Offline layer (local SQLite SSOT + durable outbox + idempotency keys) inside
  the shared core, so `ark-core` (not just the facade) carries it.
- On-device verification flow.

### Toolchain note

Needed for phases 2–5: **JDK** — installed 2026-09-28 (Eclipse Temurin
17.0.20.101 via `winget install EclipseAdoptium.Temurin.17.JDK`). Still to
install when packaging/testing on a device: Android SDK + `adb`, `rustup target
add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`,
`cargo install cargo-ndk`, `cargo install uniffi-bindgen` (or the
`uniffi-bindgen` bin shipped by the `uniffi` crate matching our version).

`github.com/google/artemis` (the tool named in the verification brief) is **not
a Rust/Kotlin test harness**: it is an LLM-driven Android-automation agent
(ADB + a multimodal model, Python 3.12+ / `uv`, optional MCP server, optional
web console) that drives a **real device or emulator** and returns screenshots,
Logcat and a report. It is therefore a *later, device-level* verification aid
(useful once a debug APK exists), not part of the build or the Rust↔Kotlin
bridge unit tests, which are plain host-JVM tests against the UniFFI bindings.
