# EasyTube

> 連 60 歲長輩都會用的影片下載工具 — 貼上網址，一鍵下載。

A simple, cross-platform video downloader powered by [yt-dlp](https://github.com/yt-dlp/yt-dlp).
Works on macOS, Windows, and Linux.

## Features

- **One-click download** — paste a URL, pick quality, done
- **Clipboard auto-detect** — toggle to detect copied video links automatically
- **Smart duplicate detection** — warns if same URL + same quality was already downloaded
- **Parallel downloads** — download multiple videos at once (configurable 1-60)
- **Three languages** — 繁體中文 / 简体中文 / English
- **AI-ready** — built-in MCP server for Claude Desktop, Cursor, opencode
- **CLI mode** — `easytube download <url>` for terminal users
- **No tracking, no telemetry** — clipboard contents never leave your device

## Install

### macOS
Download the `.dmg` from [Releases](https://github.com/EasyTube/easytube/releases).

Or via Homebrew (coming soon):
```bash
brew install easytube
```

### Windows
Download the `.msi` installer from [Releases](https://github.com/EasyTube/easytube/releases).

Or via winget (coming soon):
```bash
winget install EasyTube
```

### Linux
Download `.deb` or `.AppImage` from [Releases](https://github.com/EasyTube/easytube/releases).

## Development

### Prerequisites
- Node.js >= 22
- pnpm >= 9
- Rust >= 1.85

### Setup
```bash
pnpm install
pnpm tauri dev
```

### Scripts
| Script | Description |
|---|---|
| `pnpm dev` | Start Vite dev server |
| `pnpm tauri dev` | Start Tauri desktop app |
| `pnpm build` | Build frontend |
| `pnpm check` | TypeScript type-check |
| `pnpm test` | Run frontend tests |
| `pnpm lint` | Biome lint check |
| `pnpm i18n:generate` | Generate zh-CN from zh-TW via OpenCC |
| `pnpm tauri build` | Build production binaries |

### Project Structure
```
src/                React 19 + Vite 7 + Tailwind v4 + shadcn/ui + use-intl
src-tauri/          Tauri 2.11 desktop shell (Rust)
crates/
  core/             Download engine, filter pipeline, binary provider
  mcp/              MCP stdio server (AI assistant integration)
  cli/              CLI subcommands (download, probe, history)
```

## MCP Server

EasyTube can act as an MCP server for AI assistants:

```json
{
  "mcpServers": {
    "easytube": {
      "command": "easytube",
      "args": ["mcp"]
    }
  }
}
```

See [docs/mcp-setup.md](docs/mcp-setup.md) for details.

## CLI

```bash
easytube download <url> [-f <format>] [-o <output_dir>]
easytube probe <url>
easytube history
easytube settings [<key> [<value>]]
```

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop | Tauri 2.11 (Rust) |
| Frontend | React 19 + Vite 7 + TypeScript 5.8 |
| CSS | Tailwind CSS v4 + shadcn/ui v4 (radix-vega) |
| Icons | lucide-react |
| i18n | use-intl + OpenCC tw2sp |
| Caching | TanStack Query v5 + IndexedDB (idb-keyval) |
| Lint/Format | Biome (JS/TS) + rustfmt + clippy (Rust) |
| Test | Vitest + cargo test |

## License

MIT © EasyTube Contributors

## Disclaimer

EasyTube is a tool for downloading publicly available videos for personal use.
Please respect the terms of service of the platforms you use.
