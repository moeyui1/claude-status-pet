# Hooks & Status Mapping

This document explains how hook events from different AI assistants are mapped to pet status states.

## Supported Assistants

| | Claude Code | VS Code Copilot |
|---|---|---|
| **Hook config** | `hooks/hooks.json` | `copilot/hooks.json` |
| **Status writer** | `scripts/status-writer.sh` (bash + node) | `copilot/status-writer.js` (node) |
| **Event naming** | PascalCase (`PreToolUse`) | camelCase (`preToolUse`) |
| **Tool names** | `Edit`, `Read`, `Bash`, `Grep` | `replace_string_in_file`, `read_file`, `run_in_terminal`, `grep_search` |
| **Tool input keys** | snake_case (`file_path`) | camelCase (`filePath`) |
| **Session lifecycle** | `SessionStart` + `SessionEnd` | `sessionStart` + `sessionEnd` |
| **Idle trigger** | `Stop` (after full response) | `stop` (after each response) |
| **Error event** | `StopFailure` | `errorOccurred` |
| **Sub-agents** | `SubagentStart` / `SubagentStop` | not available |
| **Permission prompt** | `Notification` | not available |
| **Post-tool event** | not used (races with next PreToolUse) | `postToolUse` → mapped to `thinking` |

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

### VS Code Copilot

| Hook Event | Status State | Detail | Notes |
|---|---|---|---|
| `sessionStart` | `idle` | "Session started" | Also auto-launches pet binary |
| `userPromptSubmitted` | `thinking` | "Processing prompt..." | |
| `preToolUse` — run_in_terminal | `working` | "Running: {command}" | |
| `preToolUse` — replace_string_in_file, edit_file | `working` | "Editing {filename}" | |
| `preToolUse` — read_file | `working` | "Reading {filename}" | |
| `preToolUse` — create_file, write_file | `working` | "Writing {filename}" | |
| `preToolUse` — grep_search, semantic_search | `working` | "Searching: {query}" | |
| `preToolUse` — file_search, glob | `working` | "Finding: {pattern}" | |
| `preToolUse` — fetch_webpage | `working` | "Fetching web page..." | |
| `preToolUse` — list_dir | `working` | "Listing {path}" | |
| `preToolUse` — other | `working` | "Using {toolName}" | Fallback |
| `postToolUse` | `thinking` | "Processing..." | Avoids idle flash between tools |
| `stop` | `idle` | "Done" | Fires after each response |
| `errorOccurred` | `error` | "Error: {message}" | |
| `sessionEnd` | `offline` | "Session ended" | Writes offline, does NOT delete file |

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

> **Note:** Copilot uses a single `working` state for all tool use (maps to `running` animation). Claude Code has fine-grained states (`reading`, `editing`, `searching`, etc.) based on tool categories.

## Adding a New AI Assistant

The pet watches a JSON file — no code changes needed. Write a hook/adapter that outputs:

```json
{
  "state": "editing",
  "detail": "Editing main.rs",
  "tool": "edit",
  "event": "preToolUse",
  "session_id": "my-session-123",
  "session_name": "my-project",
  "timestamp": "2025-04-02T10:30:00Z"
}
```

to `~/.claude/pet-data/status-{session_id}.json`. See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.
