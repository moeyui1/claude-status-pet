#!/bin/bash
# GitHub Copilot CLI hook script — writes status to per-session status file
# Reads Copilot hook JSON from stdin
set -e

STATUS_DIR="$HOME/.claude/pet-data"
mkdir -p "$STATUS_DIR"

INPUT=$(cat)
HOOK_EVENT="${COPILOT_HOOK_EVENT:-unknown}"

node -e "
const input = JSON.parse(process.argv[1]);
const hookEvent = process.argv[2];
const path = require('path');
const fs = require('fs');

const cwd = input.cwd || '';
const sessionName = path.basename(cwd) || 'copilot';
// Use cwd hash as session ID since Copilot doesn't provide one
const crypto = require('crypto');
const sessionId = 'copilot-' + crypto.createHash('md5').update(cwd || 'default').digest('hex').slice(0, 8);
const statusDir = process.argv[3];
const statusFile = path.join(statusDir, 'status-' + sessionId + '.json');
const toolName = input.toolName || '';
const toolArgs = (() => { try { return JSON.parse(input.toolArgs || '{}'); } catch(e) { return {}; } })();

if (hookEvent === 'sessionEnd') {
  try { fs.unlinkSync(statusFile); } catch(e) {}
  process.exit(0);
}

let state = 'unknown';
let detail = hookEvent;

switch (hookEvent) {
  case 'sessionStart':
    state = 'idle';
    detail = 'Session started';
    break;
  case 'userPromptSubmitted':
    state = 'thinking';
    detail = 'Processing prompt...';
    break;
  case 'preToolUse':
    state = 'working';
    switch (toolName) {
      case 'bash':
        detail = 'Running: ' + (toolArgs.command || '').slice(0, 40); break;
      case 'edit':
      case 'edit_file':
        detail = 'Editing ' + path.basename(toolArgs.file || toolArgs.path || ''); break;
      case 'view':
      case 'read_file':
        detail = 'Reading ' + path.basename(toolArgs.file || toolArgs.path || ''); break;
      case 'write':
      case 'write_file':
      case 'create_file':
        detail = 'Writing ' + path.basename(toolArgs.file || toolArgs.path || ''); break;
      case 'search':
      case 'grep':
        detail = 'Searching: ' + (toolArgs.pattern || toolArgs.query || ''); break;
      case 'glob':
      case 'find':
        detail = 'Finding: ' + (toolArgs.pattern || toolArgs.glob || ''); break;
      default:
        detail = 'Using ' + toolName;
    }
    break;
  case 'postToolUse':
    state = 'working';
    detail = 'Done with ' + toolName;
    break;
  case 'errorOccurred':
    state = 'working';
    const errMsg = (input.error && input.error.message) || 'Unknown error';
    detail = 'Error: ' + errMsg.slice(0, 40);
    break;
  default:
    state = 'idle';
    detail = 'Waiting for input';
}

const status = {
  state, detail, tool: toolName, event: hookEvent,
  session_id: sessionId, session_name: sessionName + ' (Copilot)',
  timestamp: new Date().toISOString()
};

fs.writeFileSync(statusFile, JSON.stringify(status));

// On sessionStart, auto-launch the pet binary if not already running
if (hookEvent === 'sessionStart') {
  const os = require('os');
  const { execSync, spawn } = require('child_process');
  const isWin = os.platform() === 'win32';
  const binName = isWin ? 'claude-status-pet-windows-x64.exe' : 'claude-status-pet';
  const binAlt = isWin ? 'claude-status-pet.exe' : 'claude-status-pet';
  const binDir = path.join(statusDir, 'bin');
  const assetsDir = path.join(statusDir, 'assets');

  // Find the binary
  let petBin = null;
  for (const name of [binAlt, binName]) {
    const p = path.join(binDir, name);
    if (fs.existsSync(p)) { petBin = p; break; }
  }

  if (petBin) {
    // Check if already running for this session
    let alreadyRunning = false;
    try {
      const psOut = isWin
        ? execSync('tasklist /FI \"IMAGENAME eq ' + path.basename(petBin) + '\" /NH', { encoding: 'utf8', timeout: 3000 })
        : execSync('pgrep -f claude-status-pet', { encoding: 'utf8', timeout: 3000 });
      alreadyRunning = psOut.includes(path.basename(petBin)) || psOut.trim().length > 0;
    } catch(e) {}

    if (!alreadyRunning) {
      const args = ['--status-file', statusFile, '--session-id', sessionId];
      if (fs.existsSync(assetsDir)) {
        args.push('--assets-dir', assetsDir);
      }
      const child = spawn(petBin, args, { detached: true, stdio: 'ignore' });
      child.unref();
    }
  }
}
" "$INPUT" "$HOOK_EVENT" "$STATUS_DIR"
