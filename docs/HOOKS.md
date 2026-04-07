# Hooks & Status Mapping

This document explains how hook events from different AI assistants are mapped to pet status states.

## Supported Assistants

| | Claude Code | Copilot CLI | VS Code Copilot |
|---|---|---|---|
| **Hook config** | `claude/hooks/hooks.json` | `copilot/hooks.json` | `vscode/hooks/hooks.json` |
| **Status writer** | `write-status --adapter claude` | `write-status --adapter copilot` | `write-status --adapter vscode` |
| **Platforms** | Claude Code CLI | Copilot CLI | VS Code (agent mode) |
| **Event naming** | PascalCase (`PreToolUse`) | camelCase (`preToolUse`) | PascalCase (`PreToolUse`) |
| **Tool names** | `Edit`, `Read`, `Bash`, `Grep` | `replace_string_in_file`, `read_file`, `run_in_terminal` | `replace_string_in_file`, `read_file`, `run_in_terminal` |
| **Tool input keys** | snake_case (`file_path`) | camelCase (`filePath`) | both (`file_path` and `filePath`) |
| **Session lifecycle** | `SessionStart` + `SessionEnd` | `sessionStart` + `sessionEnd` | `SessionStart` + `Stop` |
| **Idle trigger** | `Stop` | `stop` | `Stop` |
| **Error event** | `StopFailure` | `errorOccurred` | — (no dedicated error event) |
| **Sub-agents** | `SubagentStart` / `SubagentStop` | not available | `SubagentStart` / `SubagentStop` |
| **Permission prompt** | `Notification` | not available | not available |
| **Post-tool event** | not used | `postToolUse` → `thinking` | `PostToolUse` → `thinking` |
| **Context compact** | not available | not available | `PreCompact` → ignored |

## Hook Event → Status State Mapping

### Claude Code

| Hook Event | Status State | Detail | Notes |
|---|---|---|---|
| `UserPromptSubmit` | `thinking` | "Processing your prompt..." | User sent a message |
| `PreToolUse` — Read, WebFetch | `reading` | "Reading {filename}" | |
| `PreToolUse` — Edit, Write | `editing` | "Editing {filename}" | |
| `PreToolUse` — Grep, Glob, WebSearch | `searching` | "Searching: {pattern}" | |
| `PreToolUse` — Agent, Skill | `delegating` | "{description}" | |
| `PreToolUse` — Bash | `running` | "Running: {command}" | Truncated to 40 chars |
| `PreToolUse` — MCP tools (`mcp__*`) | `running` | "{server}: {tool}" | Auto-formatted |
| `PreToolUse` — other | `running` | "Using {toolName}" | Fallback |
| `SubagentStart` | `delegating` | "Spawning sub-agent..." | |
| `SubagentStop` | `thinking` | "Sub-agent finished" | |
| `Notification` | `waiting` | "Waiting for approval..." | Permission prompts |
| `Stop` | `idle` | "Waiting for input" | Response complete |
| `StopFailure` | `error` | "Something went wrong" | |
| `SessionEnd` | `offline` | "Session ended" | Writes offline, does NOT delete file |

### GitHub Copilot

Copilot CLI hooks — camelCase events via `--copilot-event` CLI arg.

| Hook Event | Status State | Detail | Notes |
|---|---|---|---|
| `sessionStart` | `idle` | "Session started" | Also auto-launches pet binary |
| `userPromptSubmitted` | `thinking` | "Processing prompt..." | |
| `preToolUse` — run_in_terminal | `running` | "Running: {command}" | |
| `preToolUse` — replace_string_in_file, edit_file | `editing` | "Editing {filename}" | |
| `preToolUse` — read_file | `reading` | "Reading {filename}" | |
| `preToolUse` — create_file, write_file | `editing` | "Writing {filename}" | |
| `preToolUse` — grep_search, semantic_search | `searching` | "Searching: {query}" | |
| `preToolUse` — file_search, glob | `searching` | "Finding: {pattern}" | |
| `preToolUse` — fetch_webpage | `reading` | "Fetching web page..." | |
| `preToolUse` — list_dir | `reading` | "Listing {path}" | |
| `preToolUse` — other | `running` | "Using {toolName}" | Fallback |
| `postToolUse` | `thinking` | "Processing..." | Avoids idle flash between tools |
| `stop` | `idle` | "Done" | Fires after each response |
| `errorOccurred` | `error` | "Error: {message}" | |
| `sessionEnd` | `offline` | "Session ended" | Writes offline, does NOT delete file |

### VS Code Copilot

VS Code agent mode hooks — PascalCase events from stdin `hookEventName`.

| Hook Event | Status State | Detail | Notes |
|---|---|---|---|
| `SessionStart` | `thinking` | "Session started" | New agent session |
| `UserPromptSubmit` | `thinking` | "Processing your prompt..." | User sent a message |
| `PreToolUse` — read_file, fetch_webpage | `reading` | "Reading {filename}" | |
| `PreToolUse` — replace_string_in_file, create_file | `editing` | "Editing {filename}" | |
| `PreToolUse` — grep_search, semantic_search, file_search | `searching` | "Searching: {query}" | |
| `PreToolUse` — run_in_terminal | `running` | "Running: {command}" | |
| `PreToolUse` — runSubagent | `delegating` | "Delegating..." | |
| `PreToolUse` — MCP tools (`mcp__*`) | varies | "{server}: {tool}" | Auto-formatted |
| `PreToolUse` — other | `running` | "Using {toolName}" | Fallback |
| `PostToolUse` | `thinking` | "Processing..." | Prevents idle flash between tools |
| `PreCompact` | — | — | Ignored (no status change) |
| `SubagentStart` | `delegating` | "Spawning sub-agent..." | |
| `SubagentStop` | `thinking` | "Sub-agent finished" | |
| `Stop` | `idle` | "Done" | Session/response complete |

## Pet Status States

All states the pet can display, with their visual behavior:

| State | Animation | Label Color | Ferris Sprite | Trigger |
|---|---|---|---|---|
| `idle` | gentle float | orange | 1 | Stop / Done |
| `thinking` | slow tilt | yellow | 3, 14 | Prompt submitted |
| `reading` | fast float | blue | 10 | Read, WebFetch |
| `editing` | wiggle | green | 19 | Edit, Write |
| `searching` | fast tilt | purple | 20 | Grep, Glob, WebSearch |
| `running` | fast wiggle | orange | 2 | Bash, terminal, MCP tools |
| `delegating` | side bounce | blue | 15 | Agent, Skill, SubagentStart |
| `waiting` | pulse | orange | 5 | Permission prompt |
| `error` | shake 3× | red | 9 | StopFailure, errorOccurred |
| `offline` | slow breathing | grey | 7 | SessionEnd |

Both Claude Code and GitHub Copilot now use the same fine-grained states for tool use.

## Installation

### VS Code Copilot

Install via plugin: run `Chat: Install Plugin From Source` with `https://github.com/moeyui1/claude-status-pet`. Hooks are loaded automatically from `vscode/plugin.json`.

For **manual install only** (if plugin install is not available), copy hook config to `.github/hooks/` (workspace-level) or `~/.copilot/hooks/` (global):

```bash
# Workspace-level (per project)
mkdir -p .github/hooks
cp vscode/hooks/hooks.json .github/hooks/status-pet-vscode.json

# User-level (all projects)
mkdir -p ~/.copilot/hooks
cp vscode/hooks/hooks.json ~/.copilot/hooks/status-pet-vscode.json
```

No scripts needed — the hooks call the binary directly (VS Code pipes stdin to the command).

## Adding a New AI Assistant

No scripts needed — just call the binary with CLI args from your hooks:

```bash
claude-status-pet write-status \
  --event tool \
  --tool edit \
  --detail "Editing main.rs" \
  --session-id my-session-123 \
  --session-name my-project
```

The binary handles everything: tool→state mapping, status file writing, and auto-launching the GUI.

For deeper integration (parsing native stdin JSON), create a built-in adapter. See [CONTRIBUTING.md](../CONTRIBUTING.md#adding-a-new-ai-agent-adapter) for details.
