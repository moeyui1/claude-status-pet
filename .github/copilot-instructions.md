# Copilot Instructions — Claude Status Pet

## Architecture

A Tauri 2 desktop pet that shows AI assistant status in real time. Single Rust binary, two modes:

```
Hook event → binary write-status → status-{id}.json → binary run (Tauri GUI, file watcher) → UI update
```

- **`write-status`** — CLI hot path (<100ms). Parses hook stdin, writes JSON status file, exits immediately via `process::exit(0)`. Must NEVER block, spawn children, or make network calls.
- **`run`** — Long-lived Tauri GUI. Watches status file, emits events to WebView frontend. Auto-binds to the most recently updated session when launched without `--status-file`.

### Adapter system (`src/adapter/`)

Each AI agent gets an adapter that normalizes its hook JSON into a common `NormalizedEvent`. Adapters only handle parsing quirks — tool→state mapping is shared in `status_map.rs`.

- `claude.rs` — PascalCase events from stdin (`PreToolUse`, `Stop`)
- `copilot.rs` — camelCase events from `--copilot-event` CLI arg; `postToolUse` → thinking (prevents idle flash); `userPromptSubmitted` → ignored
- `vscode.rs` — PascalCase events from stdin (`PreToolUse`, `PostToolUse`); `PostToolUse` → thinking; `PreCompact` → ignored

See `docs/HOOKS.md` for full event→state mapping reference.

### Frontend (`pet-app/src/`)

Plain HTML/CSS/JS, no framework, no build step. All animations are CSS `@keyframes`. Assets loaded as base64 data URLs (WebView2 blocks `file://`).

## Build & Test

```bash
cd pet-app && npm install && npx tauri build    # full build (requires Rust + MSVC on Windows)
cd pet-app/src-tauri && cargo test              # run unit tests
cd pet-app/src-tauri && cargo test test_name    # run a single test
cd pet-app/src-tauri && cargo check             # fast compile check without building
```

**Important**: Use `npx tauri build` from `pet-app/`, NOT `cargo build` from `src-tauri/`. The latter doesn't bundle the frontend.

## Critical Rules

- **`toolArgs` must be `Option<serde_json::Value>`**, not `Option<String>`. Copilot sends JSON string for preToolUse but object for postToolUse. Wrong type silently drops all fields.
- **All `truncate()` must use `is_char_boundary()`** to avoid UTF-8 panics.
- **`write-status` must never spawn child processes.** PowerShell `&` waits for all children — spawning GUI from write-status blocks the hook.
- **Session IDs must be validated** with `is_safe_session_id()` before use in file paths (path traversal prevention).
- **`.claude-plugin/plugin.json`**: Do NOT add a `"hooks"` field — `hooks/hooks.json` is auto-discovered by the plugin system.
- **Debug logging**: Disabled by default. Set env var `PET_DEBUG=1` to enable logging to `pet-debug.log`.
- **Deployed binary filename**: `claude-status-pet-windows-x64.exe` (not `claude-status-pet.exe`).

## Releasing

Version must be updated in 6 files: `.claude-plugin/plugin.json`, `copilot/plugin.json`, `vscode/plugin.json`, `pet-app/src-tauri/tauri.conf.json`, `pet-app/src-tauri/Cargo.toml`, `pet-app/package.json`.

Then: `git tag vX.Y.Z && git push origin --tags` — CI builds binaries + asset zip automatically.

## SKILL files

`skills/pet/SKILL.md`, `copilot/skills/pet/SKILL.md`, and `vscode/skills/pet/SKILL.md` must be kept in sync (the copilot/vscode copies are for plugin packaging). SKILL commands use native shell (PowerShell/bash), never Node.js.

## Documentation Index

| Doc | Covers |
|-----|--------|
| `CLAUDE.md` | Full developer guide — project structure, binary modes, states, status file format |
| `CONTRIBUTING.md` | Adding characters (SVG/GIF/ASCII), `character.json` schema |
| `docs/HOOKS.md` | Hook event→state mapping for Claude Code, Copilot & VS Code |
| `docs/INSTALL.md` | Agent-readable install instructions (plugin + manual paths) |
| `docs/CUSTOM-CHARACTERS.md` | Custom character pack creation guide |
