---
title: 'v2.1.0-alpha.2: CLI-bridge plugin pattern (replaces fake OAuth)'
date: 2026-06-30
---

## v2.1.0-alpha.2 — CLI-bridge plugin pattern

Previously the alpha.1 release shipped OAuth endpoints that I made
up — `https://auth.convex.dev/oauth/authorize` and a similar one for
Vercel. Those don't exist (and won't for the foreseeable future
unless Convex / Vercel ship dedicated login APIs). This version
replaces them with what **actually** works: shelling out to the
vendor's first-party CLI.

### What you'll do as the operator

```
pnpm install                   # first time
pnpm tauri:dev                 # desktop app
# In Options → General → Cloud Services:
#   1. "Connect Convex"  →  spawns `npx convex login`. CLI opens browser.
#   2. Authorize Convex's GitHub device flow.
#   3. "Push schema"    →  spawns `npx convex deploy --prod`.
#   4. "Connect Vercel"  →  spawns `vercel login`. CLI opens browser.
#   5. Authorize vercel.com.
#   6. "Deploy web"      →  spawns `vercel deploy --prod`.
```

That's the whole flow. No TOML to copy, no env vars to type beyond
the defaults. Adapters read the secrets each CLI writes
(`~/.convex/credentials.json` for Convex, `~/.vercel/auth.json` for
Vercel) and persist a mirror in `~/.ark-asa/plugins/<channel>.toml`.

If a CLI flow doesn't fit your setup, every plugin has a
**Paste … token** fallback: paste an existing deploy_key (Convex)
or `VERCEL_TOKEN` (Vercel) and the same plugin starts working.

### What changed since v2.1.0-alpha

- Removed: `crate::plugins::convex::begin_convex_oauth`,
  `crate::plugins::convex::complete_convex_oauth`,
  `crate::plugins::vercel::begin_vercel_oauth`,
  `crate::plugins::vercel::complete_vercel_oauth`.
- Added: `begin_convex_link`, `begin_vercel_link`,
  `past_convex_deploy_key`, `paste_vercel_token`. Both `*_link`
  variants shell out to the vendor's CLI; both `paste_*` variants
  are the air-gapped fallback.
- `PluginDescriptor::oauth_url = None` for both new plugins — neither
  Convex nor Vercel exposes per-operator OAuth serverside for this
  use case.
- New `docs/INTEGRATIONS.md` documents the operator flow.
- `CHANGELOG.md` updated.

### What still blocks a v2.1.0 GA
1. **Plugin registry enumeration at runtime.** Today
   `register_default_plugins` only ships convex + vercel. The other
   channels (Discord, WhatsApp, Signal, WeChat, SSH) still have to
   be added one by one. The Plugin trait is documented and stable —
   adding a new adapter takes about 200 lines.
2. **Telegram polling loop's Send bound.** `spawn_looper` currently
   returns an empty future because `tokio::sync::MutexGuard` over
   `reqwest::Client` isn't `Send`. Real work is in Hito 12 (last
   polish).
3. **Build the NSIS installer** (`cargo tauri build`). Pending
   Windows + Internet host with NSIS in PATH.

### Credibility check

```
$ cargo check
   Compiling ark-asa-config v2.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.16s
```

```
$ npx tsc --noEmit
(no output)
```

…across all 5 packages (`frontend`, `web`, `convex`,
`packages/shared-types`, `src-tauri`).
