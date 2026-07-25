# CLI Tool Documentation — **NOT IMPLEMENTED in v2.1.0**

> **Honest disclosure — P3b / 2026-07.**
> The previous release shipped a `src-tauri/src/cli.rs` skeleton (254 LOC)
> that pretended to be a standalone CLI bridge. It was never wired into a
> `[[bin]] = "ark-config"` in `Cargo.toml`, never produced a binary, and
> would print fake `"Server: RUNNING (PID: 1234, Uptime: 1h)"` output if
> the operator built it. **This document previously documented that fake
> behaviour.** That skeleton is now deleted (commit on branch
> `fix/p31-gdpr-receipts-ttl`).
>
> **Operator-facing CLI is currently delivered through the desktop app**
> (`Options → Actions → start/stop/restart`, `Options → Plugins → Convex
> deploy`, `Options → Network → Tailscale setup`, …). All commands that
> used to be advertised here are still available via the desktop UI or
> the loopback HTTP API at `127.0.0.1:8765` (bearer-authenticated via
> the token displayed in `Options → Remote Admin`).

## Why no standalone CLI today

A second `[[bin]]` in `Cargo.toml` would need ~0.5 engineer-day of work
to properly wire the same `RouterFn` closure the chat adapters and the
loopback HTTP API already share. Until this is prioritised (roadmap
post-v2.1.0-rc.3), operators use **either**:

- **Desktop UI** (Tauri build) — full feature parity, recommended.
- **Loopback HTTP API** — `curl -H "Authorization: Bearer <token>" \
   -d '{}' http://127.0.0.1:8765/api/v1/{start,stop,restart}`.
- **Chat-bot adapters** — Telegram / Discord / Slack / WhatsApp /
  WeChat / Signal via the multi-channel router.
- **Web admin** (`vercel-deploy`) — once a Vercel token is configured in
  `Options → Plugins → Vercel`, the same commands reach the loopback
  over HTTP.
- **Convex publisher** — once a Convex deploy key is configured in
  `Options → Database → Convex`, internal mutations from the deployed
  Convex backend can `start` / `stop` the server.

## Roadmap

Track the standalone CLI bridge in
[`OPEN_WORK.md`](./OPEN_WORK.md) §**P3**. It is queued behind the
operator-side ship-blocker items and re-enters the queue once
`v2.1.0-rc.3` cuts.
