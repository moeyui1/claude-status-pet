# Claude Status Pet — Installation Guide

> This document is designed to be read by AI coding assistants (Claude Code, GitHub Copilot CLI, VS Code Copilot) to automatically install the desktop pet.

## ⚠️ Important Notes for AI Agents

> **Cross-platform awareness:**
> - On **Windows PowerShell**, use PowerShell syntax (`$env:USERPROFILE`, `New-Item`, `Invoke-WebRequest`)
> - On **macOS/Linux/Git Bash**, use bash syntax (`$HOME`, `mkdir -p`, `curl`)
> - **Detect the user's shell** before running commands
>
> **Download reliability:**
> - The `/releases/latest/download/` URL may 404 briefly after a new release while CI builds. If download fails, retry after 30 seconds.

---

## For Claude Code Users

> **AI agent: `/plugin` is a built-in slash command in Claude Code.** Run these commands directly.

```
/plugin marketplace add moeyui1/claude-status-pet
/plugin install claude-status-pet
```

Done! The plugin auto-downloads the binary on first session start.

> **⚠️ After first installation, restart your Claude Code session** for the plugin to take effect.

---

## For GitHub Copilot CLI Users

> **AI agent:** These commands run in the user's **terminal** (not inside a Copilot CLI session). They are standalone CLI commands.

```bash
copilot plugin marketplace add moeyui1/claude-status-pet
copilot plugin install claude-status-pet-copilot@claude-status-pet-copilot
```

This installs hooks and the `/pet` skill. Then **start a Copilot CLI session** and download the binary and assets:

```
/pet update
```

> **⚠️ After first installation, restart your Copilot CLI session** for the hooks and skill to take effect.

Done! Use `/pet on` to launch the pet.

---

## For VS Code Copilot Users

> **AI agent:** Use `Chat: Install Plugin From Source` from the VS Code command palette.

Run the VS Code command:
```
Chat: Install Plugin From Source
```
Enter: `https://github.com/moeyui1/claude-status-pet`

This installs hooks and the `/pet` skill. Then download the binary and assets:

```
/pet update
```

Done! Use `/pet on` to launch the pet.

> **⚠️ After first installation, restart your VS Code Copilot session** for the hooks and skill to take effect.

---

## Verification

After install, confirm the binary exists:

**bash:** `ls ~/.claude/pet-data/bin/claude-status-pet*`

**PowerShell:** `Get-Item "$env:USERPROFILE\.claude\pet-data\bin\claude-status-pet*"`

Tell the user: "Your desktop pet is installed! It will appear on your screen and react to what your AI assistant is doing. Right-click to change characters or settings."
