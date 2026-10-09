# Mobile: decision record and step-by-step plan

Status: **decision made, phase 2 ready to execute**
Supersedes Â§3â€“Â§5 of `MOBILE_STRATEGY.md` as the *execution* plan (that document
remains the research/quality-model reference).
Date: 2026-09-28 Â· Branch: `fix/rc.4-p27-receipts`

---

## 0. TL;DR â€” the decision

| Question | Answer |
|---|---|
| Bridge Rust â†” Kotlin | **UniFFI 0.32.2**, proc-macro mode, in a dedicated facade crate. Not hand-written JNI. Not Gobley (yet). |
| UI stack | **Compose Multiplatform 1.12.1** (Kotlin), Android-first. |
| Verification the bridge works | **Host-JVM JUnit tests** (`java.library.path` â†’ the Windows `.dll`). **No Android SDK, no emulator, no ArtÃ©mis needed for this.** |
| First executable step | Phase 2A below: `crates/ark-mobile-facade` + a host-JVM smoke test. |
| Role of `google/artemis` | *Device-level* QA later (needs a real device/emulator + an LLM key + `uv`). **Not** a Rust/Kotlin test harness; not on the critical path. |
| Big picture | Phase 2A is cheap and reversible; the expensive irreversible commitment (Android SDK/NDK, Compose, Play) is **gated behind a go/no-go** after 2A. |

**Why this is the professional answer, in one sentence:** it is the same
architecture Mozilla ships in Firefox, it is testable without a phone, it keeps
100 % of the existing Rust core as the single source of truth, and it defers
every toolchain that has no alternative (the Android SDK/NDK) until after the
risky part has already been proven.

---

## 1. What was actually verified (not assumed)

Sources in Â§6. Everything below was fetched or measured on 2026-09-28.

| Fact | Value | Source |
|---|---|---|
| `uniffi` newest release | **0.32.2** (2026-09-23) | crates.io API |
| `uniffi` license | MPL-2.0 (file-level copyleft) | crates.io API |
| UniFFI ships | Kotlin **and** Swift **and** Python from one definition | UniFFI docs |
| Async support | Rust `Future` â†’ Kotlin `suspend fun`; `#[uniffi::export]` + `#[async_trait]` | UniFFI futures docs |
| Tokio integration | supported; keep a `Runtime`, `spawn` onto it. The `async_runtime` macro is **under debate** (#1726) â†’ pin the runtime by hand | UniFFI internals |
| Proc-macro mode | `uniffi::setup_scaffolding!()`; crate must be a library, `crate-type` includes `cdylib` | UniFFI proc-macro docs |
| Kotlin cross-platform community route | **Gobley** (3rd-party KMP bindgen for UniFFI) | UniFFI README |
| Compose Multiplatform current | **1.12.1** (Sept 2026) | JetBrains changelog |
| Gradle current | 9.8.0 (2026-09-24) | services.gradle.org |
| `cargo-ndk` current | requires **Rust â‰¥ 1.86**; auto-detects the NDK | cargo-ndk README |
| Local Rust | 1.96.0 âœ… (above the 1.86 floor) | `rustc --version` |
| Local JDK | Temurin **17.0.20.101** âœ… installed 2026-09-28 | filesystem |
| Local Android bits | **absent**: no SDK, no NDK, no `adb`, no android rustup target | filesystem |
| 16 KB page size | **Play requirement**; needs **AGP â‰¥ 8.5.1**, and `bundletool` does not zipalign by default on 8.3â€“8.5 | Android docs |
| Project Tauri commands | **87** registered in `generate_handler!` | measured |
| Project frontend | **93** `.ts/.tsx` files, **15 855** LOC | measured |
| Project config surface | `ark-core` now holds all of it (phase 1 done) | `<crates/ark-core>` |

---

## 2. The options considered â€” pros and cons

Scored against the project's *actual* constraints: 15.9 k LOC of existing React
UI, an 87-command Tauri IPC surface, a Rust core that already speaks to the OS
(filesystem, RCON sockets, keyring), the user's list (parity, efficiency, speed,
performance, maintainable, easy to use, many features, offline-first), and
**React Native explicitly rejected**.

### Option A â€” UniFFI facade + Compose Multiplatform (Kotlin), Android-first âœ… CHOSEN

- **Pros**
  - Canonical, Mozilla-proven Rustâ†”mobile path; bindings generated, not hand-written.
  - Kotlin **and** Swift from one Rust definition â†’ iOS in phase 6 costs a facade re-export, not a rewrite of the domain.
  - True offline: the Rust core runs **on the phone**; no desktop needed.
  - Native performance (no webview), native background/push/Keystore.
  - **Testable without an Android device**: UniFFI's Kotlin output is plain JVM code loading a `.dll`/`.so` via JNA. A Windows JUnit test can exercise the whole bridge today.
  - Keeps the Rust core as the single source of truth; the 87 commands become a facade over the same functions.
- **Cons**
  - The React UI is **not** reused (Compromise: the *contract* is shared, the widgets are not).
  - Requires the Android SDK/NDK + `cargo-ndk` for the actual app.
  - UniFFI has sharp edges: no future *cancellation*, `async_runtime` macro in flux, `Send + Sync` on shared objects, iOS background sockets die.
  - Two extra layers (facade + offline) before the first screen.

### Option B â€” Tauri 2 mobile (reuse the Rust core **and** the React UI)

- **Pros**
  - Maximum *parity*: same core, same 15.9 k LOC of React, same 87 commands â€” arguably days not weeks to a first build.
  - No new bridge technology, no UniFFI, no Compose.
- **Cons** (these are exactly the user's requirements, which is why it loses)
  - Background Rust execution, push notifications and plugin coverage on mobile are the known weak spots.
  - Webview performance and memory on low-end Android.
  - Mobile DX is young; iOS support is community-driven.
  - It optimises the *first week* while mortgaging the *first year*.
- **Verdict:** excellent **fallback** (documented in Â§5), wrong as the platform.

### Option C â€” Flutter (+ `flutter_rust_bridge` or FFI)

- **Pros:** mature UI framework, great tooling, `flutter_rust_bridge` is solid.
- **Cons:** introduces **Dart** as a third production language alongside Rust and TypeScript; `flutter_rust_bridge` is a different, less standard bridge; no path to reusing anything existing; the Rust ecosystem alignment is weaker than UniFFI's.

### Option D â€” Kotlin Multiplatform + hand-written JNI

- **Pros:** no codegen dependency; full control.
- **Cons:** you hand-maintain a C ABI for every type, every string, every error, every callback, times two platforms. This is a known maintenance sink; UniFFI exists precisely to delete it.
- **Verdict:** only if UniFFI's codegen is blocked, which it is not.

### Option E â€” Gobley (KMP-native UniFFI binding generation)

- **Pros:** generated bindings land directly in a KMP source set (JVM + Native), which is exactly our target shape; built by people shipping KMP+Rust.
- **Cons:** third-party, pre-1.0 maturity, another moving part and an extra Gradle plugin to keep in sync with UniFFI 0.32.x. UniFFI's own Kotlin output already runs on Android (it is JVM code).
- **Verdict:** **revisit at the iOS phase**, where its multiplatform source-set story is genuinely valuable. For Android-first, stock UniFFI + Gradle is fewer moving parts.

### Option F â€” `google/artemis` as the test strategy âš ï¸ misconception corrected

ArtÃ©mis is **not** a Rust/Kotlin harness. It is an **LLM-driven Android
automation agent**: it drives a *real device or emulator* through ADB, uses a
multimodal model (Gemini/Claude/GPT-4o/Qwen-VL) to decide taps, and exposes MCP
+ a Python SDK + a web console. It needs Python 3.12+/`uv`, ADB, scrcpy, FFmpeg
and an **API key**.

- **Where it is great:** after a debug APK exists, it can be told *"open the app,
  add a server profile, verify no crash dialog, screenshot the dashboard"* and it
  will produce screenshots, Logcat and a report â€” with no test code written.
  That is a genuinely excellent **device-level acceptance** layer.
- **Where it is wrong:** it cannot test a Rust `cdylib` bridge in isolation, it
  cannot run in CI without a device and a paid model key, and it is
  non-deterministic by nature (an LLM is choosing the taps).
- **Verdict:** adopt later as a *complement* to deterministic JUnit/Compose
  tests, never as the foundation.

---

## 3. The recommended plan â€” step by step

Ordered so that **every step is independently verifiable** and the expensive,
irreversible tooling comes *after* the risky part is proven.

### Phase 1 â€” extract the platform-agnostic core  âœ… DONE (2026-09-28)

Delivered `crates/ark-core` (config + error), `src-tauri` re-exports it, workspace
root added. Verified: `cargo test --workspace --lib` â†’ 168 + 18 passed.
See `MOBILE_STRATEGY.md` Â§7.

### Phase 2A â€” the facade + prove the bridge on the host JVM  âœ… DONE (2026-09-28)

Executed exactly as planned. What exists now:

| Artifact | Purpose |
|---|---|
| `crates/ark-mobile-facade/` | The **only** crate with `#[uniffi::export]`. Thin facade over `ark-core`; `crate-type = ["lib", "cdylib", "staticlib"]`. |
| `MobileCore` (`#[uniffi::Object]`) | Entry point; constructed with the config dir (Android `filesDir` / iOS Documents). |
| `ProfileView` (`uniffi::Record`) | Owned record, deliberately decoupled from the core type so the core can evolve. |
| `FacadeError` (`uniffi::Error`) | Typed errors â†’ catchable Kotlin exceptions. |
| `scripts/generate-uniffi-bindings.ps1` | Pinned regeneration of the Kotlin bindings. |
| `crates/ark-mobile-facade/jvmTest/` | Gradle + JUnit module that compiles the generated Kotlin **as production code** and tests it against the real Rust `.dll`. |
| `crates/ark-mobile-facade/jvmTest/gradlew` | Checked-in wrapper (Gradle 9.8.0) â€” no global Gradle needed. |

**Verified result:**

```
cargo test --workspace --lib   â†’ 168 + 18 + 4 passed, 0 failed
gradle test (jvmTest)          â†’ 6 passed, 0 failed
```

The JVM tests prove: the library loads via JNA, a `MobileCore` constructs, a
String round-trips through Rust and back, the adopted `default` profile is
listed with `isActive = true`, a real profile persisted by `ark-core` is read
by Kotlin with every field intact, `Option::None` lifts as Kotlin `null`, and a
Rust `FacadeError` surfaces as a typed catchable exception (not a crash).

**Findings worth recording (they would otherwise bite later):**

1. UniFFI 0.32.2 **ships `uniffi-bindgen`** itself; install it pinned with
   `cargo install --version 0.32.2 uniffi --features cli --locked`. A global,
   unpinned `uniffi-bindgen` will eventually drift from the crate and fail with
   a **checksum mismatch** at runtime.
2. An error enum field named **`message` breaks the generated Kotlin**, because
   each variant becomes an exception subclass and `message` collides with
   `Throwable.message`. The field here is `detail`. This is a real trap.
3. `layout.projectDirectory.dir("../../x")` does **not** normalise `..` and
   silently produces `NO-SOURCE`. The build script resolves the repo root with
   `projectDir.absoluteFile.parentFileâ€¦` instead.
4. Prefer `uniffi::setup_scaffolding!()` + `#[uniffi::export]` in the lib, and
   generate the `uniffi-bindgen` bin with
   `cargo install --version 0.32.2 uniffi --features cli --locked`.

### Phase 2B â€” Android toolchain + first APK  âœ… DONE (2026-09-28)

Executed. What exists now:

| Artifact | Detail |
|---|---|
| Android SDK | `platform-tools` (adb), `platforms;android-36`, `build-tools;36.0.0`, **NDK 29.0.14206865** |
| Rust | targets `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`; `cargo-ndk 4.1.2` |
| `android-app/` | AGP 8.7.3, Kotlin 2.4.10, Compose (BOM 2024.12.01), `minSdk 26`, `targetSdk 36`, Gradle wrapper 9.8.0 |
| `scripts/build-android.ps1` | `cargo ndk` per ABI â†’ `jniLibs/` |
| APK | 11.03 MB debug / **3.43 MB release**; contains `lib/arm64-v8a/libark_mobile_facade.so` |

**Verified:**

```
cargo ndk -t arm64-v8a â€¦ build --release   â†’  libark_mobile_facade.so 0.82 MB (pre-redb)
llvm-readelf -l                            â†’  3Ã— LOAD with Align 0x4000 (16 KB) âœ…
llvm-readelf -h                            â†’  ELF64 / AArch64 / DYN
gradlew assembleDebug                      â†’  BUILD SUCCESSFUL
zipalign -c -P 16 -v 4 app-release.apk     â†’  "Verification successful" âœ…
```

**A real bug the Android build caught** (and the reason this phase is worth
running before any UI work): `ark-core::config::defaults::AppConfig::
get_config_dir` had `#[cfg]` arms for Windows/macOS/Linux only. It compiled
fine on Windows and **failed to compile for Android** (`expected PathBuf,
found ()`). Fixed with an explicit `android`/`ios` arm that panics with a clear
message (desktop path resolution is never called on mobile â€” the facade is
handed the app-private directory) plus a neutral fallback for any other target.

### Phase 3 â€” offline layer in the shared core  âœ… DONE (2026-09-29)

Implemented in `ark-core` (`offline/`), not in Kotlin: the store is the single
source of truth, so it must be shared with the desktop app and covered by the
Rust test suite.

| Module | Responsibility |
|---|---|
| `offline/store.rs` | `OfflineStore`: materialised state the UI renders. Profiles stored as JSON per row, so adding a field needs no migration. `replace_profiles` is a full transactional replace â€” a profile deleted on disk cannot survive in the store. |
| `offline/outbox.rs` | `Outbox`: durable write-ahead queue. **The idempotency key is the primary key**, so re-enqueuing the same intent is a no-op rather than a duplicate. States `Pending â†’ InFlight â†’ Done`, with `â†’ Pending` on retry and `â†’ Failed` when the attempt budget is spent. Exponential backoff (1 s doubling, capped at 15 min), `prune_done` for retention, `counts` for a status row. |

**Storage decision: `redb` 4.3, not SQLite.** The research shortlist was
`rusqlite` (bundled) vs `redb` vs `sqlx` vs `sled`. `redb` won because it is
**pure Rust**: no C toolchain in the cross-compile path. That mattered
concretely â€” the Android build had already surfaced one cross-platform defect
(`defaults.rs`), and `rusqlite` bundled would have added a C compiler plus
~1.3 MB/ABI to the same path for no functional gain here. `sled` was excluded as
unmaintained (last release 2021). Confirmed by building: `Compiling redb 4.3.0`
â†’ `Finished` for `aarch64-linux-android` with **no extra flags**.

**Exposed to mobile as `OfflineCore`** (a second UniFFI object, deliberately not
more methods on `MobileCore`): `sync_profiles`, `cached_profiles`,
`enqueue_action`, `outbox_counts`, `mark_action_done`, `mark_action_failed`.
The `OfflineStore`/`Outbox` types never cross the FFI boundary â€” they are
implementation details, so their helpers live outside the exported `impl`.

**Two real defects the work surfaced:**

1. `prune_done(u64::MAX)` saturates `now - age` to 0 and therefore prunes
   **nothing** â€” a silent retention leak. My own test caught it; the limitation
   is now documented and the test asserts the honest semantics instead of a
   convenient fiction.
2. `scripts/generate-uniffi-bindings.ps1` built the `.dll` but the generator
   could read a **stale** one, producing bindings missing `OfflineCore` while
   the `.so` already had it. The script now rejects a cdylib older than 30
   minutes.

**Verification (mobiai-mobile-verification HARD-GATE â€” fresh evidence):**

```
cargo test --workspace --lib   â†’ 168 + 30 + 7 = 205 passed, 0 failed
gradlew test (jvmTest)         â†’ 11 passed, 0 failed
gradlew assembleDebug          â†’ BUILD SUCCESSFUL
cargo ndk -t arm64-v8a         â†’ libark_mobile_facade.so 2.21 MB, 3Ã— LOAD @ 16 KB
adb install -r / am start      â†’ Success / PID 20402 alive
logcat FATAL/AndroidRuntime/UnsatisfiedLink â†’ (none)
```

And the decisive on-device evidence â€” the offline layer really materialised in
app-private storage:

```
run-as com.arkasa.mobile ls files/ark-asa/
  offline.redb   53 KB      â† local source of truth
  outbox.redb    1.03 MB    â† durable write queue
  profiles.json  257 B      â† original config, untouched (it is the source)
  servers/                  â† profiles
```

### Phase 3b — outbox consumer  ✅ DONE (2026-09-29)

The queue is durable and something now drains it: a sender that takes `due()`
entries, marks `InFlight`, attempts the call, then `mark_done` / `mark_failed`.
The idempotency keys already made retries safe, so this was added without
touching the queue's design — which is exactly what the queue was built for.

**The transport decision (the one that shapes the API).** `ark-core` is
portable: it must not own a socket, because the desktop dials `127.0.0.1` and
the phone dials a tunnel. So the split is:

| Layer | Owns | Lives in |
|---|---|---|
| Wire format | RCON framing/parsing, no I/O | `ark-core::rcon` (6 tests) |
| Policy | which entries, in what order, what on failure | `ark-core::offline::sender` (6 tests) |
| Transport | the actual connection | the host: `src-tauri` RCON dispatcher, or the app on Android |

This is why the same state machine — and therefore the same idempotency
guarantees — now backs the desktop queue and the mobile queue, with only the
socket differing.

**Note on the mobile API being *pull*-shaped rather than a callback.** A UniFFI
object cannot receive a Kotlin lambda without rewriting the crate's threading
model, and the app already has a coroutine scheduler. So the mobile surface is
`due_actions()` → `claim_action()` → attempt → `mark_action_done/_failed()`,
with the durable state still in Rust. The one-shot `OutboxSender` in the core
is what the desktop and the tests use.

**Two real defects this phase surfaced, both via its own tests:**

1. **`due_actions()` conflated "recover" with "list".** The first mobile test
   failed and exposed the design flaw: if merely *reading* the queue reclaimed
   in-flight entries, a second drain pass would receive an action already in
   progress — defeating the claim. Split into `reclaim_interrupted_actions()`
   (an explicit decision, once per drain) and `due_actions()` (a pure read).
2. **Reclaiming does not make an entry immediately due.** The second failed
   test was asserting this: recovery goes through the normal failure path, which
   schedules a backoff. Confirmed and the test now asserts the honest semantics —
   the entry returns to `Pending` (never lost) and the next drain picks it up
   once the backoff elapses.

**No-regression refactor:** `src-tauri/src/ark/rcon.rs` no longer defines
`rcon_packet`; it calls `ark_core::rcon::encode_packet`. One definition of the
wire format across desktop, CLI and mobile.

**Verified (fresh, 2026-09-29):**

```
cargo test --workspace --lib        →  168 + 42 + 7 = 217 passed, 0 failed
cargo test -p ark-mobile-facade     →  9 passed (was 7)
gradlew jvmTest                     →  BUILD SUCCESSFUL
cargo ndk arm64-v8a + x86_64        →  .so 2.23 / 2.29 MB
gradlew assembleDebug               →  BUILD SUCCESSFUL (APK 20.75 MB)
adb install + am start              →  Success / PID 1398 alive
logcat FATAL|UnsatisfiedLink        →  none
```

**Device-level proof that the new API reached the phone** (not just that an APK
installed): the `.so` was extracted *out of the installed `base.apk`* and its
SHA-256 is byte-identical to the locally cross-compiled binary
(`72E9500A…A50B`), and it contains the new UniFFI symbols
`offlinecore_due_actions`, `offlinecore_claim_action`,
`offlinecore_reclaim_interrupted_actions`, plus the `DueAction` / `DrainOutcome`
records. The private store on the device holds `offline.redb` (57 KB) and
`outbox.redb` (1.03 MB), both re-written at this launch.



Executed. What exists now:

| Artifact | Detail |
|---|---|
| Android SDK | `platform-tools` (adb), `platforms;android-36`, `build-tools;36.0.0`, **NDK 29.0.14206865** |
| Rust | targets `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`; `cargo-ndk 4.1.2` |
| `android-app/` | AGP 8.7.3, Kotlin 2.4.10, Compose (BOM 2024.12.01), `minSdk 26`, `targetSdk 36`, Gradle wrapper 9.8.0 |
| `scripts/build-android.ps1` | `cargo ndk` per ABI → `jniLibs/` |
| APK | contains `lib/arm64-v8a/libark_mobile_facade.so` |

**Verified:**

```
cargo ndk -t arm64-v8a … build --release   →  libark_mobile_facade.so
llvm-readelf -l                            →  3× LOAD with Align 0x4000 (16 KB) ✅
llvm-readelf -h                            →  ELF64 / AArch64 / DYN
gradlew assembleDebug                      →  BUILD SUCCESSFUL
zipalign -c -P 16 -v 4 app-debug.apk       →  "Verification successful" ✅
```

**A real bug the Android build caught** (and the reason this phase is worth
running before any UI work): `ark-core::config::defaults::AppConfig::
get_config_dir` had `#[cfg]` arms for Windows/macOS/Linux only. It compiled
fine on Windows and **failed to compile for Android** (`expected PathBuf,
found ()`). Fixed with an explicit `android`/`ios` arm that panics with a clear
message (desktop path resolution is never called on mobile â€” the facade is
handed the app-private directory) plus a neutral fallback for any other target.

---
### Phase 4 — the app

Dashboard, server profiles, console/log tail, config editor, notifications.
MVI with one immutable state object; no `Context` in the domain layer.
Ship an internal beta (Play internal track).

### Phase 5 — hardening

Crash reporting + native symbolication (upload `.so` + `mapping.txt`), size and
startup budgets in CI, a11y + i18n + RTL, SBOM/license check, staged rollout
and a kill switch.

### Phase 6 — iOS (only after Android ships)

`aarch64-apple-ios` + simulator slices → `xcodebuild -create-xcframework`,
Swift Package `binaryTarget` with checksum. **At this point re-evaluate
Gobley**, whose multiplatform source sets pay off exactly here.

---

## 4. Rejected orderings, and why

- **Android SDK first.** It is the only piece with no shortcut, so intuition
  says start there. But it cannot be validated until a facade exists, and it
  adds hours of download for zero information. Install it *after* 2A.
- **Compose UI first with hard-coded data.** Produces a demo that cannot talk to
  the core; every screen would be rewritten once the bridge lands.
- **ArtÃ©mis first.** Needs a device, a model key and an APK that does not exist
  yet; it would be *testing the tester*.
- **Bindings by `cargo install uniffi-bindgen`.** Drifts from the crate version
  and breaks silently; the bindgen must be pinned in-repo.

---

## 5. Fallback if the budget collapses

**Thin client:** a small native Android app that talks to the **already-shipped
loopback HTTP API** (or the tunnel-reachable agent) for read-only status and
notifications. Days, not weeks. It gives up embedded/offline, but it is a real
product, and the `ark-core` extraction done in phase 1 is exactly what makes the
later migration to the embedded core a facade change rather than a rewrite.

---

## 6. Sources (verified 2026-09-28)

- UniFFI README, proc-macro guide, futures guide, async internals + issue #1726
  (`mozilla.github.io/uniffi-rs`, `github.com/mozilla/uniffi-rs`).
- crates.io API: `uniffi` 0.32.2, MPL-2.0, feature list.
- `github.com/gobley/gobley` README (KMP bindgen).
- `JetBrains/compose-multiplatform` CHANGELOG (1.12.1, Sept 2026).
- `services.gradle.org/versions/current` (9.8.0).
- `github.com/bbqsrc/cargo-ndk` README (MSRV 1.86, NDK auto-detection).
- `developer.android.com/guide/practices/page-sizes` (16 KB, AGP â‰¥ 8.5.1).
- `github.com/google/artemis` README (LLM Android-automation agent, ADB + `uv`
  + model key; **not** a JUnit/Rust harness).
- Local measurements: `rustc 1.96.0`, Temurin 17.0.20.101, 87 Tauri commands,
  93 TS files / 15 855 LOC, no Android SDK/NDK/targets present.

---

## 7. Decision summary table

| Step | Cost | Reversible? | Information gained | Verdict |
|---|---|---|---|---|
| Phase 1 `ark-core` | done | yes | architecture is shareable | âœ… done |
| **Phase 2A facade + JVM test** | **hours** | **yes** | **does the bridge actually work?** | **â† do now** |
| Phase 2B Android toolchain + APK | 1â€“2 days | yes (disk) | can it run on a phone? | after 2A, needs go/no-go |
| Phase 3 offline layer | weeks | yes | offline correctness | after 2B |
| Phase 4 app | weeks | yes | product | after 3 |
| Phase 5 hardening | weeks | yes | shippable | before Play |
| Phase 6 iOS / Gobley | weeks | yes | second platform | after Android ships |

## 8. Does this choice get maximum performance, versus "pure Android"?

The user asked specifically whether the KMP+UniFFI route costs anything against
native Android. Measured answers, not opinions:

| Dimension | Result here | Verdict |
|---|---|---|
| Native library size (shipping ABI) | **0.82 MB** stripped, 843 KB in the APK | lean; there is no JVM-side domain code |
| Dynamic dependencies | only `libdl.so` + `libc.so` | no surprise transitive native deps |
| Cost of an FFI call | one JNA `Native.register` trampoline per call | the **only** real overhead vs. pure-JNI; see below |
| Domain logic location | 100 % Rust, shared with the desktop app | one implementation, not two |
| 16 KB page compliance | already satisfied | no rework later |
| APK size | 11 MB debug (Compose + JNA + Rust) | normal for Compose; release + R8 + ABI splits reduce it |

**Where "pure Android" would actually be faster, and by how much.** The only
genuine difference is that UniFFI's Kotlin calls through **JNA** rather than
hand-written JNI. JNA adds a small per-call boundary cost (roughly tens of
microseconds, dominated by the JNA dispatch, not by our code). This is
irrelevant for this app, whose calls are coarse-grained (list profiles, read
metrics, send an RCON command, validate a config) and dominated by I/O â€” file
reads, sockets, TOML parsing â€” all of which happen inside Rust. It would matter
if we were crossing the boundary thousands of times per second on the hot path
of a game loop; we are not.

**Where this choice is strictly faster than pure Android.** Every rule, parser
and validator is written **once** in Rust and reused by the desktop app, the
CLI and the phone. A pure-Kotlin rewrite would duplicate the entire config
schema, INI/TOML generation and validation logic into Kotlin, where it would
drift from the Rust implementation â€” the classic two-sources-of-truth defect.
It would also forfeit iOS, which the same facade already supports.

**If per-call overhead ever became real** (it will not for these workloads), the
escape hatch is supported: generate the `uniffi-bindgen-java`/JNI-style bindings
or hand-write JNI for the hot function only, keeping UniFFI for the rest. That
is a localised change behind the facade, not a rewrite of the app.

**Bottom line:** the domain runs at native speed on the phone; the boundary is
coarse-grained and I/O-bound, so the JNA tax is unmeasurable in practice; and
the architecture buys single-source-of-truth plus iOS for free. For this app the
choice is not a performance compromise â€” a pure-Kotlin rewrite would be *slower
to build, slower to keep correct, and no faster to run*.

---

## 9. Android module layering (mobiai-android-architecture review)

The first Android screen shipped as a single `MainActivity.kt` holding layout,
state and core access â€” fine as a smoke test, wrong as a pattern. Reviewed
against the `mobiai-android-architecture` skill's MVVM + layered conventions and
restructured:

```
com/arkasa/mobile/
  MainActivity.kt                    # entry point: builds the core, wires the factory
  data/
    MobileCoreProvider.kt            # THE ONLY place that knows Context / filesDir
  presentation/
    DashboardScreen.kt               # @Composable, observes state, no Context, no IO
    DashboardViewModel.kt            # StateFlow<DashboardUiState>, reads on Dispatchers.IO
    DashboardUiState.kt              # sealed interface: Loading | Ready | Error
    components/
      ProfileCard.kt                 # stateless, previewable
```

What the review changed, concretely:

| Convention (skill) | Before | After |
|---|---|---|
| UI / domain / data layers | 1 file, everything mixed | `data/` + `presentation/` + `components/` |
| One immutable state object | ad-hoc `mutableStateOf` in the composable | `StateFlow<DashboardUiState>` (sealed) |
| State holder lifecycle-agnostic | core built inside the Activity | core injected into the ViewModel |
| UI never touches IO/DB/socket | `core.listProfiles()` called during composition | ViewModel reads on `Dispatchers.IO` |
| Reusable components | inline card | `components/ProfileCard.kt`, stateless |
| State survives config change | state lost on rotation | ViewModel survives; only the first read is re-sent |

Deliberate non-adoptions, with reasons: no Hilt (one ViewModel with one
dependency does not justify the annotation processor and build-time cost â€” the
skill lists Hilt as "most common", not mandatory), and no `UseCase` layer (a
single read with no business rules; introducing a pass-through UseCase would be
ceremony). Both should be revisited when the dashboard grows real operations
(RCON commands, config writes).

**Verification (mobiai-mobile-verification HARD-GATE â€” fresh evidence):**

```
gradlew assembleDebug        â†’ BUILD SUCCESSFUL
adb install -r app-debug.apk â†’ Success
pidof com.arkasa.mobile      â†’ 29959 (alive)
logcat FATAL/AndroidRuntime/UnsatisfiedLink/NullPointer/IllegalState â†’ (none)
screenshot                   â†’ dashboard renders "Servidor principal Â· activo"
```
