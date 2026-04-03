# Claude Status Pet — Installation Guide

> This document is designed to be read by any AI coding assistant (Claude Code, GitHub Copilot, Cursor, etc.) to automatically install the desktop pet.

## ⚠️ Important Notes for AI Agents

> **Cross-platform awareness:**
> - The bash scripts below work on **macOS, Linux, Git Bash, and WSL**
> - On **Windows PowerShell** (e.g. GitHub Copilot CLI, VS Code terminals), you MUST translate each bash command to PowerShell equivalents:
>   - `$HOME` → `$env:USERPROFILE`
>   - `mkdir -p` → `New-Item -ItemType Directory -Force`
>   - `curl -sLo <file> <url>` → `Invoke-WebRequest -Uri <url> -OutFile <file>`
>   - `chmod +x` → skip (not needed on Windows)
>   - Do NOT use `~` in PowerShell paths — it doesn't expand in all contexts
> - **Detect the user's shell** before running commands. If running in PowerShell, use PowerShell syntax.
>
> **Download reliability:**
> - The `/releases/latest/download/` URL may 404 briefly after a new release while CI builds. If download fails, retry after 30 seconds or use a specific version: `https://github.com/moeyui1/claude-status-pet/releases/download/v0.6.0/<asset>`
> - Always verify the downloaded file is not an HTML error page (check file size > 1MB for binary, > 1KB for scripts)

## For Claude Code Users

### Quick Install (Plugin)

```
/plugin marketplace add moeyui1/claude-status-pet
/plugin install claude-status-pet
```

Done! The plugin auto-downloads the binary on first session start.

### Manual Install

> AI agent: follow these steps if the plugin method is unavailable.

#### 1. Download binary

```bash
INSTALL_DIR="$HOME/.claude/pet-data/bin"
mkdir -p "$INSTALL_DIR"

OS=$(uname -s); ARCH=$(uname -m)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*|*_NT-*) ASSET="claude-status-pet-windows-x64.exe" ;;
  Darwin) [ "$ARCH" = "arm64" ] && ASSET="claude-status-pet-macos-arm64" || ASSET="claude-status-pet-macos-x64" ;;
  Linux) ASSET="claude-status-pet-linux-x64" ;;
esac

curl -sLo "$INSTALL_DIR/$ASSET" "https://github.com/moeyui1/claude-status-pet/releases/latest/download/$ASSET"
chmod +x "$INSTALL_DIR/$ASSET" 2>/dev/null || true
```

#### 2. Add hooks to `~/.claude/settings.json`

Merge these hooks (do not overwrite existing hooks):

```json
{
  "hooks": {
    "UserPromptSubmit": [{ "hooks": [{ "type": "command", "command": "~/.claude/pet-data/bin/claude-status-pet write-status --adapter claude", "async": true }] }],
    "PreToolUse": [{ "hooks": [{ "type": "command", "command": "~/.claude/pet-data/bin/claude-status-pet write-status --adapter claude", "async": true }] }],
    "Stop": [{ "hooks": [{ "type": "command", "command": "~/.claude/pet-data/bin/claude-status-pet write-status --adapter claude", "async": true }] }],
    "Notification": [{ "matcher": "permission_prompt", "hooks": [{ "type": "command", "command": "~/.claude/pet-data/bin/claude-status-pet write-status --adapter claude", "async": true }] }],
    "StopFailure": [{ "hooks": [{ "type": "command", "command": "~/.claude/pet-data/bin/claude-status-pet write-status --adapter claude", "async": true }] }],
    "SessionStart": [{ "matcher": "startup", "hooks": [{ "type": "command", "command": "~/.claude/pet-data/bin/claude-status-pet write-status --adapter claude", "async": true }] }],
    "SessionEnd": [{ "hooks": [{ "type": "command", "command": "~/.claude/pet-data/bin/claude-status-pet write-status --adapter claude", "async": true }] }]
  }
}
```

#### 3. Launch

```bash
~/.claude/pet-data/bin/claude-status-pet run --status-file ~/.claude/pet-data/status-test.json --session-id test --assets-dir ~/.claude/pet-data/assets
```

---

## For GitHub Copilot Users (Global Install)

> AI agent: this method installs hooks globally via VS Code user settings, so the pet works in **all workspaces** without per-repo configuration.
>
> **On Windows**, all commands below must be run in **PowerShell** (not bash). Translate accordingly — see the notes at the top of this document.

### Prerequisites

- VS Code with GitHub Copilot extension

### Steps

#### 1. Download binary

**macOS / Linux / Git Bash:**

```bash
INSTALL_DIR="$HOME/.claude/pet-data/bin"
mkdir -p "$INSTALL_DIR"

OS=$(uname -s); ARCH=$(uname -m)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*|*_NT-*) ASSET="claude-status-pet-windows-x64.exe" ;;
  Darwin) [ "$ARCH" = "arm64" ] && ASSET="claude-status-pet-macos-arm64" || ASSET="claude-status-pet-macos-x64" ;;
  Linux) ASSET="claude-status-pet-linux-x64" ;;
esac

curl -sLo "$INSTALL_DIR/$ASSET" "https://github.com/moeyui1/claude-status-pet/releases/latest/download/$ASSET"
chmod +x "$INSTALL_DIR/$ASSET"
```

**Windows PowerShell:**

```powershell
$installDir = "$env:USERPROFILE\.claude\pet-data\bin"
New-Item -ItemType Directory -Path $installDir -Force | Out-Null
$asset = "claude-status-pet-windows-x64.exe"
Invoke-WebRequest -Uri "https://github.com/moeyui1/claude-status-pet/releases/latest/download/$asset" -OutFile "$installDir\$asset"
```

#### 2. Install global hooks file

**macOS / Linux / Git Bash:**

```bash
HOOKS_DIR="$HOME/.copilot/hooks"
mkdir -p "$HOOKS_DIR"
curl -sLo "$HOOKS_DIR/status-pet.json" \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/hooks.json"
```

**Windows PowerShell:**

```powershell
$hooksDir = "$env:USERPROFILE\.copilot\hooks"
New-Item -ItemType Directory -Path $hooksDir -Force | Out-Null
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/hooks.json" -OutFile "$hooksDir\status-pet.json"
```

#### 3. Add hook location to VS Code user settings

> AI agent: merge this into the user's VS Code `settings.json` (do not overwrite existing settings). The settings file is at:
> - **Windows**: `%APPDATA%\Code\User\settings.json`
> - **macOS**: `~/Library/Application Support/Code/User/settings.json`
> - **Linux**: `~/.config/Code/User/settings.json`

Add this setting (merge, do not replace the file):

```json
{
  "chat.hookFilesLocations": {
    "~/.copilot/hooks": true
  }
}
```

This tells VS Code to load all `*.json` hook files from `~/.copilot/hooks/` globally, across all workspaces.

#### 4. Download assets

**macOS / Linux / Git Bash:**

```bash
ASSETS_DIR="$HOME/.claude/pet-data/assets"
mkdir -p "$ASSETS_DIR"
curl -sLo /tmp/pet-assets.zip "https://github.com/moeyui1/claude-status-pet/releases/latest/download/pet-assets.zip"
unzip -o /tmp/pet-assets.zip -d "$ASSETS_DIR"
rm -f /tmp/pet-assets.zip
```

**Windows PowerShell:**

```powershell
$assetsDir = "$env:USERPROFILE\.claude\pet-data\assets"
New-Item -ItemType Directory -Path $assetsDir -Force | Out-Null
Invoke-WebRequest -Uri "https://github.com/moeyui1/claude-status-pet/releases/latest/download/pet-assets.zip" -OutFile "$env:TEMP\pet-assets.zip"
Expand-Archive -Path "$env:TEMP\pet-assets.zip" -DestinationPath $assetsDir -Force
Remove-Item "$env:TEMP\pet-assets.zip"
```

---

## For GitHub Copilot Users (Per-Repo Install)

> AI agent: use this method if the user prefers per-repo hooks instead of global. Hooks are stored in `.github/hooks/` and must be added to each repo.

Follow steps 1-2 from the Global Install above, then:

#### 3. Add hooks to repo

```bash
mkdir -p .github/hooks
curl -sLo .github/hooks/status-pet.json \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/hooks.json"
git add .github/hooks/status-pet.json
git commit -m "Add status pet hooks for Copilot"
```

#### 4. Download assets

```bash
ASSETS_DIR="$HOME/.claude/pet-data/assets"
mkdir -p "$ASSETS_DIR"
curl -sLo /tmp/pet-assets.zip "https://github.com/moeyui1/claude-status-pet/releases/latest/download/pet-assets.zip"
unzip -o /tmp/pet-assets.zip -d "$ASSETS_DIR"
rm -f /tmp/pet-assets.zip
```

---

## Verification

After install, confirm:
1. Binary exists: `ls ~/.claude/pet-data/bin/claude-status-pet*` (bash) or `Get-Item "$env:USERPROFILE\.claude\pet-data\bin\claude-status-pet*"` (PowerShell)
2. Launch manually:

**bash:**
```bash
~/.claude/pet-data/bin/claude-status-pet* --status-file ~/.claude/pet-data/status-test.json --session-id test &
```

**PowerShell:**
```powershell
Start-Process "$env:USERPROFILE\.claude\pet-data\bin\claude-status-pet-windows-x64.exe" -ArgumentList "--status-file", "$env:USERPROFILE\.claude\pet-data\status-test.json", "--session-id", "test"
```

Tell the user: "Your desktop pet is installed! It will appear on your screen and react to what your AI assistant is doing. Right-click to change characters or settings."
