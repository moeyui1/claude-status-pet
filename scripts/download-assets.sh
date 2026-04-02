#!/bin/bash
# Download pet assets (character images) from GitHub Releases
# Checks version to skip re-download if already up to date
set -e

ASSETS_DIR="${1:-${CLAUDE_PLUGIN_DATA:-$HOME/.claude/pet-data}/assets}"
VERSION_FILE="$ASSETS_DIR/.version"
REPO="moeyui1/claude-status-pet"

mkdir -p "$ASSETS_DIR"

if ! command -v gh &>/dev/null; then
  echo "ERROR: gh CLI not found. Install it from https://cli.github.com/" >&2
  exit 1
fi

# Get latest release tag
LATEST=$(gh release view --repo "$REPO" --json tagName -q '.tagName' 2>&1)
if [ $? -ne 0 ] || [ -z "$LATEST" ]; then
  echo "ERROR: Failed to fetch latest release from $REPO. Is the repo public and gh authenticated?" >&2
  echo "  Detail: $LATEST" >&2
  exit 1
fi

# Check if already downloaded this version
if [ -f "$VERSION_FILE" ] && [ "$(cat "$VERSION_FILE")" = "$LATEST" ]; then
  exit 0
fi

# Download
TMPDIR=$(mktemp -d)
echo "Downloading pet assets $LATEST..."
if ! gh release download "$LATEST" --repo "$REPO" --pattern "pet-assets.zip" --dir "$TMPDIR" 2>&1; then
  echo "ERROR: Failed to download pet-assets.zip from $REPO release $LATEST" >&2
  rm -rf "$TMPDIR"
  exit 1
fi

ZIPFILE="$TMPDIR/pet-assets.zip"
if [ ! -f "$ZIPFILE" ]; then
  echo "ERROR: pet-assets.zip not found in release $LATEST" >&2
  rm -rf "$TMPDIR"
  exit 1
fi

# Extract
OS=$(uname -s)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*|*_NT-*)
    if ! powershell -Command "Expand-Archive -Path '$ZIPFILE' -DestinationPath '$ASSETS_DIR' -Force" 2>&1; then
      echo "ERROR: Failed to extract pet-assets.zip" >&2
      rm -rf "$TMPDIR"
      exit 1
    fi
    ;;
  *)
    if ! unzip -o "$ZIPFILE" -d "$ASSETS_DIR" 2>&1; then
      echo "ERROR: Failed to extract pet-assets.zip" >&2
      rm -rf "$TMPDIR"
      exit 1
    fi
    ;;
esac

rm -rf "$TMPDIR"
echo "$LATEST" > "$VERSION_FILE"
echo "Assets $LATEST installed to $ASSETS_DIR"
