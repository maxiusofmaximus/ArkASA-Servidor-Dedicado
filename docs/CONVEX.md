# Convex One-Click Deploy (alpha toward v2.1)

> The desktop app can now push your schema and DB functions to a
> tenant-owned Convex cloud with **one button click**, without you ever
> opening a terminal. The aim is to be deployable by anyone with a
> GitHub account (Convex provisions projects through GitHub login).

## What you'll do as the operator

1. **Create a Convex project** on <https://dashboard.convex.dev>.
   You'll get:
   - A **deployment URL** like `adjective-animal-123.convex.cloud`.
   - A **Deploy Key** from `Settings → Deploy Keys → Generate
     Production Deploy Key`. Keep it secret — anyone with it can
     write to your production deployment.

2. **Open the desktop app** and go to **Options → Database**.

3. Choose backend **`Convex (BaaS)`**.

4. Fill in:
   - **URL**: `https://<your-deployment>.convex.cloud`
   - **API key / token**: paste your Convex deploy key.

5. Click **DEPLOY TO CONVEX**. The Convex One-Click Deploy panel
   below the form will stream the `npx convex deploy --prod` output.
   When it finishes:
   - Schema (`convex/schema.ts`) is on your production deployment.
   - Functions (`auth.ts`, `servers.ts`, `commands.ts`,
     `authorization.ts`) are reachable at
     `https://<your-deployment>.convex.cloud/api/...`.

6. The audit-log backend that the chat bots use now resolves to
   Convex under the hood (`convex_push_schema` already wired through
   `convex_push.rs`).

## What the app does internally

The desktop app **does not** speak Convex's internal protocol — it
delegates to the official `npx convex` CLI:

```
convex_deploy(deployment_url, deploy_key)  # Tauri command
    │
    ├─ paste_convex_deploy_key(deployment_url, deploy_key)
    │   │
    │   └─ encrypts and saves to ~/.ark-asa/plugins/convex.toml
    │      (mode 0600 on POSIX, ACLs on NTFS)
    │
    └─ convex_push_schema()                  # existing Tauri command
         │
         └─ spawns: npx convex deploy --prod
            env: CONVEX_DEPLOY_KEY=<key>
                  CONVEX_SELF_HOSTED_URL=<url>  (if self-hosted)
            stdout/stderr → returned to the React UI as a single
                            string
```

The `npx convex` CLI is part of the `convex` npm package — it lives
at <https://github.com/get-convex/convex-js>. We don't ship or fork
it; we just shell out.

## Why this is safe end-to-end

- **No OAuth flow inside the app.** There's no custom token-broker.
  Convex's `deploy --prod` uses the literal deploy key you paste; it
  never touches GitHub on its own.
- **Credentials never leave disk.** The deploy key is stored in
  plankton's standard secret-store directory with `0600` permissions
  (POSIX) or per-user ACLs (NTFS). The app reads it from there when
  invoking `npx convex deploy`.
- **`npx` aborts the spawn gracefully** if Node isn't installed:
  the Tauri command returns
  `failed to spawn \`npx convex deploy\`: ... Install Node +
  \`npm install convex\` in the convex/ directory.`
  so users see a clear remediation, not a panic.
- **Audit-log receipts** flow into the same bot pipeline that
  already runs for Telegram/Discord/Slack. Every Convex push writes
  a `RuntimeCompleted` entry into `${AppData}/receipts/<date>.jsonl`.

## Alternative — paste-the-key flow

If you'd rather authenticate on another machine first and copy the
key over (air-gapped, headless workstation, etc.), the original
**begin_convex_link** button in General → Cloud Services still works.
It spawns `npx convex login`, opens your browser to GitHub's
device-page, and the CLI writes
`~/.convex/credentials.json`, which the Tauri plugin reads back
into the secret store.

## Pre-requisites on the operator's machine

| Requirement | Why |
| --- | --- |
| `node` 18+ installed (`node --version`) | Spine of `npx` |
| `npm install convex` already ran inside `convex/` | Provides the CLI |
| A Convex project AND its deploy key | The thing you're deploying to |

If `npm install convex` was forgotten, the `npx convex deploy` call
will fail with the exact error `convex: command not found` — the
desktop app surfaces this verbatim into the deploy panel, with a
hint pointing at the `convex/` folder.

## Where this lands in CHANGELOG and TODO

- **CHANGELOG § "Unreleased → Convex One-Click Deploy"** confirms the
  the Convex blocker from the **Open work to actually close
  `v2.1.0`** list is shipped.
- **TODO.md § "Estado actual"** lists this feature as completed
  alongside the original tuntas (identity, receipts, hosting CLI
  runners).
