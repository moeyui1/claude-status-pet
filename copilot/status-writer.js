#!/usr/bin/env node
// GitHub Copilot hook script — pure Node.js version (no bash required)
// Writes status to per-session status file + auto-launches pet on sessionStart

const path = require('path');
const fs = require('fs');
const os = require('os');
const crypto = require('crypto');

const STATUS_DIR = path.join(os.homedir(), '.claude', 'pet-data');
fs.mkdirSync(STATUS_DIR, { recursive: true });

const hookEvent = process.env.COPILOT_HOOK_EVENT || 'unknown';

// Read JSON from stdin
let inputData = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { inputData += chunk; });
process.stdin.on('end', () => {
  let input = {};
  try { input = JSON.parse(inputData); } catch(e) {}
  run(input);
});
// If stdin closes immediately (no data), still run
setTimeout(() => { if (!inputData) run({}); }, 100);

function run(input) {
  const cwd = input.cwd || '';
  const sessionName = path.basename(cwd) || 'copilot';
  const sessionId = 'copilot-' + crypto.createHash('md5').update(cwd || 'default').digest('hex').slice(0, 8);
  const statusFile = path.join(STATUS_DIR, 'status-' + sessionId + '.json');
  const toolName = input.toolName || '';
  let toolArgs = {};
  try { toolArgs = JSON.parse(input.toolArgs || '{}'); } catch(e) {}

  if (hookEvent === 'sessionEnd') {
    // Don't delete — write offline state so pet shows sleeping instead of closing
    const status = { state: 'offline', detail: 'Session ended', tool: '', event: hookEvent, session_id: sessionId, session_name: path.basename(cwd || sessionId), timestamp: new Date().toISOString() };
    fs.writeFileSync(statusFile, JSON.stringify(status));
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
        case 'run_in_terminal':
          detail = 'Running: ' + (toolArgs.command || '').slice(0, 40); break;
        case 'edit':
        case 'edit_file':
        case 'replace_string_in_file':
          detail = 'Editing ' + path.basename(toolArgs.file || toolArgs.filePath || toolArgs.path || ''); break;
        case 'view':
        case 'read_file':
          detail = 'Reading ' + path.basename(toolArgs.file || toolArgs.filePath || toolArgs.path || ''); break;
        case 'write':
        case 'write_file':
        case 'create_file':
          detail = 'Writing ' + path.basename(toolArgs.file || toolArgs.filePath || toolArgs.path || ''); break;
        case 'search':
        case 'grep':
        case 'grep_search':
        case 'semantic_search':
          detail = 'Searching: ' + (toolArgs.pattern || toolArgs.query || '').slice(0, 30); break;
        case 'glob':
        case 'find':
        case 'file_search':
          detail = 'Finding: ' + (toolArgs.pattern || toolArgs.glob || toolArgs.query || '').slice(0, 30); break;
        case 'fetch_webpage':
          detail = 'Fetching web page...'; break;
        case 'list_dir':
          detail = 'Listing ' + path.basename(toolArgs.path || ''); break;
        default:
          detail = 'Using ' + toolName;
      }
      break;
    case 'postToolUse':
      state = 'thinking';
      detail = 'Processing...';
      break;
    case 'stop':
      state = 'idle';
      detail = 'Done';
      break;
    case 'errorOccurred':
      state = 'error';
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
    const { execSync, spawn } = require('child_process');
    const isWin = os.platform() === 'win32';
    const binName = isWin ? 'claude-status-pet-windows-x64.exe' : 'claude-status-pet';
    const binAlt = isWin ? 'claude-status-pet.exe' : 'claude-status-pet';
    const binDir = path.join(STATUS_DIR, 'bin');
    const assetsDir = path.join(STATUS_DIR, 'assets');

    let petBin = null;
    for (const name of [binAlt, binName]) {
      const p = path.join(binDir, name);
      if (fs.existsSync(p)) { petBin = p; break; }
    }

    if (petBin) {
      let alreadyRunning = false;
      try {
        if (isWin) {
          // tasklist truncates long image names, so match on short prefix
          const psOut = execSync('tasklist /NH', { encoding: 'utf8', timeout: 3000 });
          alreadyRunning = psOut.includes('claude-status-pet');
        } else {
          execSync('pgrep -f claude-status-pet', { timeout: 3000 });
          alreadyRunning = true;
        }
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
}
