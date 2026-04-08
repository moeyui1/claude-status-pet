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
- `assets/` — DLC characters (mona, gopher, fluent-emoji, etc.)
- `characters/` — User-installed custom packs

### /pet on

Simply launch the pet binary. The binary has built-in PID lock detection, so duplicate windows are automatically prevented.

> Claude Code provides `${CLAUDE_SESSION_ID}` — use it to bind directly to the current session.

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
$bin = Get-ChildItem "$dir\bin\claude-status-pet*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $bin) { $bin = Get-ChildItem "$dir\bin\claude-status-pet*" | Select-Object -First 1 }
if (-not $bin) { Write-Host "Pet binary not found. Run /pet update first."; return }
Unblock-File $bin.FullName -ErrorAction SilentlyContinue
$sid = "${CLAUDE_SESSION_ID}"
$sf = "$dir\status-$sid.json"
$a = @("run","--status-file",$sf,"--session-id",$sid)
$assets = "$dir\assets"
if (Test-Path $assets) { $a += "--assets-dir"; $a += $assets }
Start-Process $bin.FullName -ArgumentList $a -WindowStyle Hidden
Write-Host "Pet launched"
```

**bash:**
```bash
DIR="$HOME/.claude/pet-data"
BIN=$(ls "$DIR/bin/claude-status-pet"* 2>/dev/null | head -1)
[ -z "$BIN" ] && echo "Pet binary not found. Run /pet update first." && exit 1
SID="${CLAUDE_SESSION_ID}"
SF="$DIR/status-$SID.json"
ARGS="run --status-file $SF --session-id $SID"
[ -d "$DIR/assets" ] && ARGS="$ARGS --assets-dir $DIR/assets"
nohup "$BIN" $ARGS >/dev/null 2>&1 &
echo "Pet launched"
```

### /pet update

Update the pet to the latest release.

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
Write-Host "[1/3] Stopped running pets"

# 2. Download binary
$binDir = "$dir\bin"
New-Item -ItemType Directory -Path $binDir -Force | Out-Null
$asset = "claude-status-pet-windows-x64.exe"
Invoke-WebRequest -Uri "$BASE/releases/latest/download/$asset" -OutFile "$binDir\$asset"
Unblock-File "$binDir\$asset"
Copy-Item "$binDir\$asset" "$binDir\claude-status-pet" -Force
Write-Host "[2/3] Binary updated"

# 3. Update assets
$assetsDir = "$dir\assets"
New-Item -ItemType Directory -Path $assetsDir -Force | Out-Null
Invoke-WebRequest -Uri "$BASE/releases/latest/download/pet-assets.zip" -OutFile "$env:TEMP\pet-assets.zip"
Expand-Archive -Path "$env:TEMP\pet-assets.zip" -DestinationPath $assetsDir -Force
Remove-Item "$env:TEMP\pet-assets.zip" -ErrorAction SilentlyContinue
# Clean outdated DLC so they re-download on next use
Get-ChildItem "$assetsDir\dlc\*.json" -ErrorAction SilentlyContinue | ForEach-Object {
    $dlcId = $_.BaseName; $charJson = "$assetsDir\$dlcId\character.json"
    if (Test-Path $charJson) {
        $cfg = Get-Content $_.FullName | ConvertFrom-Json
        $inst = Get-Content $charJson | ConvertFrom-Json
        if ($cfg.version -and (!$inst.version -or $inst.version -lt $cfg.version)) {
            Remove-Item "$assetsDir\$dlcId" -Recurse -Force
        }
    }
}
Write-Host "[3/3] Assets updated"

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
echo "[1/3] Stopped running pets"

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
ln -sf "$DIR/bin/$ASSET" "$DIR/bin/claude-status-pet" 2>/dev/null || true
echo "[2/3] Binary updated"

# 3. Update assets
mkdir -p "$DIR/assets"
curl -sLo /tmp/pet-assets.zip "$BASE/releases/latest/download/pet-assets.zip"
unzip -o /tmp/pet-assets.zip -d "$DIR/assets"
rm -f /tmp/pet-assets.zip
# Clean outdated DLC so they re-download on next use
for cfg in "$DIR/assets/dlc/"*.json; do
  [ -f "$cfg" ] || continue
  id=$(basename "$cfg" .json)
  cj="$DIR/assets/$id/character.json"
  [ -f "$cj" ] || continue
  cv=$(grep -o '"version"[[:space:]]*:[[:space:]]*[0-9]*' "$cfg" | grep -o '[0-9]*')
  iv=$(grep -o '"version"[[:space:]]*:[[:space:]]*[0-9]*' "$cj" | grep -o '[0-9]*')
  [ -n "$cv" ] && { [ -z "$iv" ] || [ "$iv" -lt "$cv" ]; } && rm -rf "$DIR/assets/$id"
done
echo "[3/3] Assets updated"

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
Write-Host "[1/3] Stopped running pets"

# 2. Remove pet-data (binary, scripts, assets, config, status files)
$dir = "$env:USERPROFILE\.claude\pet-data"
if (Test-Path $dir) { Remove-Item $dir -Recurse -Force; Write-Host "[2/3] Removed $dir" }
else { Write-Host "[2/3] $dir not found (skipped)" }

# 3. Remove skill
$skillDir = "$env:USERPROFILE\.claude\skills\pet"
if (Test-Path $skillDir) { Remove-Item $skillDir -Recurse -Force; Write-Host "[3/3] Removed skill" }
else { Write-Host "[3/3] No skill (skipped)" }

Write-Host "Uninstall complete."
```

**bash:**
```bash
# 1. Stop running pets
pkill -f claude-status-pet 2>/dev/null; sleep 0.5
echo "[1/3] Stopped running pets"

# 2. Remove pet-data
DIR="$HOME/.claude/pet-data"
if [ -d "$DIR" ]; then rm -rf "$DIR"; echo "[2/3] Removed $DIR"
else echo "[2/3] $DIR not found (skipped)"; fi

# 3. Remove skill
SKILL="$HOME/.claude/skills/pet"
if [ -d "$SKILL" ]; then rm -rf "$SKILL"; echo "[3/3] Removed skill"
else echo "[3/3] No skill (skipped)"; fi

echo "Uninstall complete."
```

After running, tell the user:
- "Pet uninstalled. All data and assets have been removed."
- "To also remove the plugin and its hooks, run: `/plugin uninstall claude-status-pet`"
