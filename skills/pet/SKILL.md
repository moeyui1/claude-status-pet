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
- `/pet set <character>` — Switch character
- `/pet auto on` — Enable auto-start on new sessions
- `/pet auto off` — Disable auto-start
- `/pet status` — Show current config, active sessions, and installed character packs
- `/pet pack install <url-or-path>` — Install a custom character pack
- `/pet pack list` — List all installed character packs
- `/pet pack remove <name>` — Remove a custom character pack
- `/pet help` — Show available commands

## Implementation

> **Detect the user's platform and use appropriate commands.** On Windows use PowerShell, on macOS/Linux use bash. Do NOT use Node.js.

Key paths:
- **Windows**: `$env:USERPROFILE\.claude\pet-data\`
- **macOS/Linux**: `~/.claude/pet-data/`

Sub-paths:
- `bin/` — Pet binary (`claude-status-pet*`)
- `status-*.json` — Active session status files
- `config.json` — User config (`auto_start`, `character`)
- `assets/` — DLC characters (mona, kuromi)
- `characters/` — User-installed custom packs

### /pet open

Find pet binary and launch for each active session.

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
$bin = Get-ChildItem "$dir\bin\claude-status-pet*" | Select-Object -First 1
if (-not $bin) { Write-Host "Pet binary not found"; return }
$assets = "$dir\assets"
Get-ChildItem "$dir\status-*.json" | ForEach-Object {
    $sid = $_.BaseName -replace 'status-',''
    $args = @("run","--status-file",$_.FullName,"--session-id",$sid,"--debug")
    if (Test-Path $assets) { $args += "--assets-dir"; $args += $assets }
    Start-Process $bin.FullName -ArgumentList $args -WindowStyle Hidden
}
Write-Host "Pet(s) launched"
```

**bash:**
```bash
DIR="$HOME/.claude/pet-data"
BIN=$(ls "$DIR/bin/claude-status-pet"* 2>/dev/null | head -1)
[ -z "$BIN" ] && echo "Pet binary not found" && exit 1
for f in "$DIR"/status-*.json; do
  [ -f "$f" ] || continue
  SID=$(basename "$f" .json | sed 's/status-//')
  ARGS="run --status-file $f --session-id $SID --debug"
  [ -d "$DIR/assets" ] && ARGS="$ARGS --assets-dir $DIR/assets"
  nohup "$BIN" $ARGS >/dev/null 2>&1 &
done
echo "Pet(s) launched"
```

### /pet close

**PowerShell:**
```powershell
Get-Process | Where-Object { $_.ProcessName -like "claude-status-pet*" } | ForEach-Object { Stop-Process -Id $_.Id -Force }
Write-Host "Closed"
```

**bash:**
```bash
pkill -f claude-status-pet && echo "Closed" || echo "No pets running"
```

### /pet set <character>

Update `config.json` with character name, inform user to right-click pet to switch in current session.

**PowerShell:**
```powershell
$cfg = "$env:USERPROFILE\.claude\pet-data\config.json"
$c = @{}; if (Test-Path $cfg) { $c = Get-Content $cfg | ConvertFrom-Json -AsHashtable }
$c.character = "<CHAR>"
$c | ConvertTo-Json | Set-Content $cfg
Write-Host "Default character set to: <CHAR>"
```

**bash:**
```bash
CFG="$HOME/.claude/pet-data/config.json"
echo "{\"character\":\"<CHAR>\",\"auto_start\":$(cat "$CFG" 2>/dev/null | grep -o '"auto_start":[^,}]*' || echo 'false')}" > "$CFG"
echo "Default character set to: <CHAR>"
```

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

Tell the user: "Restart the pet (right-click → Exit, then `/pet open`) to see the new character under Custom."

### /pet pack remove <name>

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data\characters\<NAME>"
if (Test-Path $dir) { Remove-Item $dir -Recurse -Force; Write-Host "Removed: <NAME>" }
else { Write-Host "Pack not found: <NAME>" }
```

Always give a short confirmation after executing.
