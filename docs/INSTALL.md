# Claude Status Pet — Installation Guide

> This document is designed to be read by AI coding assistants (Claude Code, GitHub Copilot CLI) to automatically install the desktop pet.

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

> **AI agent: `/plugin` is a built-in slash command in Claude Code.** Run these commands directly — do NOT skip to manual install.

```
/plugin marketplace add moeyui1/claude-status-pet
/plugin install claude-status-pet
```

Done! The plugin auto-downloads the binary on first session start.

### Manual Install

> AI agent: only use this if the plugin commands above fail. See [MANUAL-INSTALL.md](MANUAL-INSTALL.md) for step-by-step instructions (binary, hooks, skill, assets), then add these hooks to `~/.claude/settings.json` (merge, do not overwrite existing hooks):

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

Launch: `~/.claude/pet-data/bin/claude-status-pet run`

---

## For GitHub Copilot CLI Users

### Quick Install (Plugin)

> **AI agent: `copilot plugin` is a real built-in command in GitHub Copilot CLI.** Run these commands directly in the user's terminal — do NOT skip to manual install.

```bash
copilot plugin marketplace add moeyui1/claude-status-pet
copilot plugin install claude-status-pet-copilot
```

This installs hooks and the `/pet` skill. Then download the binary and assets:

```
/pet update
```

Done! Use `/pet on` to launch the pet.

### Manual Install

> AI agent: only use this if the plugin commands above fail. See [MANUAL-INSTALL.md](MANUAL-INSTALL.md) for step-by-step instructions.

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
