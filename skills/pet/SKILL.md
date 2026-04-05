---
name: pet
description: Manage your desktop pet — on, update, status
user-invocable: true
---

# /pet command

Manage the Claude Status Pet desktop companion.

## Usage

The user will run `/pet` with an optional subcommand. Parse the arguments and execute accordingly:

### Subcommands

- `/pet` or `/pet on` — Launch the pet (session selection handled by the app UI)
- `/pet update` — Update binary, hooks, skill, and assets to the latest release
- `/pet status` — Show current config, active sessions, and installed character packs
- `/pet uninstall` — Uninstall the pet completely (stop processes, remove all data, uninstall plugin)
- `/pet help` — Show available commands

> **Closing the pet:** Right-click the pet → Exit. There is no `/pet off` command.
>
> **Switching characters:** Right-click the pet to open the character menu.
>
> **Custom characters:** To create, install, or share custom character packs, read https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/docs/CUSTOM-CHARACTERS.md

## Implementation

> **Detect the user's platform and use appropriate commands.** On Windows use PowerShell, on macOS/Linux use bash. Do NOT use Node.js.

Key paths:
- **Windows**: `$env:USERPROFILE\.claude\pet-data\`
- **macOS/Linux**: `~/.claude/pet-data/`

Sub-paths:
- `bin/` — Pet binary (`claude-status-pet*`)
- `status-*.json` — Active session status files
- `pet-*.lock` — PID lock files (one per running pet window)
- `config.json` — User config (`character`)
- `assets/` — DLC characters (mona, kuromi)
- `characters/` — User-installed custom packs

### /pet on

Simply launch the pet binary. The binary has built-in PID lock detection, so duplicate windows are automatically prevented.

> **Claude Code** provides `${CLAUDE_SESSION_ID}` — use it to bind directly to the current session.
> **Copilot CLI / VS Code Copilot** do not provide a session ID variable — the pet will auto-bind to the most recently updated session.

**PowerShell (Claude Code):**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
$bin = Get-ChildItem "$dir\bin\claude-status-pet*" | Select-Object -First 1
if (-not $bin) { Write-Host "Pet binary not found"; return }
$sid = "${CLAUDE_SESSION_ID}"
$sf = "$dir\status-$sid.json"
$a = @("run","--status-file",$sf,"--session-id",$sid)
$assets = "$dir\assets"
if (Test-Path $assets) { $a += "--assets-dir"; $a += $assets }
Start-Process $bin.FullName -ArgumentList $a -WindowStyle Hidden
Write-Host "Pet launched"
```

**bash (Claude Code):**
```bash
DIR="$HOME/.claude/pet-data"
BIN=$(ls "$DIR/bin/claude-status-pet"* 2>/dev/null | head -1)
[ -z "$BIN" ] && echo "Pet binary not found" && exit 1
SID="${CLAUDE_SESSION_ID}"
SF="$DIR/status-$SID.json"
ARGS="run --status-file $SF --session-id $SID"
[ -d "$DIR/assets" ] && ARGS="$ARGS --assets-dir $DIR/assets"
nohup "$BIN" $ARGS >/dev/null 2>&1 &
echo "Pet launched"
```

**PowerShell (Copilot CLI):**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
$bin = Get-ChildItem "$dir\bin\claude-status-pet*" | Select-Object -First 1
if (-not $bin) { Write-Host "Pet binary not found"; return }
$a = @("run")
$assets = "$dir\assets"
if (Test-Path $assets) { $a += "--assets-dir"; $a += $assets }
Start-Process $bin.FullName -ArgumentList $a -WindowStyle Hidden
Write-Host "Pet launched"
```

**bash (Copilot CLI):**
```bash
DIR="$HOME/.claude/pet-data"
BIN=$(ls "$DIR/bin/claude-status-pet"* 2>/dev/null | head -1)
[ -z "$BIN" ] && echo "Pet binary not found" && exit 1
ARGS="run"
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
> - **VS Code Copilot (plugin install):** Run `Chat: Install Plugin From Source` with `https://github.com/moeyui1/claude-status-pet` — this updates hooks and skill automatically. Then follow steps 1-2 and 5 below for binary and assets.
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
Write-Host "[1/6] Stopped running pets"

# 2. Download binary
$binDir = "$dir\bin"
New-Item -ItemType Directory -Path $binDir -Force | Out-Null
$asset = "claude-status-pet-windows-x64.exe"
Invoke-WebRequest -Uri "$BASE/releases/latest/download/$asset" -OutFile "$binDir\$asset"
Write-Host "[2/6] Binary updated"

# 3. Update hooks (only for installed hook locations)
$hookUpdated = $false
# Copilot/VS Code global hooks
$copilotHooksDir = "$env:USERPROFILE\.copilot\hooks"
if (Test-Path $copilotHooksDir) {
    $copilotHookFile = "$copilotHooksDir\status-pet.json"
    $vscodeHookFile = "$copilotHooksDir\status-pet-vscode.json"

    if (Test-Path $copilotHookFile) {
        Invoke-WebRequest -Uri "$RAW/copilot/hooks.json" -OutFile $copilotHookFile
        $hookUpdated = $true
    }

    if (Test-Path $vscodeHookFile) {
        Invoke-WebRequest -Uri "$RAW/vscode/hooks/hooks.json" -OutFile $vscodeHookFile
        $hookUpdated = $true
    }

    if ($hookUpdated) { Write-Host "[3/6] Installed hooks updated" }
}
if (-not $hookUpdated) { Write-Host "[3/6] No hook locations to update (skipped)" }

# 3b. Update hook scripts
$scriptsDir = "$dir\scripts"
New-Item -ItemType Directory -Path $scriptsDir -Force | Out-Null
Invoke-WebRequest -Uri "$RAW/copilot/scripts/hook.sh" -OutFile "$scriptsDir\copilot-hook.sh"
Invoke-WebRequest -Uri "$RAW/copilot/scripts/hook.ps1" -OutFile "$scriptsDir\copilot-hook.ps1"
Write-Host "[4/6] Hook scripts updated"

# 4. Update skill
$skillDir = "$env:USERPROFILE\.claude\skills\pet"
New-Item -ItemType Directory -Path $skillDir -Force | Out-Null
Invoke-WebRequest -Uri "$RAW/skills/pet/SKILL.md" -OutFile "$skillDir\SKILL.md"
Write-Host "[5/6] Skill updated"

# 5. Update assets
$assetsDir = "$dir\assets"
New-Item -ItemType Directory -Path $assetsDir -Force | Out-Null
Invoke-WebRequest -Uri "$BASE/releases/latest/download/pet-assets.zip" -OutFile "$env:TEMP\pet-assets.zip"
Expand-Archive -Path "$env:TEMP\pet-assets.zip" -DestinationPath $assetsDir -Force
Remove-Item "$env:TEMP\pet-assets.zip" -ErrorAction SilentlyContinue
Write-Host "[6/6] Assets updated"

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
echo "[1/6] Stopped running pets"

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
ln -sf "$ASSET" "$DIR/bin/claude-status-pet" 2>/dev/null || true
echo "[2/6] Binary updated"

# 3. Update hooks (only for installed hook locations)
HOOK_UPDATED=0
if [ -d "$HOME/.copilot/hooks" ]; then
  curl -sLo "$HOME/.copilot/hooks/status-pet.json" "$RAW/copilot/hooks.json"
  curl -sLo "$HOME/.copilot/hooks/status-pet-vscode.json" "$RAW/vscode/hooks/hooks.json"
  HOOK_UPDATED=1; echo "[3/6] Copilot and VS Code hooks updated"
fi
[ "$HOOK_UPDATED" -eq 0 ] && echo "[3/6] No hook locations to update (skipped)"

# 3b. Update hook scripts
mkdir -p "$DIR/scripts"
curl -sLo "$DIR/scripts/copilot-hook.sh" "$RAW/copilot/scripts/hook.sh"
curl -sLo "$DIR/scripts/copilot-hook.ps1" "$RAW/copilot/scripts/hook.ps1"
chmod +x "$DIR/scripts/copilot-hook.sh" 2>/dev/null || true
echo "[4/6] Hook scripts updated"

# 4. Update skill
mkdir -p "$HOME/.claude/skills/pet"
curl -sLo "$HOME/.claude/skills/pet/SKILL.md" "$RAW/skills/pet/SKILL.md"
echo "[5/6] Skill updated"

# 5. Update assets
mkdir -p "$DIR/assets"
curl -sLo /tmp/pet-assets.zip "$BASE/releases/latest/download/pet-assets.zip"
unzip -o /tmp/pet-assets.zip -d "$DIR/assets"
rm -f /tmp/pet-assets.zip
echo "[6/6] Assets updated"

echo "Update complete! Run /pet on to start."
```

Tell the user: "Update complete! Use `/pet on` to start the pet."

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

Always give a short confirmation after executing.

### /pet uninstall

> **Important:** Confirm with the user before proceeding. This is destructive and cannot be undone.

**PowerShell:**
```powershell
# 1. Stop running pets
Get-Process | Where-Object { $_.ProcessName -like "claude-status-pet*" } | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }
Start-Sleep -Milliseconds 500
Write-Host "[1/4] Stopped running pets"

# 2. Remove pet-data (binary, scripts, assets, config, status files)
$dir = "$env:USERPROFILE\.claude\pet-data"
if (Test-Path $dir) { Remove-Item $dir -Recurse -Force; Write-Host "[2/4] Removed $dir" }
else { Write-Host "[2/4] $dir not found (skipped)" }

# 3. Remove copilot hooks
$copilotHook = "$env:USERPROFILE\.copilot\hooks\status-pet.json"
if (Test-Path $copilotHook) { Remove-Item $copilotHook -Force; Write-Host "[3/4] Removed Copilot hooks" }
else { Write-Host "[3/4] No Copilot hooks (skipped)" }

# 4. Remove skill
$skillDir = "$env:USERPROFILE\.claude\skills\pet"
if (Test-Path $skillDir) { Remove-Item $skillDir -Recurse -Force; Write-Host "[4/4] Removed skill" }
else { Write-Host "[4/4] No skill (skipped)" }

Write-Host "Uninstall complete."
```

**bash:**
```bash
# 1. Stop running pets
pkill -f claude-status-pet 2>/dev/null; sleep 0.5
echo "[1/4] Stopped running pets"

# 2. Remove pet-data
DIR="$HOME/.claude/pet-data"
if [ -d "$DIR" ]; then rm -rf "$DIR"; echo "[2/4] Removed $DIR"
else echo "[2/4] $DIR not found (skipped)"; fi

# 3. Remove copilot hooks
HOOK="$HOME/.copilot/hooks/status-pet.json"
if [ -f "$HOOK" ]; then rm -f "$HOOK"; echo "[3/4] Removed Copilot hooks"
else echo "[3/4] No Copilot hooks (skipped)"; fi

# 4. Remove skill
SKILL="$HOME/.claude/skills/pet"
if [ -d "$SKILL" ]; then rm -rf "$SKILL"; echo "[4/4] Removed skill"
else echo "[4/4] No skill (skipped)"; fi

echo "Uninstall complete."
```

After running, tell the user:
- "Pet uninstalled. All data, scripts, and assets have been removed."
- If using Claude Code plugin: "Run `/plugin uninstall claude-status-pet` to also remove the plugin hooks."
- If using Copilot plugin: "Run `copilot plugin uninstall claude-status-pet-copilot` to also remove the plugin."
