# Claude Status Pet — Developer Guide for AI Coders

This document is for AI coding assistants (Claude Code, GitHub Copilot, Cursor, etc.) working on this repo. Read this before making changes.

## What This Project Is

A desktop pet (Tauri app) that floats on screen and shows what an AI coding assistant is doing in real time. It works via file-watching: hook scripts write status JSON, the pet app watches the file and updates the character.

```
Hook events → status-writer.sh → status-{session_id}.json → Tauri app (file watcher) → UI update
```

## Project Structure

```
claude-status-pet/
├── .claude-plugin/          # Claude Code plugin manifest
│   ├── plugin.json          # DO NOT add "hooks" field — hooks/hooks.json is auto-discovered
│   └── marketplace.json     # Marketplace registry (needs top-level "name" and "owner")
├── hooks/
│   └── hooks.json           # Claude Code hook definitions (auto-loaded by convention)
├── scripts/
│   ├── status-writer.sh     # Core: parses hook JSON → writes status file (uses node, NOT jq)
│   ├── launch-pet.sh        # SessionStart: downloads binary + assets, launches pet
│   ├── open-pet.sh          # Manual launcher for all active sessions
│   ├── download-assets.sh   # Downloads pet-assets.zip from GitHub Releases (bash)
│   ├── download-assets.js   # Same as above (Node.js, cross-platform)
│   ├── download-gifs.js     # Downloads GIFs from GIPHY (Node.js, cross-platform)
├── skills/
│   └── pet/SKILL.md         # /pet slash command definition
├── copilot/                 # VS Code Copilot adapter
│   ├── hooks.json           # Copilot hook definitions (camelCase format)
│   ├── status-writer.js     # Copilot status writer (Node.js, Windows-compatible)
│   ├── status-writer.sh     # Copilot status writer (bash, macOS/Linux)
│   └── README.md
├── pet-app/                 # Tauri desktop app
│   ├── src/                 # Frontend (HTML/CSS/JS — NOT a framework, plain files)
│   │   ├── index.html
│   │   ├── style.css
│   │   ├── app.js           # Main logic: character rendering, status updates, right-click menu
│   │   ├── ferris/           # Ferris SVG art (50 files from free-ferris-pack, CC0)
│   │   └── mona/             # GitHub Mona GIF stickers (from Giphy)
│   └── src-tauri/           # Rust backend
│       ├── src/lib.rs       # File watcher, WebView2 transparency, asset loading
│       ├── Cargo.toml
│       └── tauri.conf.json  # Window config: transparent, decorations:false, shadow:false
├── .github/workflows/
│   └── release.yml          # CI: builds binaries + asset zip on version tags
├── CONTRIBUTING.md          # Guide for adding new characters
├── INSTALL.md               # Agent-readable install instructions
└── README.md
```

## Key Design Decisions

### No jq dependency
Hook scripts use `node` for JSON parsing. Every Claude Code / Copilot environment has Node.js. Do NOT introduce jq as a dependency.

### Image licensing
- **Ferris SVGs** (CC0) are bundled in the repo — safe to redistribute
- **Mona GIFs** (GitHub/GIPHY) and **Kuromi GIFs** (Sanrio/GIPHY) are NOT in the repo — downloaded at runtime from GIPHY via `download-gifs.js`
- Never commit GIPHY-sourced GIFs to git. Add new GIF characters to `download-gifs.js` instead
- `pet-assets.zip` in releases only contains CC0-licensed Ferris SVGs

### Hook systems: Claude Code vs VS Code Copilot

Two different hook systems with different event names, input formats, and tool names:

**Claude Code hooks** (`hooks/hooks.json`):
- Events: `UserPromptSubmit`, `PreToolUse`, `Stop`, `StopFailure`, `Notification`, `SessionStart`, `SessionEnd`, `SubagentStart`
- Tool input uses `snake_case`: `tool_input.file_path`
- Tool names: `Edit`, `Write`, `Read`, `Bash`, `Grep`, `Glob`, `WebFetch`, `WebSearch`
- Has `Stop` hook (fires when agent finishes responding) — used for idle state
- Has `StopFailure` — used for error state

**VS Code Copilot hooks** (`copilot/hooks.json`):
- Events (camelCase in config, auto-converted to PascalCase): `sessionStart`, `userPromptSubmitted`, `preToolUse`, `postToolUse`, `stop`, `subagentStart`, `subagentStop`, `preCompact`
- Tool input uses `camelCase`: `tool_input.filePath`
- Tool names: `replace_string_in_file`, `create_file`, `read_file`, `run_in_terminal`, `grep_search`, `file_search`, `semantic_search`, `fetch_webpage`, `list_dir`
- `stop` fires after **each agent response** (not just session close) — used for idle state
- `postToolUse` fires after each tool — mapped to `thinking` (not idle, to avoid flashing)
- No `sessionEnd` event — use `stop` or file cleanup instead
- OS-specific commands: `powershell` → Windows, `bash` → macOS/Linux
- On Windows, `bash` resolves to WSL — always use `node` for PowerShell commands. Never use `~` in PowerShell paths (use `$env:USERPROFILE`).

**Doc reference**: https://code.visualstudio.com/docs/copilot/customization/hooks

### Window transparency on Windows
Tauri config: `transparent: true`, `decorations: false`, `shadow: false`. WebView2 background set to RGBA(0,0,0,0) in Rust. Do NOT use Win32 DWM hacks — Tauri handles it natively. Reference: https://github.com/ayangweb/BongoCat

### External assets via base64 data URLs
WebView2 blocks `file://` URLs for security. Assets from external directories are loaded via the `load_asset` Tauri command which returns base64 data URLs. Cached in memory on the frontend.

### Tool-to-state mapping is category-based
Tools are grouped into categories (readTools, editTools, searchTools, etc.) so new tools automatically get the right state without code changes. MCP tools (`mcp__*`) auto-format as "server: action".

## States

| State | Animation | Label Color | Trigger |
|-------|-----------|-------------|---------|
| idle | gentle float | orange | Stop hook |
| thinking | slow tilt | yellow | UserPromptSubmit |
| reading | gentle float (fast) | blue | Read, WebFetch |
| editing | wiggle | green | Edit, Write |
| searching | tilt (fast) | purple | Grep, Glob, WebSearch |
| running | fast wiggle | orange | Bash, other tools |
| delegating | bounce side-to-side | blue | Agent, SubagentStart |
| error | shake 3x | red | StopFailure |
| offline | slow breathing | grey | SessionEnd |

## Adding a New Character

See CONTRIBUTING.md. Three formats: SVG images, GIF animations, ASCII art.

## Common Pitfalls

- **plugin.json**: Do NOT add `"hooks": "./hooks/hooks.json"` — it's auto-discovered and will cause "duplicate hooks" error
- **marketplace.json**: Must have top-level `name` (string) and `owner` (object with `name` field)
- **userConfig in plugin.json**: Each field needs `type` ("string"|"number"|"boolean") and `title` (string). Skip if not needed.
- **Silent failures**: Never use `2>/dev/null || true` to swallow errors. Show clear error messages.
- **jq**: Not available in many environments. Use `node -e` instead.
- **Window border on Windows**: If you see a border/shadow, check `shadow: false` in tauri.conf.json. Do NOT try Win32 API hacks.

## Building

```bash
cd pet-app
npm install
npx tauri build  # requires Rust toolchain
```

Binary output: `pet-app/src-tauri/target/release/claude-status-pet(.exe)`

## Releasing

1. Update version in `plugin.json`
2. Commit and push
3. Tag: `git tag v0.X.0 && git push origin v0.X.0`
4. CI builds binaries + asset zip and uploads to GitHub Releases
5. Users get updates automatically via `download-assets.js` version check

## Status File Format

`~/.claude/pet-data/status-{session_id}.json`:

```json
{
  "state": "editing",
  "detail": "Editing app.js",
  "tool": "Edit",
  "event": "PreToolUse",
  "session_id": "abc123",
  "session_name": "my-project",
  "timestamp": "2026-04-02T10:30:00Z"
}
```

## Testing

```bash
# Simulate a status update
echo '{"hook_event_name":"PreToolUse","session_id":"test","tool_name":"Edit","tool_input":{"file_path":"/foo/bar.rs"},"cwd":"/home/user/project"}' | bash scripts/status-writer.sh

# Launch pet manually
bash scripts/open-pet.sh
```
