# Claude Status Pet — Developer Guide for AI Coders

This document is for AI coding assistants (Claude Code, GitHub Copilot, Cursor, etc.) working on this repo. Read this before making changes.

## What This Project Is

A desktop pet (Tauri app) that floats on screen and shows what an AI coding assistant is doing in real time. A single Rust binary handles everything: parsing hook events, writing status, and rendering the UI.

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
│   ├── hooks.json           # GitHub Copilot hooks → calls binary with --adapter copilot
│   └── README.md
├── skills/
│   └── pet/SKILL.md         # /pet slash command definition
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
│       │   ├── lib.rs       # GUI mode: file watcher, WebView2, Tauri commands
│       │   ├── adapter/     # Hook adapters (one per AI agent)
│       │   │   ├── mod.rs   # Adapter trait + registry
│       │   │   ├── claude.rs   # Claude Code: PascalCase events, snake_case input
│       │   │   └── copilot.rs  # GitHub Copilot: camelCase events + quirks
│       │   ├── status_map.rs   # Universal tool→state fuzzy matching
│       │   └── tests.rs     # 24 unit tests
│       ├── Cargo.toml
│       └── tauri.conf.json
├── .github/workflows/
│   └── release.yml          # CI: builds binaries + asset zip on version tags
├── CONTRIBUTING.md          # Adding characters + adapters
├── INSTALL.md               # Agent-readable install instructions
├── README.md                # User-facing (English)
└── README.zh-CN.md          # User-facing (Chinese)
```

## Binary Modes

The binary runs in different modes based on the first argument:

```
claude-status-pet write-status --adapter claude    # CLI: parse stdin, write status, exit (~1ms)
claude-status-pet write-status --event tool --tool edit  # CLI: generic args, any agent
claude-status-pet run --status-file <path>         # GUI: launch Tauri window
claude-status-pet demo --assets-dir <path>         # GUI: cycle all states for recording
```

- `write-status` is the **hot path** — called on every hook event. Must be fast (<5ms), non-blocking.
- `write-status` auto-launches the GUI if no pet process is running.
- `run` is the long-lived GUI process that watches the status file.

## Adapter System

Each AI agent has an adapter in `src/adapter/`. Adapters do two things:

1. **Format conversion**: Parse agent-specific stdin JSON → normalized `(event, tool, detail, session_id)`
2. **Quirk handling**: Agent-specific behavioral fixes (e.g., Copilot sessionStart race condition)

| Adapter | Event source | Tool names | Quirks |
|---------|-------------|------------|--------|
| `claude` | `stdin.hook_event_name` (PascalCase) | `Edit`, `Read`, `Bash` | None |
| `copilot` | `env.COPILOT_HOOK_EVENT` (camelCase) | `replace_string_in_file`, `read_file` | sessionStart=launch_only, postToolUse=thinking |

**Adding a new adapter**: See CONTRIBUTING.md. New agents should prefer CLI args (`--event/--tool`) over custom adapters.

### Tool→State Mapping (`status_map.rs`)

Shared by all adapters. Uses fuzzy keyword matching:

- `edit`, `write`, `replace`, `create_file` → `editing`
- `read`, `view`, `fetch`, `list_dir` → `reading`
- `grep`, `search`, `find`, `glob` → `searching`
- `bash`, `terminal`, `run`, `shell` → `running`
- `agent`, `skill`, `delegate` → `delegating`
- anything else → `running` (fallback)

MCP tools (`mcp__server__tool`) are auto-formatted as "server: tool".

## Character System

Characters are defined by `character.json` files (not hardcoded):

- **Bundled**: `pet-app/src/ferris/character.json` (built into frontend)
- **DLC**: `~/.claude/pet-data/assets/{mona,kuromi}/character.json` (generated by `download_dlc` Rust command)
- **Custom**: `~/.claude/pet-data/characters/*/character.json` (user-installed packs)

```json
{
  "name": "My Character",
  "type": "gif",
  "states": {
    "idle": ["mychar/idle.gif"],
    "thinking": ["mychar/think.gif"],
    ...
  }
}
```

The app auto-discovers character packs via `list_character_packs` Tauri command.

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

## Key Design Decisions

### Single binary, zero runtime deps
All hook processing is in Rust. No Node.js, Python, or shell scripts at runtime. Build-time still needs Node.js (`npx tauri build`).

### Hooks must not block
All hooks use `"async": true` (Claude Code) or `"timeoutSec": 1` (GitHub Copilot). The `write-status` CLI reads stdin (100ms timeout), writes a file, and calls `process::exit(0)` — total <100ms. GUI launch is handled separately by `sessionStart` hook via `Start-Process` (non-blocking).

**Critical**: write-status must NEVER spawn child processes. PowerShell `&` waits for all children. GUI launch uses `Start-Process` in the hook script, not inside the binary.

### Image licensing
- **Ferris SVGs** (CC0) are bundled in the repo
- **Mona/Kuromi GIFs** (GIPHY) are NOT in the repo — downloaded at runtime via `download_dlc` Rust command
- Never commit GIPHY-sourced GIFs to git

### Window transparency on Windows
`transparent: true`, `decorations: false`, `shadow: false` in tauri.conf.json. WebView2 background set to RGBA(0,0,0,0) in Rust. Do NOT use Win32 DWM hacks.

### External assets via base64 data URLs
WebView2 blocks `file://` URLs. Assets loaded via `load_asset` Tauri command → base64 data URLs, cached in frontend memory.

### Copilot-specific quirks (in adapter/copilot.rs)
- `sessionStart`: writes `thinking` state — GUI launch handled by hook script's `Start-Process`
- `userPromptSubmitted`: ignored (returns `None`) — sessionStart already sets thinking
- `postToolUse`: mapped to `thinking` — avoids idle flash between tools
- `sessionEnd`: depends on `reason` field: `complete`→idle, `error`→error, `abort`/`user_exit`→idle, `timeout`→offline
- `stop`: maps to `idle`
- `session_id`: hashed from `cwd` using simple hash

## Common Pitfalls

- **plugin.json**: Do NOT add `"hooks"` field — `hooks/hooks.json` is auto-discovered
- **marketplace.json**: Must have top-level `name` (string) and `owner` (object with `name` field)
- **Building**: Use `npx tauri build` from `pet-app/`, NOT `cargo build` from `src-tauri/` (the latter doesn't bundle frontend)
- **Window border on Windows**: Check `shadow: false` in tauri.conf.json
- **Hook blocking**: `write-status` must never block. No network calls, no spawning child processes. Write file → `process::exit(0)`.
- **PowerShell `&` waits for children**: NEVER spawn GUI from inside `& binary.exe`. Use `Start-Process` in the hook script instead.
- **Adapter quirks**: All agent-specific behavior goes in the adapter module, not shared code.
- **auto_start default**: `false` — pet only auto-launches if user explicitly sets `auto_start: true` in config.json
- **Local debugging**: Always launch pet with `--debug` flag to enable logging to `~/.claude/pet-data/pet-debug.log`
- **No scripts**: Everything is in the Rust binary. No Node.js, Python, or shell scripts at runtime.

## Building

```bash
cd pet-app
npm install
npx tauri build  # requires Rust toolchain + MSVC on Windows
```

Binary output: `pet-app/src-tauri/target/release/claude-status-pet(.exe)`

## Testing

```bash
cd pet-app/src-tauri
cargo test  # runs 24 unit tests (adapters, status mapping)
```

```bash
# Test write-status CLI
echo '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"/foo/bar.rs"},"session_id":"test","cwd":"/proj"}' | ./target/release/claude-status-pet write-status --adapter claude

# Test generic CLI args
./target/release/claude-status-pet write-status --event tool --tool edit --detail "Editing bar.rs" --session-id test

# Launch demo mode
./target/release/claude-status-pet demo --assets-dir ~/.claude/pet-data/assets
```

## Releasing

Use the `/release` skill or manually:

1. Update version in: `plugin.json`, `tauri.conf.json`, `Cargo.toml`, `package.json`
2. Commit, tag: `git tag v0.X.0 && git push origin --tags`
3. CI builds binaries + asset zip and uploads to GitHub Releases

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
