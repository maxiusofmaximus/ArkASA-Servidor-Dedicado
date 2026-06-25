# SOLID Principles for Tauri + React + Zustand Desktop Apps

## Overview

This document captures the refactoring patterns, design decisions, and SOLID principle applications discovered while evolving an ARK ASA dedicated server management tool built with **Tauri 2 (Rust)** + **React 19** + **Zustand** + **TailwindCSS** + **Vite**.

## SRP — Single Responsibility Principle

### Pattern: Extract Monolithic Components into Focused Sub-Components

**Problem**: `OptionsModal.tsx` was ~600 lines handling general settings, backup config, S3/GDrive/OneDrive auth, config INI editing, and tab switching.

**Solution**: Decompose by responsibility boundary:

| File | Responsibility |
|------|---------------|
| `components/options/GeneralTab.tsx` | Language, close behavior, on-demand, cluster delay, manual save, logs |
| `components/options/BackupTab.tsx` | Provider selection + cloud config + backup/restore UI |
| `components/options/ConfigTab.tsx` | Thin wrapper around `RawConfigViewer` for the config tab |
| `OptionsModal.tsx` | Tab switching shell only (~60 lines) |

**Guideline**: A component file should own one "screen region" concept. If a tab within a modal has its own state, effects, and async operations → extract it.

### Pattern: Extract Domain Logic into Service Modules

**Problem**: `RawConfigViewer.tsx` contained ~230 lines of pure INI-generation functions mixed with React rendering code.

**Solution**: Move to `services/configGenerators.ts` — pure functions with zero React dependency.

```ts
// services/configGenerators.ts — no React import, fully testable
export function generateGameUserSettings(config: ServerConfig): string { ... }
export function generateGameIni(config: ServerConfig): string { ... }
export function generateToml(config: ServerConfig): string { ... }
```

**Guideline**: If a function doesn't use hooks, JSX, or component state → it belongs in `services/` or `utils/`, not in a component file.

### Pattern: Extract Reusable UI Primitives

**Problem**: `Section`, `Toggle`, `Field` patterns duplicated across `OptionsModal` and would be needed in future settings screens.

**Solution**: `components/ui/OptionsUI.tsx` exports shared primitives:

```tsx
export function Section({ title, children }: { title: string; children: React.ReactNode }) { ... }
export function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) { ... }
export function Field({ label, value, onChange, placeholder, type }: FieldProps) { ... }
```

### Pattern: Extract Async Operations into Custom Hooks

**Problem**: `OptionsModal` contained 7 async operations (backupNow, testS3, gdriveAuth, onedriveAuth, listBackups, restore, + status states) inline.

**Solution**: `hooks/useBackupActions.ts` returns a typed `UseBackupActionsReturn` interface consumable by any component.

**Guideline**: When a component has >3 async operations + their loading/error states → extract into a hook.

## OCP — Open/Closed Principle

### Pattern: Strategy Pattern for Config Save Operations

**Problem**: `saveContent()` in `RawConfigViewer` had an `if/else` chain dispatching save behavior by tab type (TOML → parse+save, INI → write+merge+save, custom → write only). Adding a new file type required modifying the component.

**Solution**: `services/configSaveStrategies.ts` defines a `ConfigSaveStrategy` interface:

```ts
export interface ConfigSaveStrategy {
  save(content: string, config: ServerConfig): Promise<ConfigSaveResult>
}

export class TomlSaveStrategy implements ConfigSaveStrategy { ... }
export class IniSaveStrategy implements ConfigSaveStrategy { ... }
export class CustomFileSaveStrategy implements ConfigSaveStrategy { ... }
```

Consumer code becomes:

```ts
const strategy = getSaveStrategy(activeBuiltin, activeCustom?.path ?? null, config)
const result = await strategy.save(content, config)
```

**New file types**: add a strategy class. Zero changes to `RawConfigViewer`.

**Guideline**: When you see `if/else` or `switch` dispatching on a "type" key where each branch does structurally different work → extract a Strategy.

### Pattern: Portal-Based Dropdown (Extensibility)

The `DropdownPortal` component uses `createPortal` to escape stacking context, making any future dropdown consumer work without z-index debugging. Extract once, reuse everywhere.

## LSP — Liskov Substitution Principle

### Pattern: Strategy Substitutability

All `ConfigSaveStrategy` implementations return `ConfigSaveResult` — the consumer doesn't need to know which strategy it's using. `TomlSaveStrategy`, `IniSaveStrategy`, and `CustomFileSaveStrategy` are fully interchangeable from the caller's perspective.

### Pattern: TauriCommandService Interface

`TauriCommandService` defines the contract. `TauriCommandServiceImpl` wraps the real Tauri `invoke`. A mock implementation can be substituted for testing without any code changes in consumers.

## ISP — Interface Segregation Principle

### Pattern: Focused Hook Returns

`useBackupActions` returns only backup-related operations and states — not the entire app state. Components consume only what they need.

### Pattern: Strategy Interface Minimalism

`ConfigSaveStrategy` has a single method: `save(content, config)`. Each strategy implements only what it needs. No "god interface" with methods for every file type.

## DIP — Dependency Inversion Principle

### Pattern: TauriCommandService Abstraction

**Before**: Every component imported `invoke` from `services/tauri.ts` and called Tauri commands directly with raw string command names:

```ts
const parsed = await invoke<ServerConfig>('parse_config_from_toml', { tomlStr: content })
```

**After**: Components depend on the `TauriCommandService` interface:

```ts
export interface TauriCommandService {
  loadConfigOrDefault(): Promise<ServerConfig>
  saveConfig(config: ServerConfig): Promise<void>
  parseConfigFromToml(tomlStr: string): Promise<ServerConfig>
  // ... 20 more methods
}
```

`TauriCommandServiceImpl` implements it with real `invoke` calls. A `MockTauriCommandService` can implement the same interface for unit tests.

**Benefits**:
- Components don't know about `invoke` or raw command strings
- Strategies receive the service via constructor (injectable)
- Testing: swap `tauriCommands` for a mock without touching component code

### Pattern: Constructor Injection in Strategies

```ts
export class IniSaveStrategy implements ConfigSaveStrategy {
  constructor(
    private path: string,
    private cmd: TauriCommandService = tauriCommands,
  ) {}
}
```

Default production usage: `new IniSaveStrategy(path)` (uses real Tauri).
Test usage: `new IniSaveStrategy(path, mockService)`.

## Design Patterns Summary

| Pattern | Where Applied | SOLID Principle |
|---------|--------------|-----------------|
| Strategy | `configSaveStrategies.ts` — per-file-type save behavior | OCP |
| Portal | `DropdownPortal.tsx` — z-index escape hatch | OCP (extensible) |
| Facade | `TauriCommandService.ts` — typed wrapper over raw `invoke` | DIP |
| Custom Hook | `useBackupActions.ts` — async ops + state | SRP, ISP |
| Decomposition | `options/GeneralTab`, `BackupTab`, `ConfigTab` | SRP |
| Service Module | `configGenerators.ts` — pure logic, no React | SRP, DIP |

## File Structure Convention

```
frontend/src/
  services/              # Pure logic, no React
    configGenerators.ts    # INI/TOML generators
    configSaveStrategies.ts # Strategy pattern implementations
    TauriCommandService.ts # DIP interface + impl
    tauri.ts              # Low-level invoke wrapper
  hooks/                 # Reusable stateful logic
    useBackupActions.ts   # Async ops + status
    useTextHistory.ts     # Undo/redo
    useServerLifecycle.ts # Start/stop/restart
  components/
    ui/                  # Shared primitives
      OptionsUI.tsx       # Section, Toggle, Field
    options/             # OptionsModal sub-components
      GeneralTab.tsx
      BackupTab.tsx
      ConfigTab.tsx
    DropdownPortal.tsx   # Reusable portal dropdown
    OptionsModal.tsx     # Thin shell (~60 lines)
    RawConfigViewer.tsx  # Uses strategy pattern for save
```

## Checklist for New Features

- [ ] Does this component have >1 responsibility? → Split it
- [ ] Is there an if/else dispatching on "type"? → Strategy pattern
- [ ] Does this function use React? → No → `services/` or `utils/`
- [ ] Are you calling `invoke()` directly? → Use `TauriCommandService`
- [ ] >3 async ops in a component? → Extract to a hook
- [ ] Reusable UI pattern? → `components/ui/`
