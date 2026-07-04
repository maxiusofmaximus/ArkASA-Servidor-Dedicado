# Vercel One-Click Deploy (alpha toward v2.1)

> The desktop app can push the **web admin** to your Vercel account
> in one shot. You paste a Vercel token, optionally a project name /
> id, click DEPLOY, and `vercel deploy --prod --yes` runs against
> `web/` — the production URL is captured into the app.

## What you'll do as the operator

1. **Create a Vercel account** on <https://vercel.com>.
2. **Generate a token** at <https://vercel.com/account/tokens>.
   - Choose **Full Account** or scope it to a team if you have one.
   - Tokens are shown once at creation; copy and paste it in the next step.
3. **Open the desktop app → Options → General → Cloud Services →
   Vercel (web admin)**.
4. Open the **"⚡ One-Click Deploy (paste a token)"** disclosure.
5. Paste the token (and an optional project name if you're deploying
   against a pre-existing Vercel project).
6. Click **DEPLOY**.
7. The desktop app runs `vercel deploy --prod --yes` from the `web/`
   directory. Output streams into the panel. When it finishes:
   - Production URL (`https://<name>.vercel.app`) is captured into
     the secret store.
   - Status icon flips to **● connected**.
   - The URL is also displayed as a clickable link.

## What the app does internally

```
vercel_deploy_one_click(token, project_id)  # Tauri command
    │
    ├─ paste_vercel_token(token, project_id)
    │   │
    │   └─ encrypts and saves to ~/.ark-asa/plugins/vercel.toml
    │      (0600 on POSIX; per-user ACLs on NTFS)
    │
    └─ vercel_deploy_web()                  # existing Tauri command
         │
         ├─ runs: vercel deploy --prod --yes
         │  env: VERCEL_TOKEN=<token>
         │        VERCEL_PROJECT_ID=<id>      (if user provided one)
         │
         ├─ parses .vercel.app URL out of stdout
         │  and persists to secret store
         └─ returns stdout to the React UI for streaming display
```

The `vercel` CLI ships as part of `@vercel/cli` (`npm i -g vercel`).
We don't ship or fork it; we just shell out — same posture as the
Convex plugin.

## Why this is safe end-to-end

- **No OAuth flow invented in Rust** — we only relay a token the
  operator already has authority over (created at vercel.com). The
  token is stored at rest with `0600` / NTFS-user-ACL perms and is
  passed to the `vercel` CLI via env var.
- **End-to-end receipts**: the deployment is recorded as a
  `RuntimeCompleted` entry in the same daily JSONL ledger
  (see `docs/IDENTITY_PIPELINE.md`) that the chat bots already use.
- **Process failure is non-fatal**: if `vercel deploy` fails the
  app surfaces the stderr verbatim into the panel — the operator
  fixes and clicks DEPLOY again. Partial writes (a preview URL with
  broken build) don't pollute the production URL field.

## Pre-requisites on the operator's machine

| Requirement | Why |
| --- | --- |
| `node` 18+ (`node --version`) | Needed for `vercel` |
| `npm i -g vercel` | Provides the binary |

If `vercel` isn't installed the desktop app surfaces exactly
`failed to spawn \`vercel deploy\`: ... Install with \`npm i -g vercel\`.`
— the operator sees the fix, not a panic.

## Why this is in `GeneralTab.tsx` and not `DatabaseTab.tsx`

Vercel hosts the **web admin** (a Vercel-deployed Next.js app), not
a database. It lives next to Convex in the **Cloud Services** card
so the operator thinks through the full remote stack in one place:
1. Convex (BaaS) push schema,
2. Vercel (web admin) deploy.

## Where this lands in CHANGELOG and TODO

- **CHANGELOG § "Unreleased" — Vercel One-Click Deploy** confirms
  blocker #2 of the **Open work to actually close v2.1.0** list is
  shipped. There are **2 blockers remaining** (VPS self-host guide,
  Tailscale wizard).
- **TODO.md § "Estado actual — Sesión 3 (Vercel one-click)"** marks
  Vercel deploy as completed; the open blockers list shrinks to 2
  (VPS / Tailscale).
