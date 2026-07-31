# Contributing to EasyTube

## Git Workflow

- `dev` is the main development branch. All PRs target `dev`.
- `main` receives only stable releases.
- Branch naming: `feat/<issue>-<slug>`, `fix/<issue>-<slug>`, `refactor/<slug>`, `docs/<slug>`, `chore/<slug>`, `test/<slug>`.
- Use conventional commits: `feat(scope): description`, `fix(scope): description`.
- PRs squash-merge to `dev`. Require CI green + 1 review.

## Code Style

### Flat Judgment Rule
- Use guard clauses and early returns. Never nest if/else deeper than 2 levels.
- All validation functions return `Result`. Use `?` operator heavily.

### Rust
- `cargo fmt` and `cargo clippy -D warnings` before commit.
- All log output goes to stderr. Stdout is reserved for MCP protocol.

### TypeScript / React
- Biome handles formatting and linting.
- Use `use-intl` for all user-facing strings. Never hardcode text.
- Components: `components/ui/` for shadcn, `components/` for app components.

## Architecture

```
src/  (React UI, calls tauri commands only)
  |
src-tauri/  (Tauri glue: commands -> core)
  |
crates/{mcp,cli}/  (thin adapters over core)
  |
crates/core/  (pure domain logic, no tauri deps)
```

Core DTOs live in `crates/core/src/types.rs`; all adapters reuse them.
No business logic in adapter layers.
