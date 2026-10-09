# Research & New Ideas — ARK ASA Configuration Manager

> Deep research pass (2026-09). Three parallel investigations: (1) similar
> ARK/game-server management projects, (2) GUI vs CLI vs hybrid tooling for
> self-hosted servers, (3) feature ideation + prioritisation methodology.
> This file is the durable synthesis: what to build next, in what order, and
> how to keep finding good ideas. Full source lists are at the end.

---

## 0. TL;DR

- **Our differentiator is not "another INI editor."** Incumbents split into
  generic panels (deep fleet/RBAC, shallow ARK semantics: Pterodactyl,
  Pelican, AMP) and ARK tools (deep ARK semantics, single-surface, often
  abandoned/unlicensed: ASM, arkmanager, POK, DiFuks).
- **Our wedge:** *ARK-native depth + three surfaces (desktop / CLI / web) +
  self-hosted OSS + no per-slot billing*, executing **reliability automation**
  and **config safety** better than both camps.
- The two failure modes to design against, both heavily documented in the wild:
  1. **A manager that rewrites the ARK config and destroys user intent** (the
     dominant r/playark complaint for a decade).
  2. **A management surface that is unsafe on the network or unusable
     headlessly.**
- Fastest wins reuse components we already have: **Prometheus/Grafana
  exporter**, **config-drift detection**, **public status page**. Strongest
  differentiators: **mod-health watchdog** and **self-healing guardian**
  (they attack the #1 recurring pain: mods/updates breaking servers).

---

## 1. Competitive landscape (condensed)

### ARK-specific

| Tool | Surface | License | Standout | Weakness we exploit |
|---|---|---|---|---|
| **ARK Server Manager (ASM)** | Desktop (Win) | source-available, no license | whichever | ASE-only, dev ended, **mangles configs** |
| **ark-server-tools / arkmanager** | CLI (Linux) | MIT | staged updates, multi-instance, cron | CLI-only, hand-edited config |
| **POK-manager (Docker)** | CLI/Docker | OSS | graceful save barrier, AsaApi, rollback | Linux/Docker only |
| **Arkon** | Discord bot | proprietary | deepest *player-facing* features (shop, stats, live map, anti-cheat) | not a lifecycle/config manager |
| **AMU (Ark-Mod-Updater)** | Desktop (Win) | OSS | ~370 typed settings, mod pipeline, signed self-updater | Windows-only |
| **sanjay-m6 / ASM 2.0** | Tauri+React | OSS | closest architectural peer; AI log analysis, visual cluster builder | Windows-only, AI-heavy |
| **AsaApi** | framework | MIT | server-side plugin ecosystem | fragile across ASA updates |

### Generic panels

Pterodactyl (MIT, Docker/Wings, SFTP, RBAC), **Pelican** (AGPL, real plugin
system, S3 remote backups), PufferPanel (single binary), Crafty (Minecraft,
schedules/RBAC), **AMP** (proprietary, state tracking, `ampinstmgr` CLI,
firewall automation), LinuxGSM (MIT, 100+ servers, alerting), WindowsGSM
(MIT, plugin system), Maryl (modern LinuxGSM control plane).

**Feature catalogue they have that we don't (deduped):** mod browser with
search/resolved names; mod load-order + auto-update + `ActiveMods` sync;
AsaApi plugin manager; crash/hang watchdog + adopt-running-server;
staged/zero-downtime updates; countdown broadcasts; graceful-stop primitive
everywhere; scheduler (cron + event triggers); deep typed config + raw array
editors (comment/encoding preserving); cluster "sync settings"; backup
rotation/retention + restore + include-configs; whitelist/ban + cluster ban
sync; live player view + status embeds; firewall/UPnP + reachability check;
web file manager; SFTP; subuser RBAC + audit log + API keys; OIDC/2FA;
multi-node; import existing install; signed self-updater w/ rollback.

---

## 2. New feature ideas (ranked backlog)

Effort: S ≈ ≤1 day, M ≈ 2–5 days, L ≈ 1–2 weeks. "Plugs into" = existing
component reused.

| # | Idea | Pitch | Effort | Plugs into |
|---|------|-------|--------|------------|
| 1 | **Prometheus exporter + Grafana bundle** | Expose `/metrics` (players, uptime, RSS, save age, restarts, query latency) from the loopback API + ship a ready Grafana dashboard JSON | **S** | loopback API + metrics |
| 2 | **Config drift detection + preset library** | Diff *desired* config vs *on-disk* (catch silent resets) and publish/import shareable presets | **S** | config editor + INI gen + backup |
| 3 | **Mod-health watchdog** | After every ARK/Steam update, verify each mod loads, quarantine broken ones, notify before players hit a crash loop | **M** | mods + diagnostics + notifications |
| 4 | **Self-healing guardian** | Supervisor: crash-loop / query-hang / leak / stale-save detection → graded response (restart → isolate mod → restore save → alert) | **M** | lifecycle + backup + diagnostics |
| 5 | **Public status page + embed badges** | One-toggle public page (players/uptime/map/mods/rules) + SVG badges for Discord/websites | **S** | web dashboard + loopback API |
| 6 | **Cluster integrity guard** | Pre-flight validation of shared cluster dir; detect stale/corrupt transfer files, enforce transfer policy, recover orphan characters | **M** | cluster + diagnostics + backup |
| 7 | **Backup integrity + DR drills** | Checksum-verify backups and periodically restore into a scratch dir, alert if unusable | **M** | cloud backup + diagnostics |
| 8 | **Alert rules + notification routing** | Threshold rules (players<1 for 2h, RAM>90%, no save in 30m, mod failed) routed per severity to the right channel | **S/M** | notifications + metrics |
| 9 | **Discord command-center bot** | Slash commands (status/players/kick/ban/broadcast/restart/whitelist) + log feed over the existing chat bridge | **M** | remote admin + RCON |
| 10 | **Event & rate scheduler** | Calendar-driven rate changes (weekend 2×, `ActiveEvent`), event nights, timed cluster-wide announcements | **S/M** | config + RCON + chat |
| 11 | **Player/tribe stats + leaderboards** | Auto-tracked kills/tames/playtime + weekly winners announced in chat, shown on web dashboard | **M/L** | Convex web + chat bridge |
| 12 | **Tribe log viewer + per-tribe alerts** | Persistent searchable tribe-log archive with keyword alerts (offline raid, dino death, admin action) | **M** | diagnostics + Convex |
| 13 | **Multi-server local** *(this session)* | Manage several servers from one app, each with its own config + saves + selector | **M/L** | config + commands + CLI + UI |
| 14 | **Live player map** | Real-time map of players/tames/tribe territory, click-to-act | **L** | web + savegame parsing + RCON |
| 15 | **Vote + whitelist onboarding portal** | Player web page to vote/apply/verify + Discord role linking | **M** | web + chat + Convex |
| 16 | **Shop / donation & rewards engine** | Points from playtime/votes, in-game shop, RCON-delivered kits | **L** | RCON + payments + Convex |
| 17 | **Admin AI assistant + dino/item DB** | NL ops ("wipe wild dinos on all maps") + Dododex-style spawn/stat DB feeding RCON | **M** | RCON + diagnostics |

**Sequencing:** ship the cheap/visible wedge first (#1, #2, #5) in one cycle;
then reliability hardening (#3, #4); then the flagship engagement bets
(#11, #14) once validated.

---

## 3. GUI vs CLI vs hybrid — design rules

**Verdict: hybrid, one shared core.** The community's own framing:

> "GUI - easier discovery / CLI - easier to script/automate… If you're making
> tools that will be automated… stick with CLI. If you're making an enduser
> facing service — you'll need a GUI." — r/selfhosted

> "GUIs have a nice ramp-up and a fixed ceiling, whereas command lines…
> as long as they're scriptable, the ceiling is close to unlimited." — HN

Our loopback API + `integrations::dispatch` is already that shared core; the
GUI, CLI, bots, web and Convex are thin adapters. Rules to keep:

1. **Anything the GUI can do, the CLI must also do** (and vice versa) — same
   core call.
2. **CLI contract:** `--json`/`-o json` everywhere, auto-JSON when not a TTY;
   `stdout = data`, `stderr = context`; semantic exit codes; typed error
   envelope; `--dry-run` default for destructive ops; `--yes` to bypass
   confirmation; **never** accept secrets as flags (use file/stdin/env).
3. **Config is source of truth and must never be wholesale-rewritten:**
   preserve unknown keys + section order, force correct encoding (ASCII or
   UTF-16+BOM), write only the canonical `Saved/Config/<Platform>Server/` path,
   refuse to write while the server runs or the file changed underneath
   (hash-check), snapshot+diff+rollback before every write.
4. **Back up `Mods/`, not just `Saved/`** — the mod-rollback gap is the single
   most-cited unannounced breakage.
5. **Cluster consistency validator** before start (matching `-clusterid`, shared
   `-ClusterDirOverride`, identical mod lists, disk space).
6. **Daemon ≠ GUI:** servers must survive the GUI closing (tray residency) and
   be installable as a service (`ark-config service install`).

### Security lessons (from Pterodactyl's CVEs, etc.)

- Never put secrets in URLs/query strings (logged in plaintext).
- Loopback API binds `127.0.0.1` by default; remote = reverse proxy + TLS + auth.
- Non-loopback bind requires explicit opt-in.
- RCON: default localhost, never port-forward 27020, restrict password charset.
- Rate-limit login; audit log; remote tokens scoped to a single instance.
- Never run the GUI elevated; small privileged helper + UAC/pkexec.
- Auto-update opt-in, staged, signed, with rollback.

### Most-wanted automation recipes (all → ordered task chains)

`broadcast(countdown) → saveworld → verify save → backup → stop → update →
mod swap → start`. Needs: ordering, per-step delays/timeouts, retries, run
history, timezone-aware cron, event triggers (player joined/left/threshold).

---

## 4. How to keep surfacing good ideas (repeatable process)

**Frameworks to combine** (don't pick one): JTBD / "how might we" to frame
problems; **Opportunity Solution Tree** (Torres) to map outcome →
opportunities → solutions → experiments; **Kano** (8–15 users) to split
must-be from delighter; **MoSCoW** to scope a release; **ICE** for weekly
triage; **RICE/WSJF** for the quarterly ranking; **SCAMPER/Blue Ocean** as
periodic unstick prompts; telemetry + dogfooding + 5-whys for evidence.

**Cadence**

- **Weekly (30–45 min):** capture every signal into an Idea Inbox; 5-whys the
  top 1–2 into *opportunities* (not features); ICE-screen; dogfood on a real
  server and log friction.
- **Monthly:** community scan (Reddit/Steam/Discord/GitHub); 2–5 short
  interviews (ask about *past behaviour*, not "would you use X"); update OST.
- **Quarterly (half-day):** Kano survey → MoSCoW against a fixed *appetite* →
  RICE/WSJF → commit Now/Next/Later with a measurable outcome each → retro.

**Artifacts**

```
docs/product/
  OUTCOMES.md          # north-star + this quarter's single outcome metric
  opportunities.md     # "I struggle to…", evidence, Importance×(1−Satisfaction)
  idea-inbox.md        # raw captures (or a GitHub "idea" issue template)
  ost.md               # Opportunity Solution Tree
  backlog.md           # idea, reach, impact, confidence, effort, ICE, RICE, MoSCoW
  kano-survey.md       # question set + tally
  research/YYYY-MM-*.md
  roadmap.md           # Now / Next / Later + target outcome per item
  decisions/           # ADR-style: date, decision, evidence, alternatives, revisit-on
```

Plus: GitHub Discussions with a public "Feature requests" category (👍 +
labels `idea`/`validated`/`building`/`shipped`/`wontfix`) and a CHANGELOG so
shipping closes the loop.

## 5. Cheap validation playbook

Match the method to the riskiest assumption; cheapest first.

| Method | Cost | Use for |
|---|---|---|
| GitHub Discussion + 👍 tally | ~0 | demand signal |
| **Fake door** (disabled button → "notify me") | $ | feature-level demand |
| Smoke-test landing page | $ | whole-concept demand |
| **Concierge MVP** (do it by hand for 1–2 servers) | hours | is the value real |
| **Wizard of Oz** (looks automated, human behind) | $-$$ | experience value |
| Kano survey | low | delight vs must-have |
| 5–10 JTBD interviews | time | problem truth |
| Opt-in telemetry | low | real usage |
| Single-feature MVP | S | riskiest assumption only |
| Paid pilot | $$ | commit vs politeness |

**Rules:** define numeric pass/fail before running; prefer skin-in-the-game
signals over opinions; a *flattening retention curve* beats an interest spike;
follow every signal with one "why"; kill in 2–4 weeks if unvalidated.

---

## 6. Sources (selected)

**Similar projects / panels:** github.com/Bletch1971/ServerManagers ·
github.com/ChronosWS/asma · arkonbot.com · github.com/arkmanager/ark-server-tools ·
github.com/Acekorneya/Ark-Survival-Ascended-Server · github.com/Hermsi1337/docker-ark-server ·
github.com/DiFuks/ark-manager · github.com/Ch4r0ne/ARK-Ascended-Server-Manager ·
github.com/oNdsen/Ark-Mod-Updater · github.com/sanjay-m6/ark-asa-server-manager-2.0 ·
github.com/ArkServerApi/AsaApi · pterodactyl.io · github.com/pelican-dev/panel ·
github.com/pufferpanel/pufferpanel · craftycontrol.com · cubecoders.com/AMP ·
linuxgsm.com · github.com/WindowsGSM/WindowsGSM · github.com/Daziusm/maryl

**GUI vs CLI / hybrid:** clispec.dev/spec/v0.2 · clig.dev · news.ycombinator.com
(26745037, 40322593, 40885635) · reddit.com/r/selfhosted (141lyqj, 1avxab8,
1fv6coe) · v2.tauri.app/concept/architecture · github.com/buYoung/my_supervisor ·
github.com/dodoflix/blkbstr

**Security:** Pterodactyl advisories GHSA-24wv-6c99-f843 (RCE),
GHSA-c479-wq8g-57hr (password in query string), GHSA-g7vw-f8p5-c728 (cross-node
config disclosure) · pterodactyl/panel#3112 (RCON password charset) ·
guildorder.com/games/ark/guides/server-administration

**ARK pain (config/update/cluster):** reddit.com/r/playark (4e4lrc, 4qpyl8,
4jkk6s, 1rwkw8n, 1fboz42, es06kx) · github.com/FezVrasta/ark-server-tools#367 ·
supercraft.host/wiki/ark/asa_workshop_mod_broke_after_update ·
steamcommunity.com/app/2399830 (self-hosting threads)

**Ideation / prioritisation / validation:** producttalk.org/opportunity-solution-tree ·
atlassian.com/agile/product-management/prioritization-framework ·
koji.so/docs/pretotyping · exponentially.com/fake-door-test
