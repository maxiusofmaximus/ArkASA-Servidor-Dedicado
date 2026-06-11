# ARK ASA Config Manager - Architecture

## Overview

This document describes the professional architecture of the ARK ASA Configuration Manager system.

## System Design

```
┌─────────────────────────────────────────────────────┐
│  Desktop Application (Tauri)                        │
├─────────────────────────────────────────────────────┤
│  Frontend Layer                                     │
│  ├── React Components (Navigation, Forms)          │
│  ├── Zustand State (Config, Validation Results)    │
│  └── TypeScript Types (Type Safety)                │
├─────────────────────────────────────────────────────┤
│  Tauri Runtime (IPC Bridge)                        │
├─────────────────────────────────────────────────────┤
│  Backend Layer (Rust)                              │
│  ├── Config Module (Load, Validate, Persist)      │
│  ├── ARK Module (Server Mgmt - Future)            │
│  ├── Storage Module (SQLite - Future)             │
│  └── Error Module (Type-Safe Errors)              │
├─────────────────────────────────────────────────────┤
│  Storage Layer                                      │
│  ├── Disk: config.toml (Primary)                   │
│  ├── Disk: Game.ini + GameUserSettings.ini         │
│  └── SQLite: Audit logs + Version history          │
└─────────────────────────────────────────────────────┘
```

## Module Breakdown

### Frontend (React + TypeScript)

**File Structure:**
```
frontend/src/
├── components/
│   ├── Navigation.tsx        # Tab navigation
│   ├── ConfigForm.tsx        # Form wrapper with save/reset
│   └── ServerStatus.tsx      # Server status sidebar
├── pages/
│   ├── GeneralSettings.tsx   # Identification + Network
│   ├── GameplaySettings.tsx  # Gameplay multipliers
│   ├── ServerSettings.tsx    # World, performance
│   └── AdvancedSettings.tsx  # Advanced options
├── stores/
│   └── configStore.ts        # Zustand global state
├── services/
│   └── api.ts               # Tauri invoke helpers
├── types/
│   └── index.ts             # TypeScript interfaces
└── styles/
    └── index.css            # Tailwind + custom styles
```

**Design Principles:**
- Component-driven: Reusable, testable components
- Single responsibility: Each component does one thing
- Props-based: Minimal prop drilling via Zustand
- Type-safe: Full TypeScript coverage

### Backend (Rust)

**SOLID Application:**

#### 1. Single Responsibility Principle
Each module has ONE job:
- `schema.rs` - Data structures only
- `validator.rs` - Validation only
- `loader.rs` - Reading files only
- `persister.rs` - Writing files only

#### 2. Open/Closed Principle
New validators can be added WITHOUT changing existing code:

```rust
// Add new validator without touching old ones
struct MyNewValidator;
impl ConfigValidator for MyNewValidator {
    async fn validate(&self, config: &ServerConfig) -> Result<()> {
        // Custom validation logic
    }
}

// Compose with existing validators
let validators = CompositeValidator::new()
    .with_validator(Box::new(MyNewValidator));
```

#### 3. Liskov Substitution Principle
All validators implement the same `ConfigValidator` trait:

```rust
#[async_trait]
pub trait ConfigValidator: Send + Sync {
    async fn validate(&self, config: &ServerConfig) -> Result<()>;
    fn name(&self) -> &str;
}
```

#### 4. Interface Segregation Principle
Traits are minimal and focused:
- `ConfigValidator` - Only validation
- `ConfigLoader` - Only loading
- `ConfigPersister` - Only persisting

#### 5. Dependency Inversion Principle
High-level code depends on abstractions, not concrete implementations:

```rust
// Future: Inject different implementations
pub struct ArkServer {
    installer: Arc<dyn Installer>,      // Injected
    process_mgr: Arc<dyn ProcessMgr>,   // Injected
}
```

### Config Module

**Validation Pipeline:**
```
User Input
    ↓
Load TOML/INI
    ↓
Deserialize to ServerConfig
    ↓
Run Validators (Port, Password, Mods, Paths, Multipliers)
    ↓
If Valid → Save to Disk + Generate INI
    ↓
If Invalid → Return Errors to UI
```

**Validators (Composable):**
1. `PortValidator` - Port uniqueness and range
2. `PasswordValidator` - Admin password strength
3. `PlayerCountValidator` - Player count bounds
4. `ModValidator` - Mod ID format and no zeros
5. `PathValidator` - Directory accessibility
6. `MultiplierValidator` - Multiplier ranges

Each validator is independent and can be tested in isolation.

### Error Handling

**Typed Errors:**
```rust
pub enum Error {
    ValidationError(String),
    ValidationFailed { field, message, code },
    ServerNotFound { path },
    ConfigNotFound { path },
    ProcessError(String),
    DatabaseError(sqlx::Error),
    IoError(io::Error),
    // ... more
}
```

**Serializable for UI:**
```rust
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}
```

## Data Flow

### Save Configuration

```
1. User modifies form
   ↓
2. onChange handler → Zustand store
   ↓
3. User clicks "Save"
   ↓
4. Invoke Tauri command: save_config(ServerConfig)
   ↓
5. Backend validates config
   - CompositeValidator runs all rules
   - Returns ValidationResult
   ↓
6. If valid:
   - Serialize to TOML
   - Save to config.toml
   - Generate Game.ini
   - Generate GameUserSettings.ini
   - Log to database
   ↓
7. Return success/error to frontend
   ↓
8. UI shows toast notification
```

### Load Configuration

```
1. App starts
   ↓
2. Invoke Tauri command: load_config()
   ↓
3. Backend:
   - Check if config.toml exists
   - If not → return defaults
   - If exists → load and deserialize
   ↓
4. Return ServerConfig to frontend
   ↓
5. Zustand store receives config
   ↓
6. Components re-render with values
```

## Configuration Schema

### TOML Format

```toml
[identification]
session_name = "My Server"
server_password = ""
admin_password = "MySecurePassword"
server_message_of_the_day = "Welcome!"

[network]
port = 7777
query_port = 27015
rcon_port = 27020
server_platform = "ALL"

[gameplay]
server_pve = true
max_players = 70
difficulty_offset = 2.0
dino_count_multiplier = 2.0

[multipliers]
xp_multiplier = 3.0
taming_speed_multiplier = 15.0
harvest_amount_multiplier = 8.0
baby_mature_speed_multiplier = 40.0
egg_hatch_speed_multiplier = 20.0

[mods]
active_mods = ["955131", "1102729"]

[paths]
steam_cmd_dir = "C:\\ASA\\steamcmd"
server_dir = "C:\\ASA\\server"
```

### INI Generation

The app generates platform-specific INI files:

**Game.ini:**
```ini
[/script/shootergame.shootergamemode]
SessionName=My Server
AdminPassword=MySecurePassword
MaxPlayers=70

[/script/shootergame.shootergamestate]
DinoCountMultiplier=2.0
```

**GameUserSettings.ini:**
```ini
[ServerSettings]
Port=7777
QueryPort=27015

[Multipliers]
XPMultiplier=3.0
TamingSpeedMultiplier=15.0
```

## Testing Strategy

### Unit Tests (Rust)

```rust
#[tokio::test]
async fn test_port_validation_rejects_invalid() {
    let validator = PortValidator;
    let mut config = ServerConfig::default();
    config.network.port = 999;
    assert!(validator.validate(&config).await.is_err());
}
```

### Integration Tests (Frontend)

```typescript
test('can save and load config', async () => {
    // Load defaults
    // Modify values
    // Save
    // Reload
    // Verify
});
```

## Performance Considerations

| Metric | Target | Status |
|--------|--------|--------|
| App Size | < 50MB | ✓ Tauri (5-15MB) |
| Idle RAM | < 30MB | ✓ Optimized |
| UI Response | < 200ms | ✓ React + Zustand |
| Config Save | < 500ms | ✓ Async Tokio |

## Extensibility

### Adding New Configuration Section

1. Add struct to `schema.rs`
2. Add impl Default
3. Create validation logic in `validator.rs`
4. Add loader logic in `loader.rs`
5. Add persister logic in `persister.rs`
6. Create React component in `pages/`
7. Add route to `App.tsx`

No existing code needs modification (OCP).

### Adding New Validator

1. Create struct implementing `ConfigValidator`
2. Add to `CompositeValidator::new()` or inject dynamically
3. Done - no other changes needed

### Migrating from Legacy

Old PowerShell scripts are archived. Config loading supports:
- New TOML format (primary)
- Legacy INI format (compatibility layer)

## Future Enhancements

### Phase 2: Server Management
- Server start/stop/restart
- SteamCMD integration
- Process monitoring

### Phase 3: Advanced Features
- Config version history
- Backup/restore
- Mod management UI
- Multi-server support

### Phase 4: Web Support
- Optional web UI
- Remote management
- API server

## Deployment

### Building

```bash
npm run tauri:build
```

Outputs:
- `src-tauri/target/release/bundle/msi/ARK*.msi` - Windows installer

### Installation

Users simply run the MSI and get a desktop shortcut.

## Debugging

### Rust Backend

```bash
RUST_LOG=debug npm run tauri:dev
```

Check console for logs.

### Frontend

Open DevTools (F12) for React component inspection and network monitoring.

### Logging

Tauri logs to:
- `~\AppData\Roaming\ARK ASA Config Manager\logs\`

## Security Considerations

1. **Passwords** - Never logged or transmitted without encryption
2. **File Access** - Validated paths only
3. **Input Validation** - All user input sanitized
4. **Dependencies** - Regular audits (`cargo audit`, `npm audit`)

## Code Quality

- **Lint:** `cargo clippy`, `npm run lint`
- **Format:** `cargo fmt`, `prettier`
- **Tests:** `cargo test`, `npm run test`
- **Coverage:** Aim for >80% on critical paths

---

**Last Updated:** 2026-06-10  
**Version:** 2.0.0
