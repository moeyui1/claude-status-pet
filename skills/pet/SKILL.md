---
name: pet
description: Manage your desktop pet — open, close, set character, toggle auto-start
user-invocable: true
---

# /pet command

Manage the Claude Status Pet desktop companion.

## Usage

The user will run `/pet` with an optional subcommand. Parse the arguments and execute accordingly:

### Subcommands

- `/pet` or `/pet open` — Open pet(s) for all active sessions
- `/pet close` — Close all running pets  
- `/pet close all` — Same as close
- `/pet set <character>` — Switch character. Available: `ferris`, `mona`, `kuromi`, `chonk`, `cat`, `ghost`, `robot`, `duck`, `snail`, `axolotl`
- `/pet auto on` — Enable auto-start on new sessions
- `/pet auto off` — Disable auto-start
- `/pet status` — Show current config and active sessions
- `/pet update` — Update binary and assets to latest release
- `/pet help` — Show available commands

## Implementation

Use Bash to execute the actions. First, determine the plugin root directory:

```bash
# Try plugin root, then common install locations
PET_ROOT="${CLAUDE_PLUGIN_ROOT:-$(find "$HOME" -maxdepth 3 -name "claude-status-pet" -type d 2>/dev/null | head -1)}"
```

- **Pet binary**: `$PET_ROOT/pet-app/src-tauri/target/release/claude-status-pet.exe` (Windows) or without `.exe` (macOS/Linux)
- **Status files**: `~/.claude/pet-data/status-*.json` (one per session)
- **Config file**: `~/.claude/pet-data/config.json`
- **Open script**: `bash $PET_ROOT/scripts/open-pet.sh`

For `open`: run the open-pet.sh script.
For `close`: run `taskkill //F //IM claude-status-pet.exe` on Windows, or `pkill claude-status-pet` on macOS/Linux.
For `set <char>`: update `config.json` with the character name, and inform the user to right-click the pet to switch in the current session.
For `auto on/off`: update `config.json` field `auto_start` to true/false.
For `status`: read config.json and list status-*.json files.
For `update`:
  1. Check current vs latest version:
     ```bash
     # Read installed version from plugin.json
     INSTALLED=$(node -e "try{console.log(JSON.parse(require('fs').readFileSync('$PET_ROOT/.claude-plugin/plugin.json','utf8')).version)}catch(e){console.log('unknown')}")
     # Get latest release tag
     LATEST=$(curl -sL https://api.github.com/repos/moeyui1/claude-status-pet/releases/latest | node -e "let d='';process.stdin.on('data',c=>d+=c);process.stdin.on('end',()=>{try{process.stdout.write(JSON.parse(d).tag_name||'unknown')}catch(e){process.stdout.write('unknown')}})" 2>/dev/null || echo 'unknown')
     ```
     Report both versions to the user. If they match, say "Already up to date." and stop.
  2. If outdated, update plugin: suggest the user run `/plugin install claude-status-pet`
  3. Update binary: delete and re-download:
     ```bash
     rm -f ~/.claude/pet-data/assets/.version ~/.claude/pet-data/bin/claude-status-pet*
     bash $PET_ROOT/scripts/download-assets.sh ~/.claude/pet-data/assets
     ```
  4. Update GIFs (if installed): delete version and re-download:
     ```bash
     rm -f ~/.claude/pet-data/assets/.gifs-version
     bash $PET_ROOT/scripts/download-gifs.sh ~/.claude/pet-data/assets
     ```
  5. Close and reopen: `taskkill //F //IM claude-status-pet.exe` then `bash $PET_ROOT/scripts/open-pet.sh`

Always give a short confirmation after executing.
