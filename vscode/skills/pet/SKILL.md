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

> VS Code Copilot does not provide a session ID variable — the pet will auto-bind to the most recently updated session.

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
$bin = Get-ChildItem "$dir\bin\claude-status-pet*" | Select-Object -First 1
if (-not $bin) { Write-Host "Pet binary not found. Run /pet update first."; return }
$a = @("run")
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
ARGS="run"
[ -d "$DIR/assets" ] && ARGS="$ARGS --assets-dir $DIR/assets"
nohup "$BIN" $ARGS >/dev/null 2>&1 &
echo "Pet launched"
```

### /pet update

Update the pet to the latest release.

> **For plugin install:** Run `Chat: Install Plugin From Source` with `https://github.com/moeyui1/claude-status-pet` — this updates hooks and skill automatically. Then follow steps 1-2 and 5 below for binary and assets.
>
> **For manual install:** Follow all steps below.

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
Copy-Item "$binDir\$asset" "$binDir\claude-status-pet" -Force
Write-Host "[2/5] Binary updated"

# 3. Update hooks (only for installed hook locations)
$hookUpdated = $false
$copilotHooksDir = "$env:USERPROFILE\.copilot\hooks"
if (Test-Path $copilotHooksDir) {
    $vscodeHookFile = "$copilotHooksDir\status-pet-vscode.json"
    if (Test-Path $vscodeHookFile) {
        Invoke-WebRequest -Uri "$RAW/vscode/hooks/hooks.json" -OutFile $vscodeHookFile
        $hookUpdated = $true; Write-Host "[3/5] VS Code hooks updated"
    }
}
if (-not $hookUpdated) { Write-Host "[3/5] No hook locations to update (skipped)" }

# 4. Update skill
$skillDir = "$env:USERPROFILE\.claude\skills\pet"
New-Item -ItemType Directory -Path $skillDir -Force | Out-Null
Invoke-WebRequest -Uri "$RAW/vscode/skills/pet/SKILL.md" -OutFile "$skillDir\SKILL.md"
Write-Host "[4/5] Skill updated"

# 5. Update assets
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
ln -sf "$DIR/bin/$ASSET" "$DIR/bin/claude-status-pet" 2>/dev/null || true
echo "[2/5] Binary updated"

# 3. Update hooks (only for installed hook locations)
HOOK_UPDATED=0
if [ -f "$HOME/.copilot/hooks/status-pet-vscode.json" ]; then
  curl -sLo "$HOME/.copilot/hooks/status-pet-vscode.json" "$RAW/vscode/hooks/hooks.json"
  HOOK_UPDATED=1; echo "[3/5] VS Code hooks updated"
fi
[ "$HOOK_UPDATED" -eq 0 ] && echo "[3/5] No hook locations to update (skipped)"

# 4. Update skill
mkdir -p "$HOME/.claude/skills/pet"
curl -sLo "$HOME/.claude/skills/pet/SKILL.md" "$RAW/vscode/skills/pet/SKILL.md"
echo "[4/5] Skill updated"

# 5. Update assets
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
echo "[5/5] Assets updated"

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
if (Test-Path $dir) { Remove-Item $dir -Recurse -Force; Write-Host "[2/6] Removed $dir" }
else { Write-Host "[2/6] $dir not found (skipped)" }

# 3. Remove VS Code hooks
$vscodeHook = "$env:USERPROFILE\.copilot\hooks\status-pet-vscode.json"
if (Test-Path $vscodeHook) { Remove-Item $vscodeHook -Force; Write-Host "[3/6] Removed VS Code hooks" }
else { Write-Host "[3/6] No VS Code hooks (skipped)" }

# 4. Remove skill
$skillDir = "$env:USERPROFILE\.claude\skills\pet"
if (Test-Path $skillDir) { Remove-Item $skillDir -Recurse -Force; Write-Host "[4/6] Removed skill" }
else { Write-Host "[4/6] No skill (skipped)" }

# 5. Remove VS Code agent plugin
$agentPlugin = "$env:USERPROFILE\.vscode\agent-plugins\github.com\moeyui1\claude-status-pet"
if (Test-Path $agentPlugin) { Remove-Item $agentPlugin -Recurse -Force; Write-Host "[5/6] Removed VS Code plugin" }
else { Write-Host "[5/6] No VS Code plugin (skipped)" }

# 6. Remove plugin registration from copilot config
$configPath = "$env:USERPROFILE\.copilot\config.json"
if (Test-Path $configPath) {
    $config = Get-Content $configPath -Raw | ConvertFrom-Json
    $changed = $false
    if ($config.installed_plugins) {
        $before = $config.installed_plugins.Count
        $config.installed_plugins = @($config.installed_plugins | Where-Object { $_.name -notlike "claude-status-pet*" })
        if ($config.installed_plugins.Count -ne $before) { $changed = $true }
    }
    if ($config.marketplaces -and $config.marketplaces.PSObject.Properties["claude-status-pet"]) {
        $config.marketplaces.PSObject.Properties.Remove("claude-status-pet"); $changed = $true
    }
    if ($changed) { $config | ConvertTo-Json -Depth 10 | Set-Content $configPath -Encoding UTF8; Write-Host "[6/6] Cleaned plugin registry" }
    else { Write-Host "[6/6] Plugin registry already clean" }
} else { Write-Host "[6/6] No config (skipped)" }

Write-Host "Uninstall complete."
```

**bash:**
```bash
# 1. Stop running pets
pkill -f claude-status-pet 2>/dev/null; sleep 0.5
echo "[1/6] Stopped running pets"

# 2. Remove pet-data
DIR="$HOME/.claude/pet-data"
if [ -d "$DIR" ]; then rm -rf "$DIR"; echo "[2/6] Removed $DIR"
else echo "[2/6] $DIR not found (skipped)"; fi

# 3. Remove VS Code hooks
HOOK="$HOME/.copilot/hooks/status-pet-vscode.json"
if [ -f "$HOOK" ]; then rm -f "$HOOK"; echo "[3/6] Removed VS Code hooks"
else echo "[3/6] No VS Code hooks (skipped)"; fi

# 4. Remove skill
SKILL="$HOME/.claude/skills/pet"
if [ -d "$SKILL" ]; then rm -rf "$SKILL"; echo "[4/6] Removed skill"
else echo "[4/6] No skill (skipped)"; fi

# 5. Remove VS Code agent plugin
AGENT_PLUGIN="$HOME/.vscode/agent-plugins/github.com/moeyui1/claude-status-pet"
if [ -d "$AGENT_PLUGIN" ]; then rm -rf "$AGENT_PLUGIN"; echo "[5/6] Removed VS Code plugin"
else echo "[5/6] No VS Code plugin (skipped)"; fi

# 6. Remove plugin registration from copilot config
CONFIG="$HOME/.copilot/config.json"
if [ -f "$CONFIG" ] && command -v python3 >/dev/null 2>&1; then
  python3 -c "
import json, sys
with open('$CONFIG') as f: c = json.load(f)
changed = False
if 'installed_plugins' in c:
    before = len(c['installed_plugins'])
    c['installed_plugins'] = [p for p in c['installed_plugins'] if not p.get('name','').startswith('claude-status-pet')]
    if len(c['installed_plugins']) != before: changed = True
if 'marketplaces' in c and 'claude-status-pet' in c['marketplaces']:
    del c['marketplaces']['claude-status-pet']; changed = True
if changed:
    with open('$CONFIG','w') as f: json.dump(c, f, indent=2)
    print('[6/6] Cleaned plugin registry')
else: print('[6/6] Plugin registry already clean')
" 2>/dev/null || echo "[6/6] Could not clean config (skipped)"
else echo "[6/6] No config (skipped)"; fi

echo "Uninstall complete."
```

After running, tell the user:
- "Pet completely uninstalled. All data, plugins, and hooks have been removed."
