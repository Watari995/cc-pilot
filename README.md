# cc-pilot

A macOS menu bar app that monitors all your Claude Code sessions (CLI, VS Code, Cursor, Desktop, Web) in one real-time dashboard.

Built with **Tauri v2 + React + TypeScript + Rust**.

## Features

- **Real-time session monitoring** — watches `~/.claude/projects/` for live updates
- **Web session monitoring** — polls claude.ai API to show Claude Code web sessions
- **Environment detection** — automatically identifies Terminal, Cursor, VS Code, Desktop, and Web sessions
- **One-click jump** — click a session to switch to its terminal tab, IDE window, or browser
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
| Web (claude.ai) | Opens `https://claude.ai/code/{session_id}` |

## Web Session Monitoring

cc-pilot can monitor your Claude Code sessions running on claude.ai.

### Setup

1. Open claude.ai in your browser and log in
2. Open DevTools → Application → Cookies → `https://claude.ai`
3. Copy the value of the `sessionKey` cookie (`sk-ant-sid01-...`)
4. In cc-pilot Settings, paste it into the "Session Key" field

### How it works

- Polls `GET https://claude.ai/v1/sessions` every 30 seconds
- Shows sessions updated within the last 3 days
- Requires Anthropic-specific headers (`Anthropic-Beta`, `Anthropic-Client-Feature`, etc.)
- Session key has an expiration — re-enter when it expires

### Limitations

- Uses an **unofficial API** — may break if Anthropic changes the endpoint
- Less real-time than local sessions (30s polling vs instant file-change detection)
- Token counts are not available for web sessions

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
Local sessions:
  ~/.claude/projects/**/*.jsonl
    → Rust FileWatcher (notify crate)
    → JSONL parser (tail-read for performance)
    → Tauri IPC events
    → React/Zustand store → UI

Web sessions:
  claude.ai /v1/sessions API
    → Rust WebClient (reqwest, 30s polling)
    → Session struct conversion
    → Tauri IPC events
    → React/Zustand store → UI (merged with local sessions)
```

## CI/CD

- **CI**: TypeScript check, cargo check, clippy, frontend build on every push/PR
- **Release**: Tag `v*` to build a universal macOS `.dmg` and create a draft GitHub release

## License

MIT
