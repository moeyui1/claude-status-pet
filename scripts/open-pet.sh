#!/bin/bash
# Manually launch a pet for all active sessions
set -e

PET_DIR="${CLAUDE_PLUGIN_DATA:-$HOME/.claude/pet-data}"
BIN_DIR="$PET_DIR/bin"
PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-$(dirname "$(dirname "$0")")}"

# Find binary: check bin dir first, then plugin build dir
OS=$(uname -s)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*|*_NT-*) EXT=".exe" ;;
  *) EXT="" ;;
esac

PET_BIN="$BIN_DIR/claude-status-pet$EXT"
if [ ! -f "$PET_BIN" ]; then
  PET_BIN="$PLUGIN_ROOT/pet-app/src-tauri/target/release/claude-status-pet$EXT"
fi

if [ ! -f "$PET_BIN" ]; then
  echo "Pet binary not found. Run '/pet' in Claude Code to auto-download, or build from source."
  exit 1
fi

# Clean up stale sessions (older than 24 hours)
find "$PET_DIR" -name "status-*.json" -mmin +1440 -delete 2>/dev/null || true

# GIFs (Mona/Kuromi) downloaded on-demand when user selects them
ASSETS_DIR="$PET_DIR/assets"

# Find all active session status files
count=0
for f in "$PET_DIR"/status-*.json; do
  [ -f "$f" ] || continue
  SESSION_ID=$(basename "$f" | sed 's/status-//;s/.json//')
  ASSETS_DIR="$PET_DIR/assets"
  if [ -d "$ASSETS_DIR/ferris" ] || [ -d "$ASSETS_DIR/mona" ]; then
    "$PET_BIN" --status-file "$f" --session-id "$SESSION_ID" --assets-dir "$ASSETS_DIR" &
  else
    "$PET_BIN" --status-file "$f" --session-id "$SESSION_ID" &
  fi
  count=$((count + 1))
done

if [ $count -eq 0 ]; then
  echo "No active sessions found"
else
  echo "Launched $count pet(s)"
fi
