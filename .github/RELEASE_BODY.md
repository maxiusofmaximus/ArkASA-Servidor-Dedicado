---
title: 'v2.1.0-alpha: OAuth-first Convex + Vercel plugins'
date: 2026-06-30
---

## v2.1.0-alpha.2 — OAuth-first Convex + Vercel plugins (push-button onboarding)

This is the second v2.1.0-alpha which adds the **plumbing** for one-click
Convex + Vercel onboarding — but you still need OAuth clients configured
to make it work in production.

### What changed since v2.1.0-alpha

- **`src-tauri/src/plugins/`** is a new module. Each plugin is a thin adapter
  implementing the `Plugin` trait:
  - `convex/` — opens browser to `https://auth.convex.dev/...`, receives
    code at `http://127.0.0.1:8768/oauth/callback`, exchanges it for a
    deployment_key, saves it to `~/.ark-asa/plugins/convex.toml`, then
    shells out to `npx convex deploy --prod --deploy-key=...`.
  - `vercel/` — same pattern for `vercel.com/oauth/authorize` →
    `http://127.0.0.1:8769/oauth/vercel`, then `vercel deploy --prod --token=...`.
  - `secret_store.rs` — atomic disk write with 0600 perms (Unix).
- **GeneralTab** now has a **"Cloud Services"** section with two cards.
  Each card has:
  - `Connect …` button → opens browser to the OAuth URL.
  - `Push schema` / `Deploy web` button (auto-enabled after connect).
  - Live status badge (● connected / ○ not connected).
- **Auto-install**: `@tauri-apps/plugin-shell` added so
  `openExternal(url)` just works.
- **`convex.json`** added so `npx convex dev` discovers your project
  without a manual `convex configure` dance.

### Things you still have to do (and why)

Convex and Vercel both ship **production OAuth apps** with their own
client IDs. To bring this to v2.1 GA you (the operator) need to:

| What | Why |
|------|-----|
| Create a Convex OAuth client at https://dashboard.convex.dev/settings/oauth | Today we ship a default client_id (`ark-asa-config-manager-default`). Real operators should self-serve. |
| Create a Vercel OAuth client at https://vercel.com/dashboard/integrations/oidc | Vercel ship OAuth apps per-team; the default today won't work for non-personal accounts. |
| Configure `ARK_ASA_OAUTH_CLIENT_ID` + `VERCEL_OAUTH_CLIENT_ID` env vars | These get baked into the desktop build. |

After those three things: open the desktop app, click **Connect Convex**,
log in, click **Connect Vercel**, log in, click **Push Schema** + **Deploy
Web**, done. No TOML editing.

### What still needs to happen for full v2.1.0 (without -alpha)

1. **Tauri loopback HTTP server must serve `/oauth/callback` paths on
   ports 8768 (Convex) and 8769 (Vercel).** Today the routes return
   404 because OAuth callback handling was deferred to Hito 12.
2. **Reconcile `Send`-binding in Telegram polling loop.** Still stub.
3. **Bot plugins (Discord, Signal, WhatsApp, WeChat, SSH, REST).** Only
   Tauri command skeletons exist; the actual Telegram polling loop is
   wired but the rest are still TODO.
4. **Build the NSIS installer (`cargo tauri build`).** Not done.

See `CHANGELOG.md` for the full picture.
