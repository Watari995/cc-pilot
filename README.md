# cc-pilot

A macOS menu bar app that monitors all your Claude Code sessions (CLI, VS Code, Cursor, Desktop) in one real-time dashboard.

Built with **Tauri v2 + React + TypeScript + Rust**.

## Features

- **Real-time session monitoring** — watches `~/.claude/projects/` for live updates
- **Environment detection** — automatically identifies Terminal, Cursor, VS Code, and Desktop sessions
- **One-click jump** — click a session to switch to its terminal tab or IDE window
- **Approval notifications** — macOS notifications when a session needs tool approval
- **Menu bar resident** — runs in the background with a tray icon
- **Status overview** — per-status session counts in the status bar
- **Configurable** — accent color, terminal app, default IDE, notification preferences

## Supported Environments

| Environment | Jump Behavior |
|---|---|
| Ghostty | Accessibility API tab switch + activate |
| iTerm2 | AppleScript session match by CWD |
| Terminal.app | lsof + AppleScript TTY match |
| WezTerm | CLI pane/tab activation |
| VS Code / Cursor | `code` / `cursor` command |
| Claude Desktop | Deep link (`claude://`) |

## Development

```bash
# Prerequisites: Node.js 20+, Rust stable, Xcode Command Line Tools

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build

# Type checks
npx tsc --noEmit              # TypeScript
cd src-tauri && cargo clippy   # Rust
```

## Architecture

```
~/.claude/projects/**/*.jsonl
  → Rust FileWatcher (notify crate)
  → JSONL parser (tail-read for performance)
  → Tauri IPC events
  → React/Zustand store
  → UI
```

## CI/CD

- **CI**: TypeScript check, cargo check, clippy, frontend build on every push/PR
- **Release**: Tag `v*` to build a universal macOS `.dmg` and create a draft GitHub release

## License

MIT
