#!/bin/bash
# Reads hook event JSON from stdin and writes status to per-session file
# Uses node instead of jq for portability
set -e

export STATUS_DIR="${CLAUDE_PLUGIN_DATA:-$HOME/.claude/pet-data}"
mkdir -p "$STATUS_DIR"

node -e "
const input = JSON.parse(require('fs').readFileSync(0, 'utf8'));
const path = require('path');
const fs = require('fs');

const event = input.hook_event_name || 'unknown';
const sessionId = input.session_id || 'unknown';
const toolName = input.tool_name || '';
const toolInput = input.tool_input || {};
const cwd = input.cwd || '';
const sessionName = path.basename(cwd) || sessionId;
const statusDir = process.env.STATUS_DIR;
const statusFile = path.join(statusDir, 'status-' + sessionId + '.json');

if (event === 'SessionEnd') {
  try { fs.unlinkSync(statusFile); } catch(e) {}
  process.exit(0);
}

let state = 'unknown';
let detail = event;

switch (event) {
  case 'UserPromptSubmit':
    state = 'thinking';
    detail = 'Processing your prompt...';
    break;
  case 'PreToolUse': {
    // Categorize tools by nature so future tools auto-map
    const readTools = ['Read', 'WebFetch', 'ReadMcpResourceTool', 'ListMcpResourcesTool'];
    const editTools = ['Edit', 'Write', 'NotebookEdit'];
    const searchTools = ['Grep', 'Glob', 'WebSearch'];
    const delegateTools = ['Agent', 'Skill'];
    const runTools = ['Bash'];

    if (readTools.includes(toolName)) {
      state = 'reading';
      detail = 'Reading ' + path.basename(toolInput.file_path || toolInput.url || toolName);
    } else if (editTools.includes(toolName)) {
      state = 'editing';
      detail = 'Editing ' + path.basename(toolInput.file_path || '');
    } else if (searchTools.includes(toolName)) {
      state = 'searching';
      detail = toolInput.pattern ? 'Searching: ' + toolInput.pattern
             : toolInput.query ? 'Searching: ' + toolInput.query
             : 'Finding: ' + toolName;
    } else if (delegateTools.includes(toolName)) {
      state = 'delegating';
      detail = toolInput.description || toolInput.skill || 'Delegating...';
    } else if (runTools.includes(toolName)) {
      state = 'running';
      detail = 'Running: ' + (toolInput.command || '').slice(0, 40);
    } else if (toolName.startsWith('mcp__')) {
      // MCP tools — treat as running with a cleaner name
      const parts = toolName.split('__');
      state = 'running';
      detail = (parts[1] || 'MCP') + ': ' + (parts[2] || '').replace(/_/g, ' ');
    } else {
      state = 'running';
      detail = 'Using ' + toolName;
    }
    break;
  }
  case 'SubagentStart':
    state = 'delegating';
    detail = 'Spawning sub-agent...';
    break;
  case 'SubagentStop':
    state = 'thinking';
    detail = 'Sub-agent finished';
    break;
  case 'Notification':
    state = 'waiting';
    detail = 'Waiting for approval...';
    break;
  case 'StopFailure':
    state = 'error';
    detail = 'Something went wrong';
    break;
  case 'Stop':
    state = 'idle';
    detail = 'Waiting for input';
    break;
  default:
    state = 'unknown';
}

const status = {
  state, detail, tool: toolName, event,
  session_id: sessionId, session_name: sessionName,
  timestamp: new Date().toISOString()
};

fs.writeFileSync(statusFile, JSON.stringify(status));
"
