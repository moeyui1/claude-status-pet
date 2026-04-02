# Claude Status Pet — Installation Guide

> This document is designed to be read by any AI coding assistant (Claude Code, GitHub Copilot, Cursor, etc.) to automatically install the desktop pet.
>
> **No `gh` CLI required** — all downloads use `curl`, which is available on macOS, Linux, Git Bash, and WSL.

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

#### 2. Download scripts

```bash
SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
mkdir -p "$SCRIPTS_DIR"
for f in status-writer.sh launch-pet.sh open-pet.sh download-assets.sh download-assets.js download-gifs.js; do
  curl -sLo "$SCRIPTS_DIR/$f" "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/scripts/$f"
  chmod +x "$SCRIPTS_DIR/$f"
done
```

#### 3. Add hooks to `~/.claude/settings.json`

Merge these hooks (do not overwrite existing hooks):

```json
{
  "hooks": {
    "UserPromptSubmit": [{ "hooks": [{ "type": "command", "command": "bash ~/.claude/pet-data/scripts/status-writer.sh", "async": true }] }],
    "PreToolUse": [{ "hooks": [{ "type": "command", "command": "bash ~/.claude/pet-data/scripts/status-writer.sh", "async": true }] }],
    "Stop": [{ "hooks": [{ "type": "command", "command": "bash ~/.claude/pet-data/scripts/status-writer.sh", "async": true }] }],
    "Notification": [{ "matcher": "permission_prompt", "hooks": [{ "type": "command", "command": "bash ~/.claude/pet-data/scripts/status-writer.sh", "async": true }] }],
    "StopFailure": [{ "hooks": [{ "type": "command", "command": "bash ~/.claude/pet-data/scripts/status-writer.sh", "async": true }] }],
    "SessionStart": [{ "matcher": "startup", "hooks": [{ "type": "command", "command": "bash ~/.claude/pet-data/scripts/launch-pet.sh", "async": true }] }],
    "SessionEnd": [{ "hooks": [{ "type": "command", "command": "bash ~/.claude/pet-data/scripts/status-writer.sh", "async": true }] }]
  }
}
```

#### 4. Launch

```bash
bash ~/.claude/pet-data/scripts/open-pet.sh
```

---

## For VS Code Copilot Users (Global Install)

> AI agent: this method installs hooks globally via VS Code user settings, so the pet works in **all workspaces** without per-repo configuration.

### Prerequisites

- VS Code with GitHub Copilot extension
- Node.js (used by the hook script for JSON parsing)
- `curl` available in PATH (included in Windows 10+, macOS, and Linux)

### Steps

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

#### 2. Download Copilot hook scripts

```bash
SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
mkdir -p "$SCRIPTS_DIR"
curl -sLo "$SCRIPTS_DIR/copilot-status-writer.sh" \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/status-writer.sh"
curl -sLo "$SCRIPTS_DIR/copilot-status-writer.js" \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/status-writer.js"
chmod +x "$SCRIPTS_DIR/copilot-status-writer.sh"
for f in download-assets.js download-gifs.js; do
  curl -sLo "$SCRIPTS_DIR/$f" "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/scripts/$f"
done
```

#### 3. Install global hooks file

```bash
HOOKS_DIR="$HOME/.copilot/hooks"
mkdir -p "$HOOKS_DIR"
curl -sLo "$HOOKS_DIR/status-pet.json" \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/hooks.json"
```

#### 4. Add hook location to VS Code user settings

> AI agent: merge this into the user's VS Code `settings.json` (do not overwrite existing settings). The settings file is at:
> - **Windows**: `%APPDATA%\Code\User\settings.json`
> - **macOS**: `~/Library/Application Support/Code/User/settings.json`
> - **Linux**: `~/.config/Code/User/settings.json`

Add this setting:

```json
{
  "chat.hookFilesLocations": {
    "~/.copilot/hooks": true
  }
}
```

This tells VS Code to load all `*.json` hook files from `~/.copilot/hooks/` globally, across all workspaces.

#### 5. Download assets

```bash
SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
node "$SCRIPTS_DIR/download-assets.js"
```

---

## For GitHub Copilot Users (Per-Repo Install)

> AI agent: use this method if the user prefers per-repo hooks instead of global. Hooks are stored in `.github/hooks/` and must be added to each repo.

### Quick Install

```bash
bash <(curl -sL https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/setup.sh)
git add .github/hooks/status-pet.json && git commit -m "Add status pet hooks"
```

### Manual Install

> AI agent: follow these steps.

#### 1. Download binary (same as above)

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

#### 2. Download Copilot hook scripts

```bash
SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
mkdir -p "$SCRIPTS_DIR"
curl -sLo "$SCRIPTS_DIR/copilot-status-writer.sh" \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/status-writer.sh"
curl -sLo "$SCRIPTS_DIR/copilot-status-writer.js" \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/status-writer.js"
chmod +x "$SCRIPTS_DIR/copilot-status-writer.sh"
for f in download-assets.js download-gifs.js; do
  curl -sLo "$SCRIPTS_DIR/$f" "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/scripts/$f"
done
```

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
node "$SCRIPTS_DIR/download-assets.js"
```

---

## Verification

After install, confirm:
1. Binary exists: `ls ~/.claude/pet-data/bin/claude-status-pet*`
2. Launch: `~/.claude/pet-data/bin/claude-status-pet* --status-file ~/.claude/pet-data/status-test.json --session-id test`

Tell the user: "Your desktop pet is installed! It will appear on your screen and react to what your AI assistant is doing. Right-click to change characters or settings."
