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
if command -v gh &>/dev/null; then
  gh api "repos/$REPO/contents/copilot/status-writer.sh" \
    -H "Accept: application/vnd.github.raw+json" > "$SCRIPTS_DIR/copilot-status-writer.sh"
elif command -v curl &>/dev/null; then
  curl -sL "https://raw.githubusercontent.com/$REPO/main/copilot/status-writer.sh" \
    -o "$SCRIPTS_DIR/copilot-status-writer.sh"
else
  echo "ERROR: Need gh or curl to download files" >&2
  exit 1
fi
chmod +x "$SCRIPTS_DIR/copilot-status-writer.sh"

# 2. Add hooks to current repo
echo "Adding hooks to $HOOKS_DIR..."
mkdir -p "$HOOKS_DIR"
if command -v gh &>/dev/null; then
  gh api "repos/$REPO/contents/copilot/hooks.json" \
    -H "Accept: application/vnd.github.raw+json" > "$HOOKS_DIR/status-pet.json"
elif command -v curl &>/dev/null; then
  curl -sL "https://raw.githubusercontent.com/$REPO/main/copilot/hooks.json" \
    -o "$HOOKS_DIR/status-pet.json"
fi

echo ""
echo "Done! Next steps:"
echo "  1. Commit: git add .github/hooks/status-pet.json && git commit -m 'Add status pet hooks'"
echo "  2. If you don't have the pet binary yet, install the Claude Code plugin:"
echo "     /plugin marketplace add moeyui1/claude-status-pet"
echo "     /plugin install claude-status-pet"
echo "  3. Or download from: https://github.com/$REPO/releases"
echo ""
