# Claude Status Pet

A desktop pet that shows what your AI coding assistant is doing — in real time. 🦀

<table>
<tr>
<td align="center">
<img src="docs/images/showcase-ferris.gif" width="100%" alt="Ferris demo">
</td>
<td align="center">
<img src="docs/images/showcase-kuromi.gif" width="100%" alt="Kuromi demo">
</td>
<td align="center">
<img src="docs/images/showcase-mona.gif" width="100%" alt="Mona demo">
</td>
<td align="center">
<img src="docs/images/showcase-ascii.gif" width="100%" alt="ASCII demo">
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

## Quick Start

In Claude Code, run:

```
/plugin marketplace add moeyui1/claude-status-pet
/plugin install claude-status-pet
```

That's it! A pet will appear on your next session. 🎉

## Features

- 🔴 **Real-time status** — watch your pet react as the assistant reads, edits, searches, thinks
- 🎭 **10+ characters** — Ferris (SVG), Mona & Kuromi (GIF DLC), and 6 ASCII art buddies
- 💃 **Animated** — unique animations per state (floating, wiggling, bouncing, sleeping)
- 🪟 **Multi-session** — each session gets its own pet window
- 🎨 **Customizable** — right-click to change character, colors, font size
- ⚡ **Lightweight** — ~5MB binary, ~20MB RAM (built with Tauri)

## Usage

**Right-click** the pet to open the menu:
- Switch character (Ferris, Mona, Kuromi, Chonk, Cat, Ghost, Robot, Duck, Axolotl, Snail)
- Customize colors, background, font size
- Exit the pet

**`/pet` commands** (in Claude Code):

| Command | Action |
|---------|--------|
| `/pet` or `/pet open` | Open pets for all active sessions |
| `/pet close` | Close all running pets |
| `/pet set <character>` | Set default character |
| `/pet auto on/off` | Toggle auto-start on session begin |
| `/pet status` | Show config and active sessions |

## GitHub Copilot Support

Also works with **GitHub Copilot CLI** and **Copilot Coding Agent**! See [`copilot/README.md`](copilot/README.md) for setup.

Both tools can run simultaneously — each gets its own pet window.

## Other Installation Methods

<details>
<summary>📦 Ask your AI agent to install</summary>

Tell your Claude Code:

> Read https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/INSTALL.md and install it for me

</details>

<details>
<summary>📥 Download pre-built binary</summary>

Download from [GitHub Releases](https://github.com/moeyui1/claude-status-pet/releases):

| Platform | Binary |
|----------|--------|
| Windows x64 | `claude-status-pet-windows-x64.exe` |
| macOS ARM (Apple Silicon) | `claude-status-pet-macos-arm64` |
| macOS Intel | `claude-status-pet-macos-x64` |
| Linux x64 | `claude-status-pet-linux-x64` |

Then follow [INSTALL.md](INSTALL.md) to configure hooks.

</details>

<details>
<summary>🔧 Build from source</summary>

Prerequisites: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/)

```bash
git clone https://github.com/moeyui1/claude-status-pet.git
cd claude-status-pet/pet-app
npm install
npx tauri build
```

Binary output: `pet-app/src-tauri/target/release/claude-status-pet(.exe)`

</details>

<details>
<summary>⚙️ Manual hook setup (Claude Code)</summary>

Add to your `~/.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      { "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }] }
    ],
    "PreToolUse": [
      { "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }] }
    ],
    "Stop": [
      { "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }] }
    ],
    "SessionStart": [
      { "matcher": "startup", "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/launch-pet.sh", "async": true }] }
    ],
    "SessionEnd": [
      { "hooks": [{ "type": "command", "command": "bash /path/to/claude-status-pet/scripts/status-writer.sh", "async": true }] }
    ]
  }
}
```

Replace `/path/to/claude-status-pet` with your actual clone path.

**Install the `/pet` skill:**

```bash
cp -r skills/pet ~/.claude/skills/pet
```

</details>

## Uninstall

```
/pet close
/plugin uninstall claude-status-pet
/plugin marketplace remove claude-status-pet
rm -rf ~/.claude/pet-data    # optional: remove downloaded assets
```

<details>
<summary>Manual uninstall</summary>

1. Remove hooks from `~/.claude/settings.json` (delete entries referencing `status-writer.sh` and `launch-pet.sh`)
2. `rm -rf ~/.claude/skills/pet`
3. `rm -rf ~/.claude/pet-data`

</details>

## How It Works

```
┌─────────────┐     hook events      ┌──────────────────┐     file watch     ┌─────────────┐
│  Claude Code │ ──────────────────> │  status-writer.sh │ ────────────────> │  Desktop Pet │
│  (or other)  │  PreToolUse, Stop,  │  writes JSON to   │  notify crate     │  (Tauri app) │
│              │  UserPromptSubmit   │  ~/.claude/pet-data│  watches changes  │              │
└─────────────┘                      └──────────────────┘                    └─────────────┘
```

The pet is **decoupled from any specific tool** — it just watches a JSON status file. To support another assistant (Cursor, Aider, Windsurf, etc.), write a hook/adapter that writes to `~/.claude/pet-data/status-{session_id}.json`. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Credits

- **Ferris**: [free-ferris-pack](https://github.com/MariaLetta/free-ferris-pack) by Maria Letta (CC0)
- **Mona**: [GitHub on GIPHY](https://giphy.com/GitHub) (downloaded at runtime)
- **Kuromi**: [Sanrio Korea on GIPHY](https://giphy.com/SanrioKorea) (downloaded at runtime)
- **ASCII sprites**: inspired by [any-buddy](https://github.com/cpaczek/any-buddy) by cpaczek
- Built with [Tauri](https://tauri.app/)

## License

MIT
