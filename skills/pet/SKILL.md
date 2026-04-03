---
name: pet
description: Manage your desktop pet — open, close, set character, toggle auto-start, install character packs
user-invocable: true
---

# /pet command

Manage the Claude Status Pet desktop companion.

## Usage

The user will run `/pet` with an optional subcommand. Parse the arguments and execute accordingly:

### Subcommands

- `/pet` or `/pet open` — Open pet(s) for all active sessions
- `/pet close` — Close all running pets  
- `/pet set <character>` — Switch character. Available: `ferris`, `mona`, `kuromi`, `chonk`, `cat`, `ghost`, `robot`, `duck`, `snail`, `axolotl`, or any installed custom pack name
- `/pet auto on` — Enable auto-start on new sessions
- `/pet auto off` — Disable auto-start
- `/pet status` — Show current config, active sessions, and installed character packs
- `/pet update` — Update binary and assets to latest release
- `/pet pack install <url-or-path>` — Install a custom character pack
- `/pet pack list` — List all installed character packs
- `/pet pack remove <name>` — Remove a custom character pack
- `/pet pack create <name>` — Create a character pack template
- `/pet help` — Show available commands

## Implementation

Use Bash to execute the actions. First, determine the plugin root directory:

```bash
# Try plugin root, then common install locations
PET_ROOT="${CLAUDE_PLUGIN_ROOT:-$(find "$HOME" -maxdepth 3 -name "claude-status-pet" -type d 2>/dev/null | head -1)}"
```

Key paths:
- **Pet binary**: `~/.claude/pet-data/bin/claude-status-pet*`
- **Status files**: `~/.claude/pet-data/status-*.json` (one per session)
- **Config file**: `~/.claude/pet-data/config.json`
- **Assets dir**: `~/.claude/pet-data/assets/` (DLC characters: mona, kuromi)
- **Custom packs dir**: `~/.claude/pet-data/characters/` (user-installed packs)
- **Open script**: `bash $PET_ROOT/scripts/open-pet.sh`

### Basic commands

For `open`: run the open-pet.sh script.
For `close`: run `taskkill //F //IM claude-status-pet.exe` on Windows, or `pkill claude-status-pet` on macOS/Linux.
For `set <char>`: update `config.json` with the character name, and inform the user to right-click the pet to switch in the current session.
For `auto on/off`: update `config.json` field `auto_start` to true/false.
For `status`: read config.json, list status-*.json files, and list installed packs from `assets/` and `characters/` dirs.
For `update`:
  1. Delete old binary and version files:
     ```bash
     rm -f ~/.claude/pet-data/assets/.version ~/.claude/pet-data/bin/claude-status-pet*
     node $PET_ROOT/scripts/download-assets.js ~/.claude/pet-data/assets
     ```
  2. Close and reopen: kill all pet processes then `bash $PET_ROOT/scripts/open-pet.sh`

### Character Pack commands

#### `/pet pack list`

List all installed character packs:

```bash
echo "=== DLC Characters ==="
for d in ~/.claude/pet-data/assets/*/; do
  [ -f "$d/character.json" ] && node -e "const c=JSON.parse(require('fs').readFileSync('$d/character.json','utf8'));console.log('  ' + c.name + ' (' + c.type + ')')"
done
echo "=== Custom Characters ==="
for d in ~/.claude/pet-data/characters/*/; do
  [ -f "$d/character.json" ] && node -e "const c=JSON.parse(require('fs').readFileSync('$d/character.json','utf8'));console.log('  ' + c.name + ' (' + c.type + ')')"
done
```

#### `/pet pack install <url-or-path>`

Install a custom character pack from a URL (zip) or local path:

**From URL (GitHub release, direct zip link, etc.):**
```bash
PACK_URL="<user-provided-url>"
CHARS_DIR="$HOME/.claude/pet-data/characters"
mkdir -p "$CHARS_DIR"
TMPZIP="$(mktemp).zip"
curl -sLo "$TMPZIP" "$PACK_URL"
unzip -o "$TMPZIP" -d "$CHARS_DIR/"
rm -f "$TMPZIP"
```

On Windows PowerShell:
```powershell
$charsDir = "$env:USERPROFILE\.claude\pet-data\characters"
New-Item -ItemType Directory -Path $charsDir -Force | Out-Null
$tmpZip = "$env:TEMP\pet-pack.zip"
Invoke-WebRequest -Uri "<url>" -OutFile $tmpZip
Expand-Archive -Path $tmpZip -DestinationPath $charsDir -Force
Remove-Item $tmpZip
```

**From local path:**
```bash
cp -r "<local-path>" "$HOME/.claude/pet-data/characters/"
```

After installing, verify the pack has a valid `character.json`:
```bash
PACK_DIR="$HOME/.claude/pet-data/characters/<pack-name>"
node -e "const c=JSON.parse(require('fs').readFileSync('$PACK_DIR/character.json','utf8'));console.log('Installed: ' + c.name + ' (' + c.type + ', ' + Object.keys(c.states).length + ' states)')"
```

Tell the user: "Character pack installed! Right-click the pet → look under **Custom** section to select it. If the pet is not running, use `/pet open`."

#### `/pet pack remove <name>`

Remove a custom character pack:

```bash
PACK_DIR="$HOME/.claude/pet-data/characters/<name>"
if [ -d "$PACK_DIR" ]; then
  rm -rf "$PACK_DIR"
  echo "Removed character pack: <name>"
else
  echo "Pack not found: <name>"
fi
```

#### `/pet pack create <name>`

Create a character pack template for the user to fill in:

```bash
PACK_DIR="$HOME/.claude/pet-data/characters/<name>"
mkdir -p "$PACK_DIR"
cat > "$PACK_DIR/character.json" << 'EOF'
{
  "name": "<Name>",
  "type": "gif",
  "states": {
    "idle":       ["<name>/idle.gif"],
    "thinking":   ["<name>/thinking.gif"],
    "reading":    ["<name>/reading.gif"],
    "editing":    ["<name>/editing.gif"],
    "searching":  ["<name>/searching.gif"],
    "running":    ["<name>/running.gif"],
    "delegating": ["<name>/delegating.gif"],
    "waiting":    ["<name>/waiting.gif"],
    "error":      ["<name>/error.gif"],
    "offline":    ["<name>/offline.gif"],
    "unknown":    ["<name>/idle.gif"]
  }
}
EOF
```

Tell the user:
1. Template created at `~/.claude/pet-data/characters/<name>/`
2. Add your GIF/SVG/PNG images to that directory
3. Edit `character.json` to match your image filenames
4. Restart the pet to see it in the Custom menu section
5. To share: zip the `<name>/` directory and send it to others

Always give a short confirmation after executing.
