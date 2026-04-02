#!/bin/bash
# One-command setup for GitHub Copilot CLI integration
# Usage: bash <(curl -sL https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/setup.sh)
set -e

REPO="moeyui1/claude-status-pet"
SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
HOOKS_DIR=".github/hooks"

echo "=== Claude Status Pet — Copilot Setup ==="

# 1. Install the status writer script
echo "Installing status writer..."
mkdir -p "$SCRIPTS_DIR"
curl -sLo "$SCRIPTS_DIR/copilot-status-writer.sh" \
  "https://raw.githubusercontent.com/$REPO/main/copilot/status-writer.sh"
chmod +x "$SCRIPTS_DIR/copilot-status-writer.sh"

# 2. Add hooks to current repo
echo "Adding hooks to $HOOKS_DIR..."
mkdir -p "$HOOKS_DIR"
curl -sLo "$HOOKS_DIR/status-pet.json" \
  "https://raw.githubusercontent.com/$REPO/main/copilot/hooks.json"

echo ""
echo "Done! Next steps:"
echo "  1. Commit: git add .github/hooks/status-pet.json && git commit -m 'Add status pet hooks'"
echo "  2. If you don't have the pet binary yet, install the Claude Code plugin:"
echo "     /plugin marketplace add moeyui1/claude-status-pet"
echo "     /plugin install claude-status-pet"
echo "  3. Or download from: https://github.com/$REPO/releases"
echo ""
