# Claude Status Pet — Installation Guide

> This document is designed to be read by any AI coding assistant (Claude Code, GitHub Copilot, Cursor, etc.) to automatically install the desktop pet.

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

gh release download --repo moeyui1/claude-status-pet --pattern "$ASSET" --dir "$INSTALL_DIR" --clobber
chmod +x "$INSTALL_DIR/$ASSET" 2>/dev/null || true
```

#### 2. Download scripts

```bash
SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
mkdir -p "$SCRIPTS_DIR"
for f in status-writer.sh launch-pet.sh open-pet.sh download-assets.sh; do
  gh api repos/moeyui1/claude-status-pet/contents/scripts/$f \
    -H "Accept: application/vnd.github.raw+json" > "$SCRIPTS_DIR/$f"
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

## For GitHub Copilot Users

### Quick Install

```bash
bash <(curl -sL https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/setup.sh)
git add .github/hooks/status-pet.json && git commit -m "Add status pet hooks"
```

### Manual Install

> AI agent: follow these steps.

#### 1. Download binary (same as Claude Code)

```bash
INSTALL_DIR="$HOME/.claude/pet-data/bin"
mkdir -p "$INSTALL_DIR"

OS=$(uname -s); ARCH=$(uname -m)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*|*_NT-*) ASSET="claude-status-pet-windows-x64.exe" ;;
  Darwin) [ "$ARCH" = "arm64" ] && ASSET="claude-status-pet-macos-arm64" || ASSET="claude-status-pet-macos-x64" ;;
  Linux) ASSET="claude-status-pet-linux-x64" ;;
esac

gh release download --repo moeyui1/claude-status-pet --pattern "$ASSET" --dir "$INSTALL_DIR" --clobber
chmod +x "$INSTALL_DIR/$ASSET" 2>/dev/null || true
```

#### 2. Download Copilot hook script

```bash
SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
mkdir -p "$SCRIPTS_DIR"
gh api repos/moeyui1/claude-status-pet/contents/copilot/status-writer.sh \
  -H "Accept: application/vnd.github.raw+json" > "$SCRIPTS_DIR/copilot-status-writer.sh"
chmod +x "$SCRIPTS_DIR/copilot-status-writer.sh"
```

#### 3. Add hooks to repo

```bash
mkdir -p .github/hooks
gh api repos/moeyui1/claude-status-pet/contents/copilot/hooks.json \
  -H "Accept: application/vnd.github.raw+json" > .github/hooks/status-pet.json
git add .github/hooks/status-pet.json
git commit -m "Add status pet hooks for Copilot"
```

#### 4. Download assets

```bash
gh api repos/moeyui1/claude-status-pet/contents/scripts/download-assets.sh \
  -H "Accept: application/vnd.github.raw+json" > "$SCRIPTS_DIR/download-assets.sh"
chmod +x "$SCRIPTS_DIR/download-assets.sh"
bash "$SCRIPTS_DIR/download-assets.sh"
```

---

## Verification

After install, confirm:
1. Binary exists: `ls ~/.claude/pet-data/bin/claude-status-pet*`
2. Launch: `~/.claude/pet-data/bin/claude-status-pet* --status-file ~/.claude/pet-data/status-test.json --session-id test`

Tell the user: "Your desktop pet is installed! It will appear on your screen and react to what your AI assistant is doing. Right-click to change characters or settings."
