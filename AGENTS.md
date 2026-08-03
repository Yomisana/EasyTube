# EasyTube Development Guide

## Stack
React 19 + Vite 7 + Tauri 2.11 · TypeScript 5.8 · Tailwind CSS v4 + shadcn/ui v4
Rust: crates/core (download engine) + crates/mcp (MCP server) + crates/cli

## Quick start
```bash
pnpm install && pnpm tauri dev
```

## Scripts
| Command | What it does |
|---------|-------------|
| `pnpm dev` | Vite dev server |
| `pnpm tauri dev` | Tauri desktop app |
| `pnpm build` | Frontend build |
| `pnpm check` | TypeScript type-check |
| `pnpm test` | Vitest |
| `pnpm lint` | Biome check |
| `pnpm i18n:generate` | OpenCC zh-TW → zh-CN |

## CI
- Push `dev` → auto-trigger frontend(TS) → rust(fmt/clippy/test) → locale(git diff gate)
- Release tags (`v*`) → release.yml: macOS/Win/Linux installers + portable zips

## Architecture
```
src/              React 19 + Vite 7 + Tailwind v4 + shadcn/ui + use-intl
src-tauri/        Tauri 2.11 desktop shell (Rust)
crates/
  core/           Download engine, filter pipeline, binary provider
  mcp/            MCP stdio server (AI assistant integration)
  cli/            CLI subcommands (download, probe, history)
```

## Guidelines
- **Non-secret config** → `providers.toml`, not `.env`
- **No hardcoded paths** → use `binary_provider` abstraction
- **i18n sync** → edit zh-TW, OpenCC generates zh-CN, CI checks git diff
- **GUI + CLI** → same Rust backend, different presentation layer
- **No telemetry** → clipboard content never leaves the device
