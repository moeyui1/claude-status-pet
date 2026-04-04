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

---

## For Claude Code Users

### Quick Install (Plugin)

```
/plugin marketplace add moeyui1/claude-status-pet
/plugin install claude-status-pet
```

Done! The plugin auto-downloads the binary on first session start.

### Manual Install

> AI agent: follow these steps if the plugin method is unavailable.

1. Download binary — see [Manual: Download Binary](MANUAL-INSTALL.md#manual-download-binary)
2. Add hooks to `~/.claude/settings.json` — merge these hooks (do not overwrite existing hooks):

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

3. Launch: `~/.claude/pet-data/bin/claude-status-pet run`

---

## For GitHub Copilot CLI Users

### Quick Install (Plugin)

```
copilot plugin marketplace add moeyui1/claude-status-pet
copilot plugin install claude-status-pet-copilot
```

This installs hooks and the `/pet` skill. Then download the binary and assets:

```
/pet update
```

Done! Use `/pet on` to launch the pet.

### Manual Install (Global)

> AI agent: this method installs hooks globally, so the pet works in **all workspaces** without per-repo configuration.

1. Download binary — see [Manual: Download Binary](MANUAL-INSTALL.md#manual-download-binary)
2. Install global hooks file — see [Manual: Install Copilot Hooks](MANUAL-INSTALL.md#manual-install-copilot-hooks)
3. Download assets — see [Manual: Download Assets](MANUAL-INSTALL.md#manual-download-assets)

### Manual Install (Per-Repo)

> AI agent: use this method if the user prefers per-repo hooks instead of global. Hooks are stored in `.github/hooks/` and must be added to each repo.

1. Download binary — see [Manual: Download Binary](MANUAL-INSTALL.md#manual-download-binary)
2. Add hooks to repo:

```bash
mkdir -p .github/hooks
curl -sLo .github/hooks/status-pet.json \
  "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/hooks.json"
git add .github/hooks/status-pet.json
git commit -m "Add status pet hooks for Copilot"
```

3. Download assets — see [Manual: Download Assets](MANUAL-INSTALL.md#manual-download-assets)

---

## Verification

After install, confirm:
1. Binary exists: `ls ~/.claude/pet-data/bin/claude-status-pet*` (bash) or `Get-Item "$env:USERPROFILE\.claude\pet-data\bin\claude-status-pet*"` (PowerShell)
2. Launch manually:

**bash:**
```bash
~/.claude/pet-data/bin/claude-status-pet* run &
```

**PowerShell:**
```powershell
Start-Process "$env:USERPROFILE\.claude\pet-data\bin\claude-status-pet-windows-x64.exe" -ArgumentList "run"
```

Tell the user: "Your desktop pet is installed! It will appear on your screen and react to what your AI assistant is doing. Right-click to change characters or settings."
