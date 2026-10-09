# `ark-config` — Operator CLI

**Status: IMPLEMENTED** (was "NOT IMPLEMENTED in v2.1.0"). The
`skeleton in `src-tauri/src/cli.rs` was deleted in `0fc9933`; this is a real
binary, `src-tauri/src/bin/ark-config.rs`, wired to the same dispatcher the
desktop UI, the loopback HTTP API and the chat bots use.

## What it is

A standalone console front-end for the ARK ASA Configuration Manager. It does
**not** need the desktop app to be running and it makes no network round-trip:
it loads `server-config.toml` from disk and calls `integrations::dispatch`
directly — the single source of truth for every command in the product.

## Build

```bash
cd src-tauri
cargo build --release --bin ark-config
# → target/release/ark-config.exe   (Windows)
# → target/release/ark-config       (Linux/macOS)
```

The GUI binary (`ark-asa-config`) and the CLI share the same lib; a plain
`cargo build --release` builds both.

## Usage

```
ark-config [--config <path>] [--server <id>] <command> [options]

  status                 Cluster + per-map running state
  start  [--map N]       Start the whole cluster, or one map instance by index
  stop   [--map N]       Gracefully stop the whole cluster, or one map
  restart                Restart the whole cluster
  logs   [--tail N]      Last N lines of ShooterGame.log (default 20)
  ip                     Configured connection addresses
  config                 Print the effective TOML configuration
  maps                   List map index / id from the effective cluster
  validate               Run the built-in + custom validators on the config
  drift                  Compare the on-disk INIs against the config (exit 1 on drift)
  update [--check]       Verified ARK server update via steamcmd
  servers                List server profiles (the active one is marked *)
  presets                List saved config presets
  preset-save <name>     Save the active config as a preset
  preset-apply <name>    Write a preset into the active profile config
  preset-delete <name>   Delete a preset

Global:
  --server <id>, -s      Target a specific server profile (else the active one)
  --config <path>, -c    Explicit config file (overrides profiles)
  -h, --help             Show this help
  -V, --version          Show the version
```

A numeric positional is also accepted, so `ark-config start 1` is equivalent to
`ark-config start --map 1`.

### Examples

```bash
# What is running right now?
ark-config status

# Start only TheIsland (map index 0) instead of the whole cluster
ark-config start 0

# Gracefully stop it (RCON saveworld + doexit)
ark-config stop 0

# Tail the server log
ark-config logs --tail 100

# Use a non-default config
ark-config -c D:\servers\prod.toml status
```

## Multi-server profiles

The app manages several servers via a profile registry at
`%APPDATA%\ARK ASA Config Manager\profiles.json`. The legacy single
`server-config.toml` is adopted as the `default` profile on first run (nothing
is moved). Each profile stores its own config at `servers/<id>/server-config.toml`
and therefore its own saves, ports, mods, cluster and passwords.

```bash
ark-config servers              # list profiles (* marks the active one)
ark-config --server pvp status  # act on the "pvp" profile
```

Switching the active profile in the desktop UI also re-points
`ARK_ASA_CONFIG_PATH`, so the chat / loopback / Convex router follows.

## Config resolution

1. `--config <path>` / `-c <path>` if given (explicit override);
2. `--server <id>` / `-s <id>` → that profile's `server-config.toml`;
3. otherwise the **active** profile from `profiles.json`;
4. if the registry is unavailable, `$ARK_ASA_CONFIG_PATH`, then the legacy
   `%APPDATA%\ARK ASA Config Manager\server-config.toml` (Windows).

## Validation

`ark-config validate` runs the six built-in validators **plus** your declarative
custom validators (`%APPDATA%\ARK ASA Config Manager\custom_validators.toml`),
printing each violation and exiting `1` when the config is invalid. Great for CI
and pre-deploy checks. See [CUSTOM_VALIDATORS.md](CUSTOM_VALIDATORS.md).

```bash
ark-config validate && ark-config start 0
```

## Verified updates

`ark-config update --check` prints the local vs remote Steam build id.
`ark-config update` refuses to run while the server is up, runs the steamcmd
update, then **re-checks the build id** and exits `1` if the build did not move
(steamcmd occasionally reports success without applying the update). Ideal for
a nightly cron that only updates when idle.

```bash
ark-config update --check    # report only
ark-config update            # apply + verify (server must be stopped)
```

> The **app's own** signed self-updater (tauri-plugin-updater with a release
> feed + pubkey) is not wired yet — it needs the operator's signing keys and
> update endpoint. Tracked in `RESEARCH_NEW_IDEAS.md` / `OPEN_WORK.md`.

## Drift & presets

`ark-config drift` compares what the app *would* write to `Game.ini` /
`GameUserSettings.ini` against what is actually on disk and exits `1` when they
diverge — useful in cron to catch ARK (or another tool) rewriting your config.
`ark-config presets` manages shareable whole-server config templates in
`<config_dir>/presets/`.

```bash
ark-config drift                 # exit 0 = clean, 1 = drift
ark-config preset-save pvp-2x    # snapshot the active config
ark-config preset-apply pvp-2x   # write a preset into the active profile
ark-config presets               # list
```

## Notes & caveats

- Exit codes: `0` on a completed dispatch, `1` on a dispatch error, `2` on a
  usage error. Note that `start`/`stop` of the **whole cluster** return the
  cluster summary through `RouterOutcome::Error` (an upstream quirk where the
  success message rides the `Error` variant) — the CLI prints it and still
  exits `0`, matching what the chat bots show.
- `admin_password` / `server_password` are **not** passed on the ARK launch
  URL. ASA mangles them there (see `CHANGELOG.md` → [Unreleased]); the CLI (via
  `bridge::ensure_ini`) regenerates `GameUserSettings.ini` right before launch,
  which is where the server reads them.
- The other operator channels remain available and share this same dispatch
  path: desktop UI, loopback HTTP API (`127.0.0.1:8765`, bearer-authenticated),
  the chat-bot adapters (Telegram / Discord / Slack / WhatsApp / WeChat /
  Signal), the Vercel web admin, and the Convex publisher.
