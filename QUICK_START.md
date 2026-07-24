# ARK ASA Config Manager - Quick Start Guide (5 minutes)

> ⚡ **TL;DR:** Run `pnpm install && pnpm run tauri:dev` - Done!

## What You Get

✅ Modern desktop UI (Tauri, 5-15MB)  
✅ Full server configuration management  
✅ Type-safe Rust backend with validation  
✅ Auto-generates Game.ini + GameUserSettings.ini  
✅ Server start/stop/restart controls  
✅ Configuration history & audit logs  

## Prerequisites

- ✅ Rust (already installed)
- ✅ Node.js (already installed)
- ✅ Tauri (already installed)

## Step 1: Install Dependencies (1 minute)

```powershell
pnpm install
```

## Step 2: Start Development (1 minute)

```powershell
pnpm run tauri:dev
```

**Wait 2-3 minutes for first compile...**

## Step 3: See It Working (1 minute)

You'll see a window with:
- Tabs: General, Gameplay, Server, Advanced, Status
- Server status panel on the right
- Configuration forms on the left
- Save/Reset buttons at the bottom

## Test It Out

### Test 1: Change Configuration
1. Click **General** tab
2. Change "Session Name" to anything
3. Change "Admin Password" to a strong password
4. Click **Save Configuration**
5. Check: `config.toml` file created ✓

### Test 2: Validate Errors
1. Click **Gameplay** tab
2. Set "Max Players" to `0`
3. Try to save → Shows error ✓
4. Change back to `70` → Error goes away ✓

### Test 3: Explore All Tabs
- **General**: Session, network ports
- **Gameplay**: Server type, 7 multipliers
- **Server**: World settings, mods list
- **Advanced**: 11 toggle options
- **Status**: Server controls & uptime

## Code Organization

```
src/
├── config/           ← Configuration (load, validate, save)
├── ark/             ← Server management (install, start/stop)
├── storage/         ← SQLite (history, audit logs)
└── error.rs         ← Error types

frontend/src/
├── pages/           ← Configuration UI (4 tabs)
├── components/      ← Reusable components
├── stores/          ← Zustand global state
└── types/           ← TypeScript interfaces
```

## Make Changes

### Edit Rust (Backend)
```powershell
# Edit src/config/validator.rs (add new validator)
# or src/ark/server.rs (change server logic)

# Restart dev server to recompile
pnpm run tauri:dev
```

### Edit React (Frontend)
```powershell
# Edit frontend/src/pages/GameplaySettings.tsx
# (just save - hot reloads automatically)
```

## Build for Release

```powershell
pnpm run tauri:build
# Creates: src-tauri/target/release/bundle/msi/ARK*.msi
```

## Documentation

- 📖 [README.md](README.md) - Feature overview
- 🏗️ [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - System design
- 📡 [docs/API.md](docs/API.md) - All Tauri commands

## Troubleshooting

**App won't start?**
```powershell
rm -r src-tauri/target  # Clear cache
pnpm run tauri:dev       # Rebuild
```

**Rust compilation error?**
```powershell
rustup update           # Update toolchain
cargo clean             # Clean build artifacts
pnpm run tauri:dev       # Try again
```

**Port already in use?**
```powershell
netstat -ano | findstr :7777
taskkill /PID <PID> /F
```

## What's Next?

✅ Phase 1: Foundation (Config + Basic UI)  
✅ Phase 2: Backend Integration (ARK module + Storage)  
⏳ Phase 3: Advanced UI (Config preview, backup, logs)  
🔮 Phase 4: Production Ready (Web variant, API, scaling)

## Pro Tips

1. **Ctrl+Shift+I** = DevTools in Tauri window
2. **F5** = Reload app
3. **Set RUST_LOG=debug** for verbose logging
4. Check `~\AppData\Roaming\ARK ASA Config Manager\logs\` for debug logs
5. Read code comments in `src/` for architecture details

---

**That's it! You're ready to go. Happy configuring! 🚀**

Generated: 2026-06-10  
Version: 2.0.0 (Phase 2)
