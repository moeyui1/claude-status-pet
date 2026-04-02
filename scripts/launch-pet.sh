#!/bin/bash
# Launch a pet window for this session (if auto-start is enabled)
# Auto-downloads the binary on first run from GitHub Releases
set -e

PET_DIR="${CLAUDE_PLUGIN_DATA:-$HOME/.claude/pet-data}"
CONFIG_FILE="$PET_DIR/config.json"
BIN_DIR="$PET_DIR/bin"
PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-$(dirname "$(dirname "$0")")}"
REPO="moeyui1/claude-status-pet"
mkdir -p "$PET_DIR" "$BIN_DIR"

INPUT=$(cat)

# Parse session info using node
SESSION_ID=$(node -e "console.log((JSON.parse(process.argv[1]).session_id)||'unknown')" "$INPUT")
CWD=$(node -e "console.log((JSON.parse(process.argv[1]).cwd)||'')" "$INPUT")
SESSION_NAME=$(basename "$CWD")

# Check if auto-start is enabled (default: true)
AUTO_START="true"
if [ -f "$CONFIG_FILE" ]; then
  AUTO_START=$(node -e "try{console.log(JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')).auto_start!==false)}catch(e){console.log('true')}" "$CONFIG_FILE")
fi

if [ "$AUTO_START" != "true" ]; then
  exit 0
fi

# Determine platform and binary name
OS=$(uname -s)
ARCH=$(uname -m)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*|*_NT-*)
    ASSET="claude-status-pet-windows-x64.exe"
    PET_BIN="$BIN_DIR/claude-status-pet.exe"
    ;;
  Darwin)
    if [ "$ARCH" = "arm64" ]; then
      ASSET="claude-status-pet-macos-arm64"
    else
      ASSET="claude-status-pet-macos-x64"
    fi
    PET_BIN="$BIN_DIR/claude-status-pet"
    ;;
  Linux)
    ASSET="claude-status-pet-linux-x64"
    PET_BIN="$BIN_DIR/claude-status-pet"
    ;;
esac

# Auto-download binary if not present
if [ ! -f "$PET_BIN" ]; then
  if ! command -v curl &>/dev/null; then
    echo "ERROR: curl not found." >&2
    exit 1
  fi

  echo "Downloading pet binary ($ASSET)..."
  if ! curl -sLo "$BIN_DIR/$ASSET" "https://github.com/$REPO/releases/latest/download/$ASSET"; then
    echo "ERROR: Failed to download $ASSET from $REPO." >&2
    echo "  Check: https://github.com/$REPO/releases" >&2
    exit 1
  fi

  if [ -f "$BIN_DIR/$ASSET" ]; then
    mv "$BIN_DIR/$ASSET" "$PET_BIN"
    chmod +x "$PET_BIN" 2>/dev/null || true
  else
    echo "ERROR: Binary $ASSET not found after download." >&2
    exit 1
  fi
fi

# Fallback: try binary relative to plugin root (for build-from-source users)
if [ ! -f "$PET_BIN" ]; then
  case "$OS" in
    MINGW*|MSYS*|CYGWIN*|*_NT-*)
      ALT_BIN="$PLUGIN_ROOT/pet-app/src-tauri/target/release/claude-status-pet.exe" ;;
    *)
      ALT_BIN="$PLUGIN_ROOT/pet-app/src-tauri/target/release/claude-status-pet" ;;
  esac
  if [ -f "$ALT_BIN" ]; then
    PET_BIN="$ALT_BIN"
  else
    echo "ERROR: Pet binary not found. Build from source or ensure GitHub Releases are accessible." >&2
    exit 1
  fi
fi

# Download/update Ferris SVGs (CC0, always available)
SCRIPTS_DIR="$PLUGIN_ROOT/scripts"
ASSETS_DIR="$PET_DIR/assets"
bash "$SCRIPTS_DIR/download-assets.sh" "$ASSETS_DIR" &
# GIFs (Mona/Kuromi) downloaded on-demand when user selects them

# Write initial status
STATUS_FILE="$PET_DIR/status-${SESSION_ID}.json"
node -e "require('fs').writeFileSync(process.argv[1],JSON.stringify({state:'idle',detail:'Session started',tool:'',event:'SessionStart',session_id:process.argv[2],session_name:process.argv[3],timestamp:new Date().toISOString()}))" "$STATUS_FILE" "$SESSION_ID" "$SESSION_NAME"

# Determine assets dir: downloaded assets > plugin bundled > none
if [ -d "$ASSETS_DIR/ferris" ] || [ -d "$ASSETS_DIR/mona" ]; then
  RESOLVED_ASSETS="$ASSETS_DIR"
elif [ -d "$PLUGIN_ROOT/pet-app/src/ferris" ]; then
  RESOLVED_ASSETS="$PLUGIN_ROOT/pet-app/src"
fi

# Launch pet
if [ -n "$RESOLVED_ASSETS" ]; then
  "$PET_BIN" --status-file "$STATUS_FILE" --session-id "$SESSION_ID" --assets-dir "$RESOLVED_ASSETS" &
else
  echo "WARNING: No asset images found. Pet will show broken images." >&2
  echo "  Expected at: $ASSETS_DIR or $PLUGIN_ROOT/pet-app/src/" >&2
  "$PET_BIN" --status-file "$STATUS_FILE" --session-id "$SESSION_ID" &
fi
