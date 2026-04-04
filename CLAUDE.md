# Claude Status Pet — Developer Guide for AI Coders

This document is for AI coding assistants (Claude Code, GitHub Copilot, Cursor, etc.) working on this repo. Read this before making changes.

## What This Project Is

A desktop pet (Tauri app) that floats on screen and shows what an AI coding assistant is doing in real time. A single Rust binary handles everything: parsing hook events, writing status, downloading DLC, and rendering the UI.

```
Hook event → claude-status-pet write-status → status-{id}.json → claude-status-pet GUI (file watcher) → UI update
```

**Key principle**: Zero runtime dependencies. The binary is the only distributable — no scripts, no Node.js, no Python at runtime.

## Project Structure

```
claude-status-pet/
├── .claude-plugin/          # Claude Code plugin manifest
│   ├── plugin.json          # DO NOT add "hooks" field — hooks/hooks.json is auto-discovered
│   └── marketplace.json     # Marketplace registry
├── .claude/
│   └── skills/release.md    # /release command (project-level skill)
├── hooks/
│   └── hooks.json           # Claude Code hooks → calls binary with --adapter claude
├── copilot/
│   ├── plugin.json         # Copilot CLI plugin manifest
│   ├── hooks.json           # GitHub Copilot CLI hooks → calls scripts with event arg
│   ├── scripts/
│   │   ├── hook.sh          # Bash hook handler (deployed to ~/.claude/pet-data/scripts/)
│   │   └── hook.ps1         # PowerShell hook handler (deployed to ~/.claude/pet-data/scripts/)
│   ├── skills/pet/SKILL.md  # /pet skill (copy of skills/pet/SKILL.md for plugin packaging)
│   └── README.md
├── skills/
│   └── pet/SKILL.md         # /pet slash command (works with Claude Code + Copilot via ~/.claude/skills/)
├── docs/
│   ├── HOOKS.md             # Hook event → status mapping reference
│   └── images/              # Compressed showcase GIFs
├── pet-app/                 # Tauri desktop app
│   ├── src/                 # Frontend (HTML/CSS/JS — plain files, no framework)
│   │   ├── index.html
│   │   ├── style.css
│   │   ├── app.js           # Character rendering, status updates, 2-level right-click menu
│   │   └── ferris/          # Ferris SVG art + character.json
│   └── src-tauri/           # Rust backend
│       ├── src/
│       │   ├── lib.rs       # GUI mode + write-status CLI + DLC download
│       │   ├── adapter/     # Hook adapters (one per AI agent)
│       │   │   ├── mod.rs   # Adapter trait + registry + StdinInput struct
│       │   │   ├── claude.rs   # Claude Code adapter
│       │   │   └── copilot.rs  # GitHub Copilot CLI adapter
│       │   ├── status_map.rs   # Universal tool→state fuzzy matching
│       │   └── tests.rs     # Unit tests
│       ├── Cargo.toml       # Dependencies: tauri, serde, notify, base64, ureq
│       └── tauri.conf.json
├── .github/workflows/
│   └── release.yml          # CI: builds binaries + asset zip on version tags
├── CONTRIBUTING.md          # Adding characters + adapters
├── INSTALL.md               # Agent-readable install instructions
├── README.md                # User-facing (English)
└── README.zh-CN.md          # User-facing (Chinese)
```

## Binary Modes

```
claude-status-pet write-status --adapter claude         # CLI: parse stdin, write status, exit
claude-status-pet write-status --adapter copilot --copilot-event preToolUse  # CLI: Copilot with event arg
claude-status-pet write-status --event tool --tool edit  # CLI: generic args, any agent
claude-status-pet run --status-file <path>                # GUI: launch Tauri window
claude-status-pet demo --assets-dir <path>               # GUI: cycle all states for recording
```

- `write-status` is the **hot path** — called on every hook event. Must complete in <100ms.
- `write-status` outputs `<status-file-path>\t<session-id>` to stdout (used by sessionStart hook to launch GUI).
- `write-status` ends with `process::exit(0)` to kill any lingering stdin reader thread.
- `write-status` does NOT spawn child processes (PowerShell `&` waits for all children).
- `run` is the long-lived GUI process that watches the status file.

## Adapter System

Two adapters in `src/adapter/` (VS Code Copilot adapter planned — 待支持):

| Adapter | Event source | Tool names | Session ID | Quirks |
|---------|-------------|------------|------------|--------|
| `claude` | stdin `hook_event_name` (PascalCase) | `Edit`, `Read`, `Bash` | stdin `session_id` | None |
| `copilot` | `--copilot-event` CLI arg | stdin `toolName` (camelCase) | stdin `sessionId` | sessionStart=thinking, userPromptSubmitted=ignored, postToolUse=thinking |
| `vscode` | _(待支持)_ | — | — | — |

### StdinInput parsing

`StdinInput` uses `serde(alias)` to handle both snake_case and camelCase:
- `tool_name` / `toolName` → same field
- `hook_event_name` / `hookEventName` → same field
- `session_id` / `sessionId` → same field
- `tool_args` / `toolArgs` → `Option<serde_json::Value>` (can be JSON string OR object)

**Critical**: `toolArgs` must be `Option<Value>` not `Option<String>`. Copilot CLI sends it as a JSON string for preToolUse, but as an object for postToolUse. If typed as `String`, serde silently fails on the object form, losing ALL fields including `sessionId`.

### Non-blocking stdin reader

`read_stdin()` reads byte-by-byte tracking `{}` depth. Returns immediately when outermost `}` closes — does NOT wait for EOF. 100ms timeout as safety net. Uses `Vec<u8>` + `from_utf8_lossy` (not `char` cast) for UTF-8 safety.

### Tool→State Mapping (`status_map.rs`)

Shared by all adapters. Uses fuzzy keyword matching:

- `edit`, `write`, `replace`, `create_file` → `editing`
- `read`, `view`, `fetch`, `list_dir` → `reading`
- `grep`, `search`, `find`, `glob` → `searching`
- `bash`, `terminal`, `run`, `shell` → `running`
- `agent`, `skill`, `delegate` → `delegating`
- anything else → `running` (fallback)

All `truncate()` functions use `is_char_boundary()` to avoid UTF-8 panics.

## Character System

Characters defined by `character.json` files:

- **Bundled**: `pet-app/src/ferris/character.json`
- **DLC**: `~/.claude/pet-data/assets/{mona,kuromi}/character.json` (downloaded by `download_dlc` Rust command using `ureq` HTTP client)
- **Custom**: `~/.claude/pet-data/characters/*/character.json` (user-installed packs)

DLC download is async (`spawn_blocking`) with 30s HTTP timeout per file. Auto-downloads missing DLC on startup if selected character requires it.

## States

| State | Animation | Label Color | Trigger |
|-------|-----------|-------------|---------|
| idle | gentle float | orange | done event |
| thinking | slow tilt | yellow | prompt event |
| reading | gentle float (fast) | blue | read/fetch tools |
| editing | wiggle | green | edit/write tools |
| searching | tilt (fast) | purple | grep/search tools |
| running | fast wiggle | orange | bash/terminal/other tools |
| delegating | bounce side-to-side | blue | agent/subagent |
| waiting | pulse | orange | permission prompt |
| error | shake 3x | red | error event |
| offline | slow breathing | grey | offline event |

Speech bubble: always visible for active states, 30s timeout for idle/offline.

## Key Design Decisions

### Single binary, zero runtime deps
All hook processing and DLC download in Rust (using `ureq` for HTTP). No Node.js, Python, or shell scripts at runtime. Build-time still needs Node.js (`npx tauri build`).

### Hooks must not block
- Claude Code hooks: `"async": true`
- GitHub Copilot hooks: `"timeoutSec": 1`
- `write-status` reads stdin (100ms timeout), writes file, calls `process::exit(0)` — total <100ms
- GUI launch: handled by `sessionStart` hook via `Start-Process` (non-blocking), NOT inside the binary

**Critical**: write-status must NEVER spawn child processes. PowerShell `&` waits for all children to exit.

### Session ID tracking
- `write-status` outputs `<status-file-path>\t<session-id>` to stdout
- `sessionStart` hook captures this output and passes the exact file path to `run` via `Start-Process`
- No guessing, no `.last-session` files, no `ls -t`

### Copilot-specific quirks (in adapter/copilot.rs)
- `sessionStart`: writes `thinking` — GUI launch handled by hook script
- `userPromptSubmitted`: returns `None` (ignored) — avoids overwriting sessionStart's thinking
- `postToolUse`: mapped to `thinking` — avoids idle flash between tools
- `sessionEnd`: depends on `reason`: `complete`→idle, `error`→error, `abort`/`user_exit`→idle, `timeout`→offline
- `stop`: maps to `idle`
- Event name from `--copilot-event` CLI arg (stdin may not have `hookEventName`)

### Image licensing
- **Ferris SVGs** (CC0) are bundled in the repo
- **Mona/Kuromi GIFs** (GIPHY) are NOT in the repo — downloaded at runtime via `download_dlc`

### Window transparency
`transparent: true`, `decorations: false`, `shadow: false`, `skipTaskbar: true` in tauri.conf.json.

### External assets via base64 data URLs
WebView2 blocks `file://` URLs. Assets loaded via `load_asset` Tauri command → base64 data URLs, cached in frontend memory.

## Common Pitfalls

- **plugin.json**: Do NOT add `"hooks"` field — `hooks/hooks.json` is auto-discovered
- **Building**: Use `npx tauri build` from `pet-app/`, NOT `cargo build` from `src-tauri/`
- **Hook blocking**: write-status must never block. No network calls, no spawning children. Write file → `process::exit(0)`
- **PowerShell `&`**: Waits for ALL child processes. NEVER spawn GUI from inside `& binary.exe`
- **toolArgs type**: Must be `Option<Value>` not `Option<String>` (Copilot sends both formats)
- **UTF-8 safety**: All `truncate()` must use `is_char_boundary()`. stdin reader uses `Vec<u8>` not `char` cast
- **Debug logging**: Disabled by default. Set env var `PET_DEBUG=1` to enable. Logs to `pet-debug.log`
- **SKILL uses native commands**: PowerShell on Windows, bash on Unix. No Node.js in SKILL
- **Stale status files**: Cleaned up on GUI startup (>24h old). Each session writes its own file

## Building

```bash
cd pet-app
npm install
npx tauri build  # requires Rust toolchain + MSVC on Windows
```

## Testing

```bash
cd pet-app/src-tauri
cargo test
```

## Releasing

Use the `/release` skill or manually:

1. Update version in: `plugin.json`, `tauri.conf.json`, `Cargo.toml`, `package.json`
2. Commit, tag: `git tag v0.X.0 && git push origin --tags`
3. CI builds binaries + asset zip. Pre-release tags (`-rc`, `-beta`) marked as pre-release.

## Status File Format

`~/.claude/pet-data/status-{session_id}.json`:

```json
{
  "state": "editing",
  "detail": "Editing app.js",
  "tool": "Edit",
  "event": "tool",
  "session_id": "abc123",
  "session_name": "my-project",
  "timestamp": "2026-04-02T10:30:00Z"
}
```
