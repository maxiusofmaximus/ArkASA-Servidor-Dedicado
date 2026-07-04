# Self-Hosting on Your Own Hardware (alpha toward v2.1)

> Skip the cloud entirely. Run ARK ASA on a Raspberry Pi 5, an old
> Intel NUC, or a spare Windows 10 PC you have lying around. The
> desktop app emits a platform-tailored bash and a stage-by-stage
> checklist. This guide explains how to actually use that output on
> three concrete device classes.

---

## The unified flow (any device class)

1. **Open desktop app → Options → Hosting → "Run on your own
   hardware".**
2. **Pick** your hardware class:
   - Raspberry Pi 5 (Debian Bookworm arm64)
   - Debian 12 / 13 minimal (Intel NUC, x86 server)
   - Ubuntu Server 24.04 (Intel NUC, x86 server)
   - Windows 10/11 + WSL2 Debian
   - Windows 10/11 + WSL2 Ubuntu
   - Apple Silicon Mac (M1/M2/M3/M4)
   - Intel-based Mac (Intel x86_64)
3. **Paste the backup bundle URL** in the same field as for cloud
   VPS deploys. This is the same `.zip` the cloud-init would have
   downloaded; locally we just curl it instead.
4. **Click GENERATE LOCAL PLAN.** Output panes:
   - **Inline one-liner** — the operator-friendly `curl … | sudo bash`.
   - **Bundled bash script** — long-form, preferred. Same contents,
     but easier to inspect before running.
   - **Stage-by-stage checklist** — what each step is, what stdout
     should contain when it worked.
5. **Save the bundled bash** as `run.sh` on the target device. Run
   it as the user specified in the inline one-liner (Linux/WSL2:
   `sudo bash run.sh`; macOS: `bash run.sh` from your home dir).
6. **Wait ≈8 minutes** for SteamCMD to download + ARK ≃28 GB
   install. The script will then enable the systemd unit but the
   service won't be reachable from Steam's server browser until
   **the device's port 7777 is reachable** from the public
   internet (see [Network & Tailscale wizard](./NETWORK_SETUP.md) —
   blocker #4 of v2.1.0).

---

## Device Playbook 1 — Raspberry Pi 5 (Debian Bookworm arm64)

### Hardware you'll need

- Pi 5 (4 GB RAM min, 8 GB recommended if you want cluster >2 maps).
- An **active cooling fan**. The Pi 5 thermal-throttles under ARK's
  load within seconds without one.
- 64 GB U3 microSD card (for OS) **OR** USB 3.0 SSD (much better;
  SteamCMD writes ≈28 GB).
- Realtek / Intel GbE adapter over the built-in Broadcom Ethernet
  is recommended for ARK's UDP bandwidth.

### OS install

1. Flash **Raspberry Pi OS Bookworm 64-bit Lite** with the Raspberry
   Imager (`https://www.raspberrypi.com/software/`).
2. Pre-configure in Imager:
   - Hostname: `arkasa-pi5`
   - Enable SSH *(password or key — your choice)*
   - Locale / Timezone / WiFi country code
   - **DO NOT** set a custom user, leave default `pi`; we'll use
     `sudo` for everything below.
3. Boot. SSH in.

### What the operator does

```bash
# 1. Get the bundled bash from the desktop app — paste it into a file.
nano run.sh        # paste from the HostingTab output

# 2. Run it.
chmod +x run.sh
sudo bash run.sh
# (the script does `set -e`, so any failure stops. Watch each stage.)

# 3. Wait ≈8 minutes for SteamCMD + ARK download.

# 4. Verify systemd took over:
systemctl is-active arkasa        # → active
journalctl -u arkasa -f           # → ARK server starting
ss -ulnp | grep 7777              # → UDP 7777 listening
```

### Hardware validation (operator checklist)

- [ ] Pi 5 fan spinning at boot
- [ ] `vcgencmd measure_temp` reads < 70 °C under load
- [ ] `journalctl -u arkasa` shows `ARK server listening on UDP 7777`
- [ ] From a **different device on the same LAN**, you can
      `nmap -sU -p 7777 arkasa-pi5.lan` → `open`
- [ ] Steam server browser shows the server within 2 minutes
      *if* port 7777 is reachable from the internet

### Known Pi 5 gotchas

- **Bookworm 32-bit** won't run ARK ASA (~3 GB resident memory
  requirements). Always the **arm64** image.
- **active cooling** is non-negotiable. Pi 5 hits 95 °C in 30 s
  without it and downclocks.
- SteamCMD runs as `arkasa` (created by the script). If you run
  `passwd arkasa`, set a strong password even if SSH is key-only.

---

## Device Playbook 2 — Intel NUC / x86 server (Debian 12 or Ubuntu 24.04)

### Hardware you'll need

- x86_64 mini-PC (Intel NUC, Beelink, or old workstation).
- 4 GB RAM minimum, 8 GB recommended if you want a 3-map cluster.
- 64 GB SSD minimum, 120 GB recommended.

### OS install (Debian 13 "Trixie" netinstall)

1. Download `debian-13.x.x-amd64-netinst.iso`.
2. Boot from USB stick.
3. At the `tasksel` screen, uncheck everything except
   `SSH server` and `standard system utilities`.
   **Disable `desktop environment`** — you'll save ≈2 GB.
4. Reboot. `ssh user@<host>`.

### What the operator does

```bash
# Same as Pi 5: paste the bundled bash from the desktop app into run.sh,
# then run:
sudo bash run.sh

# Stage-by-stage follow-up when something fails:
journalctl -u arkasa -f
sudo -u arkasa /home/arkasa/steamcmd/steamcmd.sh +login anonymous \
    +force_install_dir /home/arkasa/server +app_update 2430930 validate +quit
```

### Hardware validation (operator checklist)

- [ ] `systemctl is-active arkasa` shows `active`
- [ ] `journalctl -u arkasa -n 50 --no-pager` does **not** show
      `Out of memory`
- [ ] `df /home/arkasa` shows at least 12 GB used (SteamCMD cache)
- [ ] Top shows `ShooterGameServer` running with ≈3 GB RSS
- [ ] Steam server browser shows the server

### Known x86 gotchas

- Debian 12 (Bookworm) and Debian 13 (Trixie) **both** work; the
  script depends only on `lib32gcc-s1 lib32stdc++6 libc6-i386`,
  which exist in both.
- On **Ubuntu 22.04 LTS** the script also works unaltered.
- Disable `snapd` (Ubuntu Server) — it can spike RAM post-boot.

---

## Device Playbook 3 — Windows 10/11 + WSL2

### Hardware you'll need

- Any 64-bit Windows 10 (build 19041+) or Windows 11 PC.
- ≥ 8 GB total system RAM (WSL2 reserves 2 GB by default; we'll bump).
- ≥ 120 GB free disk space.

### One-time Windows prep

```powershell
# Run **PowerShell as Administrator** on Windows.
# 1. Enable WSL2 + Virtual Machine Platform.
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart
shutdown /r /t 0

# 2. Install WSL2 default.
wsl --update
wsl --set-default-version 2

# 3. Install a distro (pick whichever you prefer — Debian or Ubuntu).
wsl --install -d Debian    # OR: wsl --install -d Ubuntu
```

### Configure systemd inside WSL2 (required!)

```bash
# Inside the WSL2 distro:
sudo nano /etc/wsl.conf
```

Append:

```ini
[boot]
systemd=true
```

Then **on the Windows side**:

```powershell
wsl --shutdown
wsl                 # restarts with systemd
```

Inside WSL2 verify:

```bash
systemctl list-units --type=service --no-pager | head
```

### Rebalance RAM

WSL2 uses `.wslconfig` on the Windows side:

```powershell
notepad "$env:UserProfile\.wslconfig"
```

```ini
[wsl2]
memory=6GB
processors=4
swap=2GB
```

```powershell
wsl --shutdown
wsl
```

### What the operator does

```bash
# 1. Get the bundled bash from the desktop app. **Save it as run.sh on the
# Windows filesystem** (the script reads /tmp/arkasa-bundle.zip via curl so
# the path doesn't matter).

# 2. Open the WSL2 distro. If using Debian: `wsl -d Debian`.
# If using Ubuntu: `wsl -d Ubuntu`.

# 3. Paste run.sh contents, then:
sudo bash run.sh

# 4. Wait ≈8 minutes for SteamCMD + ARK install.
```

### Hardware validation (operator checklist)

- [ ] `systemctl is-active arkasa` returns `active` inside WSL2
- [ ] `journalctl -u arkasa -n 50 --no-pager` shows ARK startup
- [ ] Port forward Windows 10 → WSL2 for UDP 7777 (see below)
- [ ] From another machine on the LAN:
      `Test-NetConnection -ComputerName <win-ip> -Port 7777 -Udp` →
      `True`
- [ ] Steam server browser shows the server (after port-forward)

### Port-forward Windows firewall to WSL2

```powershell
# As Administrator:
netsh interface portproxy add v4tov4 listenport=7777 listenaddress=0.0.0.0 \
   connectport=7777 connectaddress=<WSL-IP>

# (Get the WSL-IP with `wsl hostname -I`)

# Firewall inbound rule:
New-NetFirewallRule -DisplayName "ARK ASA UDP 7777" -Direction Inbound \
   -LocalPort 7777 -Protocol UDP -Action Allow
```

### Known WSL2 gotchas

- **systemd must be explicitly enabled** in `/etc/wsl.conf`; older
  guides show `service start arkasa` but that doesn't auto-restart.
- The bundled bash emits a **WARN** if `systemd is-active arkasa`
  fails; in that case fall back to `sudo -u arkasa bash -c 'cd
  /home/arkasa/server && ./ShooterGame/.../ShooterGameServer'`.
- WSL2's `/mnt/c` paths are slow. Keep `run.sh` inside `~/run.sh`
  (the Linux filesystem), and copy the bundle into `~/bundle.zip`
  first if you need fast reads.

---

## Apple Silicon (macOS) and Intel Mac notes

The script auto-rewrites itself for macOS:
- `apt-get` → `brew install` (with auto-bootstrap if Homebrew missing).
- `/home/arkasa` → `$SERVER_HOME` (so `~/server`).
- `systemctl ... arkasa.service` → `screen -dmS arkasa …` and logs to
  `/var/log/arkasa.log`.

ARK ASA on macOS **runs but is not officially supported by Studio
Wildcard**. Expect:
- Unstable memory after 6 hours of operation — restart manually.
- Steam browser visibility drifts; ARK .ini port + query-port bound.
- ARC-CPU/architecture warnings in stdout; non-blocking.

Use macOS **only as a last-resort personal test rig**, not as your
real game server.

---

## After the script finishes

- `systemctl status arkasa` (Linux/WSL2) or `screen -ls | grep arkasa`
  (macOS) shows the daemon.
- `journalctl -u arkasa -f` → reads logs in real time.
- Port **7777/UDP** is open to the LAN.
- For public Steam browser visibility you ALSO need:
  - A **public IPv4** (Pi 5 / NUC rely on CGNAT bypassing — see
    `docs/NETWORK_SETUP.md`).
  - **Or Tailscale** if your ISP gives you CGNAT (this is the
    blocker #4 of v2.1.0 we still need to ship — until that lands,
    a Pi 5/NUC/WSL2 server is LAN-only).

---

## Where this lands in CHANGELOG and TODO

- **CHANGELOG § "Unreleased → Self-host on Pi / NUC / WSL2 / macOS"**
  marks the third v2.1.0 blocker from the **Open work** list as
  shipped.
- **TODO.md** notes this commit as **Sesión 4**. The remaining item
  is the **Network / Tailscale wizard**, blocker #4 of v2.1.0.
