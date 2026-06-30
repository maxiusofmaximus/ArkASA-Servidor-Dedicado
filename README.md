# ARK ASA Configuration Manager

**Professional Server Configuration UI for ARK: Survival Ascended · v2.1**

[![Release](https://img.shields.io/github/v/release/maxiusofmaximus/ArkASA-Servidor-Dedicado)](https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A modern, desktop application built with **Rust + Tauri + React** for managing ARK Survival Ascended dedicated servers with an intuitive, game-like interface inspired by the official ARK UI.

## ✨ Features

- **Modern Desktop UI** - Tauri-based application (5-15MB, ultra-lightweight)
- **Cyan/Purple Theme** - Matches ARK Survival Ascended aesthetic
- **Type-Safe Configuration** - Rust backend with full validation
- **Hot Reload** - Changes apply without server restart (where possible)
- **Config Export/Import** - TOML format, human-readable
- **Automatic INI Generation** - Generates Game.ini & GameUserSettings.ini
- **Extensible Architecture** - SOLID principles throughout
- **Remote Admin via Convex + Vercel (v2.1)** - 8-channel admin: web, REST, Telegram, Discord, WhatsApp, Signal, WeChat, SSH
- **Internet-Gated Server Start (v2.1)** - Prevents silent crashes from offline boots
- **Fixed-by-Map Port Assignment (v2.1)** - Each cluster map always lands on the same triplet

## 🚀 Quick Start

### ⚡ Super Quick (30 seconds)

1. Download installer from [GitHub Releases](https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases)
2. Run the `.exe` file
3. Follow on-screen prompts
4. Open the app and start configuring!

### 📚 Complete Guide

**First time? Start here:**

👉 **[GETTING_STARTED.md](docs/GETTING_STARTED.md)** - Step-by-step guide for beginners (30+ pages, explains everything)

**All documents at a glance:**
- [📖 Documentation Index](docs/INDEX.md) - Choose your learning path
- [🔗 Network & Connection Guide](docs/STEAM_A2S.md) - How to connect locally or remotely
- [🎮 Complete User Guide](docs/USER_GUIDE.md) - All features explained
- [❓ FAQ](docs/FAQ.md) - 50+ answered questions
- [🔧 Troubleshooting](docs/TROUBLESHOOTING.md) - Solutions to common problems

### Build from Source

#### Prerequisites

1. **Rust** (1.80+) - [Install](https://rustup.rs/)
2. **Node.js** (18+) - [Install](https://nodejs.org/)

#### Installation

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm run tauri:dev

# Build for release
pnpm run tauri:build
```

### First Time Setup

1. Run the application
2. Go to "General" tab
3. Configure:
   - Session Name
   - Admin Password (REQUIRED - change from default)
   - Network ports
4. Click "Save Configuration"
5. Server is ready to start

## 📁 Project Structure

```
ArkASA-Servidor-Dedicado/
├── src-tauri/src/               # Rust backend
│   ├── config/                  # Configuration module
│   │   ├── schema.rs           # Data structures + ConnectionMethod enum
│   │   ├── validator.rs        # Validation logic (OCP)
│   │   ├── loader.rs           # Load from TOML/INI
│   │   └── persister.rs        # Save to disk + INI generation
│   ├── ark/                    # Server management
│   │   ├── launcher.rs         # CLI argument builder (single source of truth)
│   │   ├── server.rs           # Start/stop/restart lifecycle
│   │   ├── rcon.rs             # RCON client
│   │   ├── logs.rs             # Log streaming
│   │   └── metrics.rs          # Performance metrics
│   ├── storage/                # SQLite audit log + version history
│   ├── stub.rs                 # On-demand dormant server stubs
│   ├── backup.rs               # Cloud backup (S3, GDrive, OneDrive, iCloud)
│   ├── error.rs                # Error types
│   ├── lib.rs                  # Tauri commands
│   └── main.rs                 # Entry point
│
├── frontend/src/                # React + TypeScript frontend
│   ├── components/              # Reusable components
│   ├── pages/                   # Tab pages
│   ├── stores/                  # Zustand global state
│   ├── services/                # API calls to Tauri
│   ├── types/                   # TypeScript types
│   ├── App.tsx                  # Main app
│   └── main.tsx                 # React entry
│
├── docs/                        # Documentation (GitHub Pages)
├── scripts/                     # Utility scripts (DuckDNS, firewall)
├── installer/                   # NSIS full installer
├── migrations/                  # Database migrations
├── Cargo.toml                   # Rust dependencies
└── package.json                 # Node dependencies
```

## 🏗️ Architecture

### Backend (Rust)

**SOLID Principles:**
- **S**ingle Responsibility - Each module has one job
- **O**pen/Closed - Validators are composable via traits
- **L**iskov Substitution - All validators implement ConfigValidator
- **I**nterface Segregation - Minimal, focused trait interfaces
- **D**ependency Inversion - DI through constructor injection

**Modules:**
- `config` - Configuration loading, validation, persistence
- `ark` - Server lifecycle management (start/stop/restart, RCON, process monitoring, logs, metrics, on-demand stubs)
- `backup` - Cloud backup system (S3, Google Drive, OneDrive, iCloud)
- `storage` - SQLite for audit logs and version history
- `error` - Typed error handling

### Frontend (React + TypeScript)

**Tech Stack:**
- **React 19** - Modern, concurrent rendering
- **TypeScript** - Full type safety
- **Tailwind CSS** - Utility-first styling
- **Zustand** - Lightweight state management
- **Tauri** - Desktop integration

**Design System:**
- Colors: Cyan (#00d4ff), Purple (#9d4edd), Dark (#0a0e27)
- Responsive grid layout
- Keyboard-accessible components

## 🔧 Configuration

### TOML Format (Primary)

```toml
[identification]
session_name = "My ARK Server"
admin_password = "SecurePassword123"

[network]
port = 7777
query_port = 27015

[gameplay]
server_pve = true
max_players = 70
dino_count_multiplier = 2.0

[multipliers]
xp_multiplier = 3.0
taming_speed_multiplier = 15.0
```

### Generated INI Files

The app auto-generates `Game.ini` and `GameUserSettings.ini` based on TOML config. These are placed at:
- `C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\Game.ini`
- `C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini`

## ✅ Validation

All configuration changes are validated before saving:

1. **Port Validation** - Unique ports in valid range (1024-65535)
2. **Password Validation** - Not default, minimum length 4
3. **Mod Validation** - No empty IDs, no "0", numeric only
4. **Multiplier Validation** - Positive numbers, sensible ranges
5. **Path Validation** - Must point to accessible directories

Add custom validators without modifying existing code (OCP principle).

## 📝 Development

### Running Tests

```bash
# Rust tests
cargo test

# Frontend tests
pnpm run test
```

### Code Quality

```bash
# Lint Rust
cargo clippy

# Format code
cargo fmt
pnpm run lint
```

### Building for Release

```bash
pnpm run tauri:build
# Generates MSI installer in src-tauri/target/release/bundle/msi/
```

## 🔄 Legacy Migration

Old PowerShell scripts and documentation have been archived in `/archive/`.

If you need the old files:
- Scripts: `/archive/legacy-scripts/`
- Docs: `/archive/legacy-docs/`
- Config examples: `/archive/legacy-config/`

## 📖 Documentation

### 🎯 User Documentation (Start Here!)

- **[INDEX.md](docs/INDEX.md)** - Documentation hub & navigation guide
- **[GETTING_STARTED.md](docs/GETTING_STARTED.md)** - Complete beginner's guide (installation, setup, first server)
- **[STEAM_A2S.md](docs/STEAM_A2S.md)** - Network connectivity guide (local & remote, port forwarding, Tailscale)
- **[NETWORK_SETUP.md](docs/NETWORK_SETUP.md)** - Router setup, static IP, DuckDNS, firewall, RCON security
- **[USER_GUIDE.md](docs/USER_GUIDE.md)** - Complete feature reference (all tabs, settings, commands)
- **[FAQ.md](docs/FAQ.md)** - Frequently asked questions (50+ Q&A)
- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Problem solving guide (10+ common issues)

### 🔧 Technical Documentation

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design, principles, patterns
- **[API.md](docs/API.md)** - Tauri command reference
- **[CONTRIBUTING.md](docs/CONTRIBUTING.md)** - Development guidelines

## ✅ Current Status

**v1.0 Release** - Core configuration management complete:
- ✅ Desktop application with game-inspired UI
- ✅ Configuration loading/saving (TOML format)
- ✅ Automatic INI generation
- ✅ Comprehensive validation
- ✅ Type-safe Rust backend
- ✅ Full documentation

## ✅ v1.1 Complete Feature Set

**Server Management:**
- ✅ Server start/stop/restart with graceful RCON shutdown
- ✅ Real-time server status monitoring & log streaming
- ✅ Server crash detection & auto-recovery
- ✅ On-demand server stubs (dormant servers)

**Configuration:**
- ✅ Config version history ready
- ✅ AutoSave configuration feature
- ✅ Full validation pipeline
- ✅ TOML config format

**Mods & Content:**
- ✅ Mod management UI with CurseForge integration
- ✅ PC-only mod detection
- ✅ Duplicate mod prevention

**Backup & Cloud:**
- ✅ Multi-provider cloud backup (S3, Google Drive, OneDrive, iCloud)
- ✅ Backup/restore functionality
- ✅ Multiple backup scopes

**Documentation:**
- ✅ 150+ pages of user guides
- ✅ GitHub Pages documentation site
- ✅ 50+ FAQ answers
- ✅ Comprehensive troubleshooting

## ✅ v1.2 — On-Demand Notifications

- ✅ Tauri events `on-demand-waking` / `on-demand-ready` emitted to frontend
- ✅ Auto-dismiss 30s banner: DORMIDO → INICIANDO → listo

## ✅ v1.3 — Network Guide & Connection Manager

- ✅ `docs/NETWORK_SETUP.md`: complete router/network guide (port forwarding, Movistar quirks, DuckDNS, firewall, RCON security)
- ✅ `scripts/firewall-cleanup.ps1` and `scripts/duckdns-updater-template.ps1`
- ✅ Connection Manager: multi-method IP selector (Tailscale / Public IP / DuckDNS / Local / Manual)
- ✅ Friend Contacts: address book with per-contact ping keep-alive

## 🛠️ Roadmap (Future Versions)

- [x] Web UI option (in addition to desktop) — shipped in v2.1
- [x] Auto-detect public IP and Tailscale IP on startup — shipped in v2.0
- [ ] Advanced performance analytics dashboard
- [ ] Custom plugin system for validators
- [ ] Mobile companion app (React Native, shares @ark-asa/shared-types)

## 📄 License

MIT

---

**Built with ❤️ using Rust, Tauri, and React**
