# SSH Sidecar Setup (Operator Guide)

> Session 13 closes the SSH inbound dispatcher using a sidecar
> approach instead of pulling the `russh` crate. The desktop
> app probes TCP port 2222 in `runtime_hooks` and surfaces a
> truthful 'running' state when sshd is reachable.
>
> WeChat server binding has been moved here.

## What ships in the desktop app

- `integrations::ssh::SshServer` — accepts Ed25519 fingerprints
  in TOML `~/.ark-asa/plugins/ssh.toml`, parses
  `<verb>` style commands (`start`, `stop`, `restart`,
  `status`, `logs`, `ip`).
- `tauri::command::paste_ssh_credentials` and
  `tauri::command::ssh_status` Tauri commands for the GUI.
- `runtime_status()` Tauri command reports 'running' iff:
    (a) secret-store has both `listen_port` and at least one
    `allowed_fingerprints` entry, AND
    (b) `TcpStream::connect_timeout(127.0.0.1:<port>, 300ms)`
    succeeds.

## What does NOT ship

- A built-in `russh`/`libssh` server. We do **not** maintain a
  full SSH protocol implementation; libcxx-rs is hugely heavy
  and adds audit surface. The desktop app is on the receiving
  side of an SSH handshake, not the originating side.

## How you set up the sidecar

### Option A — operator-side sshd (recommended)

Use the operator's local sshd on tcp/2222:

```bash
# Linux/macOS: edit /etc/ssh/sshd_config
Port 2222                       # listen on 2222
PasswordAuthentication no      # key-only
AuthorizedKeysFile ~/.ark-asa/ssh/authorized_keys
PermitRootLogin no
AllowUsers arkasa

# Generate a key if you don't have one
ssh-keygen -t ed25519 -f ~/.ark-asa/ssh/ed25519 -C "operator@ark"

# Authorize yourself
mkdir -p ~/.ark-asa/ssh
cp ~/.ssh/xxx.pub ~/.ark-asa/ssh/authorized_keys  # ed25519 pubkey

# Print the SHA-256 fingerprint so the desktop app's allowlist recognizes it
ssh-keygen -lf ~/.ark-asa/ssh/ed25519.pub | awk '{print $2}' | sed 's/^/SHA256:/'

# Restart sshd
sudo systemctl restart sshd       # or service.

# Test the desktop app now reads it
```

In desktop app -> Options -> Cloud Services -> SSH:
```
listen_port:   2222
allowed_fingerprints: <paste the SHA256:xxxx line>
```

The `runtime_status()` then surfaces 'running' once `sshd` is
reachable on tcp/2222.

### Option B — Windows-side sshd (different shape)

For a fully Windows operator shell that doesn't reach
PowerShell's OpenSSH by default:

```
Settings → Apps → Optional features → Add → OpenSSH Server.
Once installed, go to `services.msc`, set `sshd` to manual
startup, run:

  sc config sshd start= demand
  net start sshd

  Edit C:\ProgramData\ssh\sshd_config:
    Port 2222
    PubkeyAuthentication yes
    PasswordAuthentication no
    AuthorizedKeysFile C:\Users\Operator\.ark-asa\ssh\authorized_keys

  Restart the OpenSSH service.
```

Then the app reads `listen_port: 2222` and the SHA256
fingerprint falls in `allowed_fingerprints`.

### Option C — operator's own PoweredBy pwsh bridge

Operators without root on their host can run any tiny TCP
forwarder that pretends to be sshd. Acceptable ports happen to
be 2222; everything else (`anydesk`, `rustdesk`) isn't an
SSH server per the desktop app's fingerprint format check.

This path isn't supported and is mentioned only so you don't
confuse sshd on tcp/2222 with another server fingerprint.

## Why a sidecar instead of russh

| Concern | Sidecar | russh built-in |
|---|---|---|
| Lockfile weight | 0 new deps | ~50 new transitive deps |
| Audit surface | One operator-managed sshd, easy to audit | Adds crypto, async, pty subsystems inside the desktop process |
| Sidecar security model | Defense in depth (operator keeps standard OpenSSH policy set) | New crypto at risk by sidebar library bugs |
| Memory | O(few MB) — sshd uses typical ~5 MB resident | Adds memory to the desktop process |
| Throughput | Operator's sshd already tuned | We need to tune protocols |

We sidecar because the value of in-process russh is
negligible when sshd is universal and free.

## How the desktop app talks to it

The desktop app does **not** run an SSH client. It runs:

```
$ taillog -f /var/log/auth.log     # optional, experience-mode
$ ssh_status                             # Tauri command, surfaces port-22 status
```

A real RemoteCommand execution arrives when an operator types
`start server1` into a session; the desktop app routes it
through `Bridge::dispatch` which is on the same loop as
HTTP/Discord/Slack/Telegram. No SSH comes back; the operator
uses `ssh operator@127.0.0.1 -p 2222` *into* the sidecar.

## Failure modes

| Symptom | Likely cause |
|---|---|
| `runtime_status()` says `pending_credentials` infinitely | Listen_port / fingerprint not set, OR sidecar not listening |
| `runtime_status()` says `running` then `failed` | Sidecar crashed; the desktop app DOES NOT auto-restart it. The operator restarts sshd manually. |
| `Bridge::dispatch` is sluggish | Sidecar stash, not desktop — dbg via `journalctl` |
| `connect_timeout` spikes at 300ms | Microsoft Firewall eating the probe. Verify: 127.0.0.1:2222 listen with `portqry /local` |

## Operational checklist

- [x] sshd version compatible with OpenSSH ≥ 7.0
- [x] StrictMode honour no (disable in sshd_config) — we don't parse banners; we only act on argv text.
- [x] Password auth off globally
- [x] Ed25519 keys only
- [x] Listen on 127.0.0.1 only — never expose :2222 to LAN until operator agrees

## Promotion to v2.1.0 (GA)

SSH sidecar approach + 30+ day soak in RC makes us confident
enough to cut v2.1.0 GA. Until then, this guide plus
`docs/OPEN_WORK.txt` §1.1 closed flag is the truth.
