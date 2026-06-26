# Contributing to ARK ASA Config Manager

Thank you for your interest in contributing! This guide explains how to work on the project effectively.

## Development Setup

### Prerequisites
- Rust 1.80+ ([rustup](https://rustup.rs/))
- Node.js 18+ ([nodejs.org](https://nodejs.org))
- Tauri CLI (installed with `pnpm install`)
- Git

### Initial Setup

```bash
# Clone the repo
git clone <repo-url>
cd ArkASA-Servidor-Dedicado

# Install dependencies
pnpm install

# Start development server
pnpm run tauri:dev
```

## Project Architecture

### Rust Backend (`src/`)

**Module Structure:**
```
src/
├── config/          # Configuration management
├── ark/             # Server lifecycle management
├── storage/         # Database operations
├── error.rs         # Error types
├── lib.rs           # Tauri command handlers
└── main.rs          # Entry point
```

**Key Principles:**
- **SOLID Design** - Single responsibility, open/closed principle
- **Type Safety** - Rust's type system prevents whole classes of bugs
- **Async/Await** - Non-blocking operations with Tokio
- **Error Handling** - Result types with context

### React Frontend (`frontend/src/`)

**Component Organization:**
```
frontend/src/
├── components/      # Reusable UI components
├── pages/          # Page-level components
├── hooks/          # Custom React hooks
├── stores/         # Global state (Zustand)
├── services/       # API calls to Tauri
├── types/          # TypeScript interfaces
└── styles/         # CSS and Tailwind
```

**Patterns:**
- Functional components with hooks
- Props-based data flow
- Zustand for global state
- Custom hooks for logic reuse

## Making Changes

### Backend Changes

1. **Add a new validator:**
   ```rust
   // src/config/validator.rs
   struct MyValidator;
   
   #[async_trait]
   impl ConfigValidator for MyValidator {
       async fn validate(&self, config: &ServerConfig) -> Result<()> {
           // Your validation logic
       }
       fn name(&self) -> &str { "MyValidator" }
   }
   
   // Add to CompositeValidator::new()
   ```

2. **Add a new Tauri command:**
   ```rust
   // src/lib.rs
   #[tauri::command]
   async fn my_command(param: String) -> Result<String> {
       Ok(format!("Hello, {}", param))
   }
   
   // Add to .invoke_handler()
   ```

3. **Add a database table:**
   ```sql
   -- migrations/002_new_table.sql
   CREATE TABLE my_table (
       id TEXT PRIMARY KEY,
       value TEXT NOT NULL
   );
   ```

### Frontend Changes

1. **Add a new component:**
   ```tsx
   // frontend/src/components/MyComponent.tsx
   interface MyComponentProps {
     data: string
   }

   export default function MyComponent({ data }: MyComponentProps) {
     return <div>{data}</div>
   }
   ```

2. **Use the Zustand store:**
   ```tsx
   // In component
   const { config, setConfig } = useConfigStore()
   
   // Update store
   setConfig({ ...config, field: newValue })
   ```

3. **Call a Tauri command:**
   ```tsx
   import { invoke } from '@tauri-apps/api/tauri'
   
   const result = await invoke('my_command', { param: 'value' })
   ```

## Testing

### Rust Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Test Template
```rust
#[tokio::test]
async fn test_something() {
    // Arrange
    let input = "test";
    
    // Act
    let result = do_something(input);
    
    // Assert
    assert_eq!(result, "expected");
}
```

### Frontend Tests
```bash
# Run tests
pnpm run test

# Watch mode
pnpm run test:watch
```

## Code Quality

### Rust
```bash
# Check code
cargo clippy

# Format code
cargo fmt

# Security audit
cargo audit
```

### Frontend
```bash
# Lint
pnpm run lint

# Format
pnpm exec prettier --write .

# Type check
pnpm exec tsc --noEmit
```

## Git Workflow

### Commit Messages

Follow conventional commits:
```
type(scope): description

Optional longer explanation with context.

Co-Authored-By: Name <email>
```

**Types:** feat, fix, docs, style, refactor, test, chore

**Examples:**
```
feat(config): add new validator for X
fix(server): handle process termination correctly
docs(api): update command reference
```

### Creating a Pull Request

1. Create a feature branch:
   ```bash
   git checkout -b feature/my-feature
   ```

2. Make commits (one change per commit):
   ```bash
   git commit -m "feat: add new feature"
   ```

3. Push and create PR:
   ```bash
   git push origin feature/my-feature
   ```

4. PR Description should include:
   - What changed
   - Why it changed
   - How to test it
   - Any breaking changes

### Code Review Checklist

Before submitting a PR, verify:
- [ ] Code follows SOLID principles
- [ ] No `unwrap()` without justification
- [ ] Error messages are user-friendly
- [ ] Tests pass (`cargo test`)
- [ ] Linter passes (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] TypeScript compiles (`pnpm exec tsc`)
- [ ] Component renders correctly
- [ ] Accessibility considered (if UI change)

## Documentation

- **Code Comments** - Only when WHY is non-obvious
- **Commit Messages** - Explain the change, not what changed
- **PRs** - Describe impact and testing strategy
- **Docs/** - Keep architecture and API docs updated

## Performance Considerations

### Rust
- Minimize allocations in hot paths
- Use async for I/O-bound operations
- Profile before optimizing

### React
- Avoid unnecessary re-renders (React.memo)
- Lazy-load components
- Use useCallback for event handlers

## Security

### What to Watch For
- Input validation (Rust handles most)
- SQL injection (use parameterized queries)
- XSS (React escapes by default)
- Credential handling (never log passwords)
- Path traversal (validate paths)

## Debugging

### Rust
```bash
# Enable logging
RUST_LOG=debug pnpm run tauri:dev

# Use dbg! macro temporarily
let x = dbg!(some_function());
```

### React
```tsx
// Use console.log
console.log('value:', value)

// DevTools (Ctrl+Shift+I in Tauri window)
```

### Database
```sql
-- Connect to SQLite
sqlite3 ~/.config/ark-asa-config/ark-config.db

-- View schema
.schema

-- Query data
SELECT * FROM config_snapshots LIMIT 5;
```

## Architecture Decision Record (ADR)

For significant changes, document the decision:

```markdown
# ADR-N: Title

## Status
Proposed/Accepted/Deprecated

## Context
Why we needed to make a decision

## Decision
What we decided

## Consequences
- Pro: benefit 1
- Pro: benefit 2
- Con: drawback 1

## Alternatives
Other options we considered
```

## Roadmap

Current priorities:
1. **Phase 4** - Production hardening (in progress)
2. **Phase 5** - Web UI variant
3. **Phase 6** - Multi-server support

See [README.md](../README.md) for full roadmap.

## Getting Help

- **Questions** - Open an issue with label `question`
- **Bugs** - Open an issue with label `bug`
- **Discussion** - Use GitHub discussions
- **Documentation** - PRs to `/docs`

## Code of Conduct

Be respectful, constructive, and collaborative. All contributors are expected to create an inclusive environment.

---

**Happy coding!** 🚀

For questions, open an issue or start a discussion.
