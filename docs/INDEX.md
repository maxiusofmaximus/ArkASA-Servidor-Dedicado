# ARK ASA Documentation Index

> Operator-facing docs you can read top-to-bottom in 30 minutes.
> All paths are relative to `docs/`. Docs are committed to the
> main branch; no external dependencies.

## Quick links

| Page | When to read | Lines |
|---|---|---|
| [`CONVEX.md`](./CONVEX.md) | Setting up Convex one-click deploy | 130 |
| [`VERCEL.md`](./VERCEL.md) | Setting up Vercel one-click deploy | 130 |
| [`HOSTING_SELFHOSTED.md`](./HOSTING_SELFHOSTED.md) | Running ARK on Pi 5 / NUC / WSL2 / macOS | 270 |
| [`NETWORK_TAILSCALE.md`](./NETWORK_TAILSCALE.md) | CGNAT recovery & Tailscale wizard | 130 |
| [`HOSTING.md`](./HOSTING.md) | 7 provider CLI runners for cloud VPS | 200 |
| [`NETWORK_SETUP.md`](./NETWORK_SETUP.md) | Router port-forwarding & DDNS | 245 |
| [`GETTING_STARTED.md`](./GETTING_STARTED.md) | First-run guide for the basic operator |  |
| [`USER_GUIDE.md`](./USER_GUIDE.md) | In-app navigation reference |  |
| [`TROUBLESHOOTING.md`](./TROUBLESHOOTING.md) | 5 common operator errors |  |
| [`FAQ.md`](./FAQ.md) | Frequently asked questions |  |
| [`ARCHITECTURE_AUDIT.md`](./ARCHITECTURE_AUDIT.md) | Honest gap analysis vs OpenClaw/Hermes/Mastra | 230 |
| [`OPEN_WORK.txt`](./OPEN_WORK.txt) | Open work to `v2.1.0` GA + `v2.1.1`/`v2.2.0` roadmap | 230 |
| [`SSH_SETUP.md`](./SSH_SETUP.md) | Sidecar sshd setup for the SSH inbound dispatcher | 130 |

## Plugin-level docs (in code)

| Plugin | Where it lives | Status | Doc |
|---|---|---|---|
| **Convex** | `src-tauri/src/plugins/convex/mod.rs` | One-click deploy ✅ | [CONVEX.md](./CONVEX.md) |
| **Vercel** | `src-tauri/src/plugins/vercel/mod.rs` | One-click deploy ✅ | [VERCEL.md](./VERCEL.md) |
| **Telegram** | `src-tauri/src/integrations/telegram.rs` | Real WS long-poll ✅ | this |
| **Discord** | `src-tauri/src/integrations/discord.rs` | Real WS Gateway v10 ✅ | this |
| **Slack** | `src-tauri/src/integrations/slack.rs` | Real Socket Mode ✅ | this |
| **WhatsApp** | `src-tauri/src/integrations/whatsapp.rs` | webhook mount (S10) ✅ | this |
| **Signal** | `src-tauri/src/integrations/signal.rs` | `runtime_status` parked; signal-cli spawn (post-α.3) | [`OPEN_WORK.txt`](./OPEN_WORK.txt) |
| **WeChat** | `src-tauri/src/integrations/wechat.rs` | webhook mount (S10) ✅ | this |
| **SSH** | `src-tauri/src/integrations/ssh.rs` | `runtime_status` parked; russh spawn (post-α.3) | [`OPEN_WORK.txt`](./OPEN_WORK.txt) |
| **REST** | `src-tauri/src/integrations/rest.rs` | routed through `http_api` (event-driven) ✅ | this |
| **Tailscale** | `src-tauri/src/integrations/tailscale.rs` | wizard wired to Tauri command ✅ | [`NETWORK_TAILSCALE.md`](./NETWORK_TAILSCALE.md) |
| **Self-host (Pi/NUC/WSL2)** | `src-tauri/src/integrations/local_provision.rs` | 7 hardware classes ✅ | [`HOSTING_SELFHOSTED.md`](./HOSTING_SELFHOSTED.md) |

## Connection / model plugins (catalogs added in S6 P2/P3)

| Catalog | Where | Classes |
|---|---|---|
| **VPS providers** | `src-tauri/src/plugins/connection.rs` | Oracle/Hetzner/DO/Self-hosted/AWS/Azure/GCP |
| **AI model plugins** | `src-tauri/src/plugins/model.rs` | OpenAI/Cerebras/NVIDIA NIM/llama.cpp/olllama/vLLM/LM Studio/Custom |

## Status legend

| Mark | Meaning |
|---|---|
| ✅ | Code + tests + honest-truth surfaces (debug-time logs / `runtime_status()`) |
| 🟡 | Listed in `runtime_status()` but real subprocess not yet spawned by `lib::run()` |

## Version timeline

| Tag | Commit | Title |
|---|---|---|
| `v2.0.0` | `<v2.0.0>` | Original release |
| `v2.1.0-alpha` | `<v2.1.0-alpha>` | First alpha (Sesiones 1-5) |
| `v2.1.0-alpha.2` | `<v2.1.0-alpha.2>` | Sesiones 1-5 (clean tag cut) |
| `v2.1.0-alpha.3` | `4a5536f` | Pre-release with 8 bots + 2 dispatchers |
| `v2.1.0` | — | GA. Pending until runtime hooks real + operator acceptance test passes |

## Where to start if you're new

1. Read [`GETTING_STARTED.md`](./GETTING_STARTED.md) (~5 min).
2. Decide: cloud VPS (use [`HOSTING.md`](./HOSTING.md)) or local hardware (use [`HOSTING_SELFHOSTED.md`](./HOSTING_SELFHOSTED.md)).
3. For CGNAT, read [`NETWORK_TAILSCALE.md`](./NETWORK_TAILSCALE.md).
4. For chat-bot ops, peek at `src-tauri/src/integrations/{telegram,discord,slack}.rs`.
5. For architecture decisions, see [`ARCHITECTURE_AUDIT.md`](./ARCHITECTURE_AUDIT.md).

If something looks wrong or missing, the operator-side interactive is: `cargo test --lib` (should report `131/131 passing`) and `cargo build` (should emit 0 warnings).
