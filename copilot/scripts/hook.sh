#!/usr/bin/env bash
# Copilot CLI hook handler for claude-status-pet
# Called by hooks.json with the event name as the first argument.
set -euo pipefail

EVENT="${1:-}"
[ -z "$EVENT" ] && exit 1

PET_BIN=$(ls "$HOME/.claude/pet-data/bin/claude-status-pet"* 2>/dev/null | head -1)
[ -z "$PET_BIN" ] && exit 0

"$PET_BIN" write-status --adapter copilot --copilot-event "$EVENT"
