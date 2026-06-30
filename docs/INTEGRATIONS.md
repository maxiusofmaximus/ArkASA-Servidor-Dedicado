# Integrations

> How ARK ASA Admin bridges to external cloud services and chat
> platforms. Everything goes through the desktop app's **plugin
> registry** (`src-tauri/src/plugins/`); the UI surface is in
> `frontend/src/components/options/GeneralTab.tsx` →
> **Options → General → Cloud Services**.

---

## Cloud services

### Convex (BaaS for the web admin)

| | |
|---|---|
| Auth mechanism | `npx convex login` ← GitHub OAuth via Convex CLI |
| Where credentials are kept | `~/.convex/credentials.json` (managed by Convex CLI) + `~/.ark-asa/plugins/convex.toml` (the Convex plugin's mirror) |
| Where the deployment URL goes | first `npx convex dev` writes `convex/.env.local` + `convex/.convex/config.json` |
| Deploy mechanism | `npx convex deploy --prod --yes` (with `CONVEX_DEPLOY_KEY`) |
| Documentation | <https://docs.convex.dev/cli> |

#### Operator flow

1. Open the desktop app, **Options → General → Cloud Services**.
2. Click **Connect Convex**. The plugin spawns `npx convex login`,
   which opens the browser to GitHub's device-code page. Authorize;
   CLI watches; Convex writes the credentials.
3. Once connected, click **Push schema**. The plugin invokes
   `npx convex deploy --prod` so your `convex/convex/*` reaches the
   cloud BaaS.
4. The web admin at `<https://ark-asa-admin.vercel.app>` (Vercel URL)
   starts streaming real-time state.

#### Fallback

If you can't run `npx convex login` (e.g. headless server, SSH'd
workstation), copy the deploy key from another machine and use
**Paste deploy key**. The same `~/.ark-asa/plugins/convex.toml` gets
written.

---

### Vercel (web admin hosting)

| | |
|---|---|
| Auth mechanism | `vercel login` (CLI) |
| Where credentials are kept | `~/.vercel/auth.json` (managed by Vercel CLI) + `~/.ark-asa/plugins/vercel.toml` (the plugin's mirror) |
| Deploy mechanism | `vercel deploy --prod --yes` (with `VERCEL_TOKEN` env export) |
| Documentation | <https://vercel.com/docs/cli> |

#### Operator flow

1. In the desktop app: **Options → General → Cloud Services → Deploy web → Connect Vercel**.
2. The plugin spawns `vercel login`. CLI opens
   https://vercel.com/api/registration/.../login. Click Authorize.
3. Once connected, the **Deploy web** button runs `vercel deploy --prod`.
4. The URL `https://ark-asa-admin.vercel.app` is shown.

#### Fallback

For non-interactive machines, set `VERCEL_TOKEN` (issued at
<https://vercel.com/account/tokens>) and use **Paste VERCEL token** in
the same Cloud Services card.

---

## Local HTTP API (Convex → Tauri → ARK binary)

The Convex backend calls the **Tauri app's loopback HTTP API** at
`http://127.0.0.1:8765/api/v1/internal/dispatch`. This is what does
the actual `start_server`/`stop_server` invocations on the ARK
binary. See `src-tauri/src/integrations/http_api.rs`.

Endpoints:

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/health` | liveness |
| `GET` | `/api/v1/state` | current cluster status (maps running, MOTD, IPs, primary IP) |
| `GET` | `/api/v1/logs?tail=200` | tail of `ShooterGame.log` |
| `GET` | `/api/v1/config` | full TOML config |
| `POST` | `/api/v1/start` | start a cluster map (admin role only) |
| `POST` | `/api/v1/stop` | stop |
| `POST` | `/api/v1/restart` | restart |
| `POST` | `/api/v1/internal/dispatch` | internal service-to-service: Convex forwards a `RemoteCommand` here |

All endpoints except `/api/v1/health` require `Authorization: Bearer <token>`.
Token lives at `~/.ark-asa/admin.token` and is surfaced in
**Options → General → Remote Admin** (Hito 12 wire-up) — for now
read the file directly.

---

## Channel plugins (Hito 6-10)

Each adapter implements `Plugin::start(ctx)` and renders a polling
loop. Today only the **Telegram adapter** is wired; others are
stubs:

| Channel | Auth | Tauri module | Status |
|---|---|---|---|
| Web (Convex) | via Convex backend | `convex/` | ✅ ready (this file) |
| Telegram | `BotFather` token | `integrations/telegram.rs` | ✅ loop pending Send-bound |
| Discord | Discord dev portal bot token | (todo) | stub |
| WhatsApp | Meta Cloud API business account | (todo) | stub |
| Signal | `signal-cli` daemon + phone | (todo) | stub |
| WeChat | WeChat OA app credentials | (todo) | stub |
| SSH | authorized_keys | (todo) | stub |
| HTTP/REST | bearer tokens | `integrations/http_commands.rs` | scaffold |

Adding the missing channel adapters is a **token-by-token** process:

1. Implement `src-tauri/src/plugins/<channel>/mod.rs` following the
   `Setup`-`Begin`/`Poll`-`Dispatch` pattern.
2. Add the channel-descriptor to `plugins/mod.rs::register_default_plugins`.
3. Add a Tauri command `command:` in the React `GeneralTab`.
4. Document the operator's manual flow here.

For bots (Telegram, Discord, etc), the operator pastes the bot token
into **Options → Plugins** (Hito 12 mock), which writes
`~/.ark-asa/plugins/<channel>.toml`.

---

## Debugging

```bash
# Convex plugin
cat ~/.ark-asa/plugins/convex.toml
tail -f ~/.convex/config.json
npx convex logs --prod
npx convex dashboard

# Vercel plugin
cat ~/.ark-asa/plugins/vercel.toml
vercel inspect --prod

# Tauri loopback
curl -H "Authorization: Bearer $(cat ~/.ark-asa/admin.token)" \
     http://127.0.0.1:8765/api/v1/state
```

Disable or detach any plugin by deleting its TOML file at
`~/.ark-asa/plugins/<plugin>.toml`; the plugin registry reflects the
disk state on next restart.
