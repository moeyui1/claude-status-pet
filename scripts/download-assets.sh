#!/bin/bash
# Download pet assets (character images) from GitHub Releases
# Checks version to skip re-download if already up to date
# Uses curl + node (no gh CLI required)
set -e

ASSETS_DIR="${1:-${CLAUDE_PLUGIN_DATA:-$HOME/.claude/pet-data}/assets}"
VERSION_FILE="$ASSETS_DIR/.version"
REPO="moeyui1/claude-status-pet"

mkdir -p "$ASSETS_DIR"

if ! command -v curl &>/dev/null; then
  echo "ERROR: curl not found." >&2
  exit 1
fi

if ! command -v node &>/dev/null; then
  echo "ERROR: node not found. Install Node.js to continue." >&2
  exit 1
fi

# Get latest release tag via GitHub API + node for JSON parsing
LATEST=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | node -e "
  let d='';
  process.stdin.on('data',c=>d+=c);
  process.stdin.on('end',()=>{
    try { process.stdout.write(JSON.parse(d).tag_name || ''); }
    catch(e) { process.exit(1); }
  });
")
if [ -z "$LATEST" ]; then
  echo "ERROR: Failed to fetch latest release from $REPO." >&2
  exit 1
fi

# Check if already downloaded this version
if [ -f "$VERSION_FILE" ] && [ "$(cat "$VERSION_FILE")" = "$LATEST" ]; then
  exit 0
fi

# Download
TMPDIR=$(mktemp -d)
echo "Downloading pet assets $LATEST..."
if ! curl -sLo "$TMPDIR/pet-assets.zip" "https://github.com/$REPO/releases/download/$LATEST/pet-assets.zip"; then
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
