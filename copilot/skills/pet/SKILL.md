---
name: pet
description: Manage your desktop pet — on, update, toggle auto-start, install character packs
user-invocable: true
---

# /pet command

Manage the Claude Status Pet desktop companion.

## Usage

The user will run `/pet` with an optional subcommand. Parse the arguments and execute accordingly:

### Subcommands

- `/pet` or `/pet on` — Launch the pet (session selection handled by the app UI)
- `/pet update` — Update binary, hooks, skill, and assets to the latest release
- `/pet auto on` — Enable auto-start on new sessions (Claude Code only; not supported under Copilot — use `/pet on` manually)
- `/pet auto off` — Disable auto-start
- `/pet status` — Show current config, active sessions, and installed character packs
- `/pet pack install <url-or-path>` — Install a custom character pack
- `/pet pack list` — List all installed character packs
- `/pet pack remove <name>` — Remove a custom character pack
- `/pet help` — Show available commands

> **Note on auto-start:** Auto-start relies on `sessionStart` hooks, which only work in Claude Code. Under GitHub Copilot CLI, auto-start has no effect — use `/pet on` to manually launch the pet each session.
>
> **Closing the pet:** Right-click the pet → Exit. There is no `/pet off` command.
>
> **Switching characters:** Right-click the pet to open the character menu.

## Implementation

> **Detect the user's platform and use appropriate commands.** On Windows use PowerShell, on macOS/Linux use bash. Do NOT use Node.js.

Key paths:
- **Windows**: `$env:USERPROFILE\.claude\pet-data\`
- **macOS/Linux**: `~/.claude/pet-data/`

Sub-paths:
- `bin/` — Pet binary (`claude-status-pet*`)
- `status-*.json` — Active session status files
- `pet-*.lock` — PID lock files (one per running pet window)
- `config.json` — User config (`auto_start`, `character`)
- `assets/` — DLC characters (mona, kuromi)
- `characters/` — User-installed custom packs

### /pet on

Simply launch the pet binary. If there are multiple unlocked sessions, the app will show a session picker UI. The binary has built-in PID lock detection, so duplicate windows are automatically prevented.

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
$bin = Get-ChildItem "$dir\bin\claude-status-pet*" | Select-Object -First 1
if (-not $bin) { Write-Host "Pet binary not found"; return }
$a = @("run","--debug")
$assets = "$dir\assets"
if (Test-Path $assets) { $a += "--assets-dir"; $a += $assets }
Start-Process $bin.FullName -ArgumentList $a -WindowStyle Hidden
Write-Host "Pet launched"
```

**bash:**
```bash
DIR="$HOME/.claude/pet-data"
BIN=$(ls "$DIR/bin/claude-status-pet"* 2>/dev/null | head -1)
[ -z "$BIN" ] && echo "Pet binary not found" && exit 1
ARGS="run --debug"
[ -d "$DIR/assets" ] && ARGS="$ARGS --assets-dir $DIR/assets"
nohup "$BIN" $ARGS >/dev/null 2>&1 &
echo "Pet launched"
```

### /pet update

Update the pet to the latest release.

> **Detect the agent platform and use the appropriate update method:**
>
> - **Claude Code (plugin install):** Run `/plugin marketplace update claude-status-pet` — this updates the binary and hooks automatically. Then only update the skill file (step 4 below).
> - **GitHub Copilot CLI (plugin install):** Run `copilot plugin update claude-status-pet-copilot` in a terminal — this updates hooks and skill automatically. Then follow steps 1-2 and 5 below for binary and assets.
> - **Manual install (either agent):** Follow all steps below.

> **Important:** Close all running pets before updating. After updating, inform the user to restart with `/pet on`.

**PowerShell:**
```powershell
$REPO = "moeyui1/claude-status-pet"
$BASE = "https://github.com/$REPO"
$RAW  = "https://raw.githubusercontent.com/$REPO/main"
$dir  = "$env:USERPROFILE\.claude\pet-data"

# 1. Close running pets
Get-Process | Where-Object { $_.ProcessName -like "claude-status-pet*" } | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }
Start-Sleep -Milliseconds 500
Write-Host "[1/5] Stopped running pets"

# 2. Download binary
$binDir = "$dir\bin"
New-Item -ItemType Directory -Path $binDir -Force | Out-Null
$asset = "claude-status-pet-windows-x64.exe"
Invoke-WebRequest -Uri "$BASE/releases/latest/download/$asset" -OutFile "$binDir\$asset"
Write-Host "[2/5] Binary updated"

# 3. Update hooks (only for installed hook locations)
$hookUpdated = $false
# Copilot global hooks
$copilotHooksDir = "$env:USERPROFILE\.copilot\hooks"
if (Test-Path $copilotHooksDir) {
    Invoke-WebRequest -Uri "$RAW/copilot/hooks.json" -OutFile "$copilotHooksDir\status-pet.json"
    $hookUpdated = $true; Write-Host "[3/5] Copilot hooks updated"
}
if (-not $hookUpdated) { Write-Host "[3/5] No hook locations to update (skipped)" }

# 4. Update skill
$skillDir = "$env:USERPROFILE\.claude\skills\pet"
New-Item -ItemType Directory -Path $skillDir -Force | Out-Null
Invoke-WebRequest -Uri "$RAW/skills/pet/SKILL.md" -OutFile "$skillDir\SKILL.md"
Write-Host "[4/5] Skill updated"

# 5. Update assets
$assetsDir = "$dir\assets"
New-Item -ItemType Directory -Path $assetsDir -Force | Out-Null
Invoke-WebRequest -Uri "$BASE/releases/latest/download/pet-assets.zip" -OutFile "$env:TEMP\pet-assets.zip"
Expand-Archive -Path "$env:TEMP\pet-assets.zip" -DestinationPath $assetsDir -Force
Remove-Item "$env:TEMP\pet-assets.zip" -ErrorAction SilentlyContinue
Write-Host "[5/5] Assets updated"

Write-Host "Update complete! Run /pet on to start."
```

**bash:**
```bash
REPO="moeyui1/claude-status-pet"
BASE="https://github.com/$REPO"
RAW="https://raw.githubusercontent.com/$REPO/main"
DIR="$HOME/.claude/pet-data"

# 1. Close running pets
pkill -f claude-status-pet 2>/dev/null; sleep 0.5
echo "[1/5] Stopped running pets"

# 2. Download binary
mkdir -p "$DIR/bin"
OS=$(uname -s); ARCH=$(uname -m)
case "$OS" in
  Darwin) [ "$ARCH" = "arm64" ] && ASSET="claude-status-pet-macos-arm64" || ASSET="claude-status-pet-macos-x64" ;;
  Linux)  ASSET="claude-status-pet-linux-x64" ;;
  MINGW*|MSYS*|CYGWIN*|*_NT-*) ASSET="claude-status-pet-windows-x64.exe" ;;
esac
curl -sLo "$DIR/bin/$ASSET" "$BASE/releases/latest/download/$ASSET"
chmod +x "$DIR/bin/$ASSET" 2>/dev/null || true
echo "[2/5] Binary updated"

# 3. Update hooks (only for installed hook locations)
HOOK_UPDATED=0
if [ -d "$HOME/.copilot/hooks" ]; then
  curl -sLo "$HOME/.copilot/hooks/status-pet.json" "$RAW/copilot/hooks.json"
  HOOK_UPDATED=1; echo "[3/5] Copilot hooks updated"
fi
[ "$HOOK_UPDATED" -eq 0 ] && echo "[3/5] No hook locations to update (skipped)"

# 4. Update skill
mkdir -p "$HOME/.claude/skills/pet"
curl -sLo "$HOME/.claude/skills/pet/SKILL.md" "$RAW/skills/pet/SKILL.md"
echo "[4/5] Skill updated"

# 5. Update assets
mkdir -p "$DIR/assets"
curl -sLo /tmp/pet-assets.zip "$BASE/releases/latest/download/pet-assets.zip"
unzip -o /tmp/pet-assets.zip -d "$DIR/assets"
rm -f /tmp/pet-assets.zip
echo "[5/5] Assets updated"

echo "Update complete! Run /pet on to start."
```

Tell the user: "Update complete! Use `/pet on` to start the pet."

### /pet auto on/off

**PowerShell:**
```powershell
$cfg = "$env:USERPROFILE\.claude\pet-data\config.json"
$c = @{}; if (Test-Path $cfg) { $c = Get-Content $cfg | ConvertFrom-Json -AsHashtable }
$c.auto_start = $<true|false>
$c | ConvertTo-Json | Set-Content $cfg
Write-Host "Auto-start: <on|off>"
```

### /pet status

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
if (Test-Path "$dir\config.json") { Write-Host "Config:"; Get-Content "$dir\config.json" }
$sessions = Get-ChildItem "$dir\status-*.json" -ErrorAction SilentlyContinue
Write-Host "Active sessions: $($sessions.Count)"
$sessions | ForEach-Object { Write-Host "  $($_.Name)" }
foreach ($sub in @("assets","characters")) {
    $d = "$dir\$sub"
    if (-not (Test-Path $d)) { continue }
    Get-ChildItem $d -Directory | Where-Object { Test-Path "$($_.FullName)\character.json" } | ForEach-Object {
        $cfg = Get-Content "$($_.FullName)\character.json" | ConvertFrom-Json
        $label = if ($sub -eq "assets") { "DLC" } else { "Custom" }
        Write-Host "${label}: $($cfg.name) ($($_.Name))"
    }
}
```

### /pet pack list

Same as the status command's pack listing section above.

### /pet pack install <url-or-path>

**From URL (zip):**

**PowerShell:**
```powershell
$charsDir = "$env:USERPROFILE\.claude\pet-data\characters"
New-Item -ItemType Directory -Path $charsDir -Force | Out-Null
$tmp = "$env:TEMP\pet-pack.zip"
Invoke-WebRequest -Uri "<URL>" -OutFile $tmp
Expand-Archive -Path $tmp -DestinationPath $charsDir -Force
Remove-Item $tmp
Write-Host "Pack installed. Restart pet to see it."
```

**From local path:**
```powershell
Copy-Item -Recurse "<LOCAL_PATH>" "$env:USERPROFILE\.claude\pet-data\characters\"
Write-Host "Pack installed. Restart pet to see it."
```

Tell the user: "Restart the pet (right-click → Exit, then `/pet on`) to see the new character under Custom."

### /pet pack remove <name>

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data\characters\<NAME>"
if (Test-Path $dir) { Remove-Item $dir -Recurse -Force; Write-Host "Removed: <NAME>" }
else { Write-Host "Pack not found: <NAME>" }
```

Always give a short confirmation after executing.
