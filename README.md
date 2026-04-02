# Claude Status Pet

A desktop pet that shows your AI coding assistant's working status in real time. Watch a pixel-art Ferris (or ASCII art buddy) react as your assistant reads files, edits code, runs commands, and thinks.

## Showcase

<table>
<tr>
<td align="center">
<img src="https://github.com/user-attachments/assets/ffa59b1d-ad7f-47a7-869a-a2e31ff0c353" width="100%" alt="Ferris demo">
</td>
<td align="center">
<img src="https://github.com/user-attachments/assets/1df49ae1-d6ae-439d-9549-3c65135f3adf" width="100%" alt="Kuromi demo">
</td>
<td align="center">
<img src="https://github.com/user-attachments/assets/ecf7b3cd-499d-459e-8fa6-333669f2308c" width="100%" alt="Mona demo">
</td>
<td align="center">
<img src="https://github.com/user-attachments/assets/30f05b6c-11b6-4a79-a96f-fa404ddd8080" width="100%" alt="ASCII demo">
</td>
</tr>
</table>

<details>
<summary>📸 More screenshots</summary>
<br>
<table>
<tr>
<td align="center">
<img src="https://github.com/user-attachments/assets/f8174efa-99a5-4891-93db-5b269d7965ed" width="100%" alt="Ferris in action">
</td>
<td align="center">
<img src="https://github.com/user-attachments/assets/645fea27-ef66-46ee-974b-bf0161a8de98" width="100%" alt="Kuromi in action">
</td>
</tr>
</table>
</details>

## Features

- **Real-time status** — see what your AI assistant is doing (reading, editing, searching, thinking, idle)
- **Multiple characters** — switch between Ferris (SVG art from [free-ferris-pack](https://github.com/MariaLetta/free-ferris-pack)) and ASCII art buddies (Chonk, Cat, Ghost, Robot, Duck, Axolotl)
- **Animated** — each state has unique animations (floating, wiggling, bouncing, sleeping)
- **Multi-session** — each session gets its own pet, with session name displayed
- **Customizable** — right-click to change character, accent color, ASCII fill, background, font size
- **Lightweight** — built with Tauri (Rust + WebView), ~5MB binary, ~20MB RAM
- **Auto-start** — optionally launch a pet automatically when a new session starts
- **`/pet` command** — open, close, configure pets from within your AI assistant

## How It Works

```
┌─────────────┐     hook events      ┌──────────────────┐     file watch     ┌─────────────┐
│  Claude Code │ ──────────────────> │  status-writer.sh │ ────────────────> │  Desktop Pet │
│  (or other)  │  PreToolUse, Stop,  │  writes JSON to   │  notify crate     │  (Tauri app) │
│              │  UserPromptSubmit   │  ~/.claude/pet-data│  watches changes  │              │
└─────────────┘                      └──────────────────┘                    └─────────────┘
```

1. **Hooks** fire on tool use, prompt submit, stop, session start/end
2. **status-writer.sh** parses the hook JSON and writes a per-session status file
3. **Pet app** watches the status file and updates the character's pose and speech bubble

## Installation

### Option A: Plugin Install (easiest)

```
/plugin marketplace add moeyui1/claude-status-pet
/plugin install claude-status-pet
```

The plugin handles everything — hooks, scripts, and auto-downloads the binary on first session.

### Option B: Ask Your AI Agent

Tell your Claude Code:

> Read https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/INSTALL.md and install it for me

### Option C: Download Pre-Built Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/moeyui1/claude-status-pet/releases):

| Platform | Binary |
|----------|--------|
| Windows x64 | `claude-status-pet-windows-x64.exe` |
| macOS ARM (Apple Silicon) | `claude-status-pet-macos-arm64` |
| macOS Intel | `claude-status-pet-macos-x64` |
| Linux x64 | `claude-status-pet-linux-x64` |

Then follow the [INSTALL.md](INSTALL.md) guide to configure hooks.

## Uninstall

### Plugin install

```
/pet close
/plugin uninstall claude-status-pet
/plugin marketplace remove claude-status-pet
```

Then optionally clean up downloaded assets and data:

```bash
rm -rf ~/.claude/pet-data
```

### Manual install

1. Remove hooks from `~/.claude/settings.json` (delete all entries referencing `status-writer.sh` and `launch-pet.sh`)
2. Remove the skill: `rm -rf ~/.claude/skills/pet`
3. Remove data: `rm -rf ~/.claude/pet-data`

### Option D: Build from Source

Prerequisites: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/)

```bash
git clone https://github.com/moeyui1/claude-status-pet.git
cd claude-status-pet/pet-app
npm install
npx tauri build
```

The binary will be at `pet-app/src-tauri/target/release/claude-status-pet.exe` (Windows).

### Setup Hooks (Claude Code)

#### Manual Hook Setup

Add to your `~/.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }]
      }
    ],
    "PreToolUse": [
      {
        "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }]
      }
    ],
    "PostToolUse": [
      {
        "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }]
      }
    ],
    "Stop": [
      {
        "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }]
      }
    ],
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/launch-pet.sh", "async": true }]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }]
      }
    ]
  }
}
```

Replace `/path/to/claude-status-pet` with your actual clone path.

### Install the `/pet` Skill

Copy the skill to your Claude Code skills directory:

```bash
cp -r skills/pet ~/.claude/skills/pet
```

Then use `/pet`, `/pet open`, `/pet close`, `/pet status`, etc.

## Usage

### Launch Pet Manually

```bash
bash scripts/open-pet.sh
```

### Right-Click Menu

Right-click the pet to:
- Switch character (Ferris, Chonk, Cat, Ghost, Robot, Duck, Axolotl)
- Open **Settings** (accent color, ASCII fill, background color, font size)
- **Close** the menu
- **Exit** the pet

### `/pet` Commands

| Command | Action |
|---------|--------|
| `/pet` or `/pet open` | Open pets for all active sessions |
| `/pet close` | Close all running pets |
| `/pet set <character>` | Set default character |
| `/pet auto on/off` | Toggle auto-start |
| `/pet status` | Show config and active sessions |

## Architecture

```
claude-status-pet/
├── .claude-plugin/
│   └── plugin.json          # Claude Code plugin manifest
├── hooks/
│   └── hooks.json           # Hook definitions (for plugin mode)
├── scripts/
│   ├── status-writer.sh     # Hook script: parses events → status JSON
│   ├── launch-pet.sh        # Hook script: auto-launches pet on session start
│   └── open-pet.sh          # Manual launcher for all active sessions
├── skills/
│   └── pet/SKILL.md         # /pet slash command
└── pet-app/                  # Tauri desktop app
    ├── src/                  # Frontend (HTML/CSS/JS)
    │   ├── index.html
    │   ├── style.css
    │   ├── app.js
    │   └── ferris/           # SVG art from free-ferris-pack (CC0)
    └── src-tauri/            # Rust backend
        └── src/lib.rs        # File watcher + window transparency
```

### Status File Format

Each session writes to `~/.claude/pet-data/status-{session_id}.json`:

```json
{
  "state": "working",
  "detail": "Editing main.rs",
  "tool": "Edit",
  "event": "PreToolUse",
  "session_id": "abc123",
  "session_name": "my-project",
  "timestamp": "2025-04-02T10:30:00Z"
}
```

States: `idle`, `thinking`, `working`, `delegating`, `offline`

## GitHub Copilot Support

The pet also works with GitHub Copilot CLI and Copilot Coding Agent! See [`copilot/README.md`](copilot/README.md) for setup instructions.

Both tools can run simultaneously — each gets its own pet window.

## Extending to Other AI Assistants

The pet app is decoupled from any specific tool — it just watches a JSON file. To support another assistant (e.g., Cursor, Aider, Windsurf), write a hook/adapter that writes the same JSON format to `~/.claude/pet-data/status-{session_id}.json`:

```json
{
  "state": "working",
  "detail": "Editing main.rs",
  "tool": "edit",
  "event": "preToolUse",
  "session_id": "my-session-123",
  "session_name": "my-project",
  "timestamp": "2025-04-02T10:30:00Z"
}
```

States: `idle`, `thinking`, `working`, `delegating`, `offline`

Contributions welcome for integrations with other tools!

## Credits

- **Ferris illustrations**: [free-ferris-pack](https://github.com/MariaLetta/free-ferris-pack) by Maria Letta (CC0, bundled in repo)
- **Mona GIFs**: [GitHub on GIPHY](https://giphy.com/GitHub) (downloaded at runtime from GIPHY, not bundled)
- **Kuromi GIFs**: [Sanrio Korea on GIPHY](https://giphy.com/SanrioKorea) (downloaded at runtime from GIPHY, not bundled)
- **ASCII buddy sprites**: inspired by [any-buddy](https://github.com/cpaczek/any-buddy) by cpaczek
- Built with [Tauri](https://tauri.app/)

## License

MIT
