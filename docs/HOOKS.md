# Hooks & Status Mapping

This document explains how hook events from different AI assistants are mapped to pet status states.

## Supported Assistants

| | Claude Code | Copilot CLI | VS Code Copilot |
|---|---|---|---|
| **Hook config** | `claude/hooks/hooks.json` | `copilot/hooks.json` | `vscode/hooks/hooks.json` |
| **Status writer** | `write-status --adapter claude` | `write-status --adapter copilot` | `write-status --adapter vscode` |
| **Platforms** | Claude Code CLI | Copilot CLI | VS Code (agent mode) |
| **Event naming** | PascalCase (`PreToolUse`) | camelCase (`preToolUse`) | PascalCase (`PreToolUse`) |
| **Tool names** | `Edit`, `Read`, `Bash`, `Grep` | `bash`, `edit`, `view`, `grep`, `glob`, `create`, `web_fetch`, `task`, `powershell`, `ask_user` | `replace_string_in_file`, `read_file`, `run_in_terminal` |
| **Tool input keys** | snake_case (`file_path`) | camelCase (`filePath`) — or snake_case `tool_input` for PascalCase events | both (`file_path` and `filePath`) |
| **Session lifecycle** | `SessionStart` + `SessionEnd` | `sessionStart` + `sessionEnd` | `SessionStart` + `Stop` |
| **Idle trigger** | `Stop` | `agentStop` | `Stop` |
| **Error event** | `StopFailure` | `errorOccurred` + `postToolUseFailure` | — (no dedicated error event) |
| **Sub-agents** | `SubagentStart` / `SubagentStop` | `subagentStart` / `subagentStop` | `SubagentStart` / `SubagentStop` |
| **Permission prompt** | `Notification` | `notification` (`permission_prompt`) + `permissionRequest` | not available |
| **Post-tool event** | not used | `postToolUse` → `thinking` | `PostToolUse` → `thinking` |
| **Context compact** | not available | `preCompact` → ignored | `PreCompact` → ignored |

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
| `SessionEnd` | `closed` | "Session ended" | Writes closed status, does NOT delete file |

### GitHub Copilot

Copilot CLI hooks — camelCase events via `--copilot-event` CLI arg. Tool names follow the
[official reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-hooks-reference):
`bash`, `edit`, `view`, `grep`, `glob`, `create`, `web_fetch`, `task`, `powershell`, `ask_user`.

| Hook Event | Status State | Detail | Notes |
|---|---|---|---|
| `sessionStart` | `thinking` | "Processing prompt..." | New or resumed session |
| `userPromptSubmitted` | `thinking` | "Processing your prompt..." | |
| `preToolUse` — `bash` / `powershell` | `running` | "Running: {command}" | |
| `preToolUse` — `edit` / `create` | `editing` | "Editing {filename}" | |
| `preToolUse` — `view` | `reading` | "Reading {filename}" | |
| `preToolUse` — `grep` | `searching` | "Searching: {query}" | |
| `preToolUse` — `glob` | `searching` | "Finding: {pattern}" | |
| `preToolUse` — `web_fetch` | `reading` | "Fetching web page..." | |
| `preToolUse` — `task` | `delegating` | "Using task" | Subagent task |
| `preToolUse` — other | `running` | "Using {toolName}" | Fallback |
| `postToolUse` | `thinking` | "Processing..." | Avoids idle flash between tools |
| `postToolUseFailure` | `error` | "Error: {message}" | Tool execution failed |
| `agentStop` | `idle` | "Done" | Main agent finishes a turn |
| `subagentStart` | `delegating` | "Spawning {agentName}..." | |
| `subagentStop` | `thinking` | "Sub-agent finished" | |
| `notification` — `permission_prompt` | `waiting` | "Waiting for approval..." | |
| `notification` — `elicitation_dialog` | `waiting` | "Waiting for input..." | |
| `notification` — other types | — | — | Ignored (shell/agent completion) |
| `permissionRequest` | `waiting` | "Waiting for approval..." | Fires before permission service |
| `preCompact` | — | — | Ignored (no status change) |
| `errorOccurred` | `error` | "Error: {message}" | |
| `sessionEnd` | `closed` | "Session ended" | Writes closed status, does NOT delete file |

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
| `offline` | slow breathing | grey | 7 | timeout/unknown sessionEnd |
| `closed` | — | grey | 7 | SessionEnd (Claude) |

Both Claude Code and GitHub Copilot now use the same fine-grained states for tool use.

## Installation

### VS Code Copilot

Install via plugin: run `Chat: Install Plugin From Source` with `https://github.com/moeyui1/claude-status-pet`. Hooks are loaded automatically from `vscode/plugin.json`.

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
