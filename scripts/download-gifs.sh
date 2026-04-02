#!/bin/bash
# Download character GIFs from GIPHY at runtime
# These are NOT bundled in the repo due to licensing
# Non-fatal: logs errors but doesn't block the pet from launching

ASSETS_DIR="${1:-${CLAUDE_PLUGIN_DATA:-$HOME/.claude/pet-data}/assets}"
VERSION_FILE="$ASSETS_DIR/.gifs-version"
CURRENT_VERSION="2"

mkdir -p "$ASSETS_DIR/mona" "$ASSETS_DIR/kuromi"

# Skip if already downloaded this version
if [ -f "$VERSION_FILE" ] && [ "$(cat "$VERSION_FILE")" = "$CURRENT_VERSION" ]; then
  exit 0
fi

echo "Downloading character GIFs from GIPHY..."

# Use curl (widely available)
if ! command -v curl &>/dev/null; then
  echo "WARNING: curl not found, skipping GIF download" >&2
  exit 0
fi
DL="curl -sL --fail -o"

# Mona (GitHub mascot) — from official GitHub GIPHY channel
$DL "$ASSETS_DIR/mona/love.gif"      "https://media.giphy.com/media/jrdgDVFrcgJpNlonWO/giphy.gif" &
$DL "$ASSETS_DIR/mona/angry.gif"     "https://media.giphy.com/media/kmCCrDo2vlIu6Kswop/giphy.gif" &
$DL "$ASSETS_DIR/mona/looking.gif"   "https://media.giphy.com/media/9f8mk4P3X2Nvch1z2o/giphy.gif" &
$DL "$ASSETS_DIR/mona/mona.gif"      "https://media.giphy.com/media/OFEabGCcVqsckIGn8G/giphy.gif" &
$DL "$ASSETS_DIR/mona/tongue.gif"    "https://media.giphy.com/media/WcYnTzdrjQphdu33xs/giphy.gif" &
$DL "$ASSETS_DIR/mona/shocked.gif"   "https://media.giphy.com/media/JdQFsdoJBcHaPOANdK/giphy.gif" &
$DL "$ASSETS_DIR/mona/smirk.gif"     "https://media.giphy.com/media/0vTOscboHgOyBSuK4r/giphy.gif" &
$DL "$ASSETS_DIR/mona/laugh.gif"     "https://media.giphy.com/media/RgutegYIHk2Nhxj4m5/giphy.gif" &
$DL "$ASSETS_DIR/mona/ohbrother.gif" "https://media.giphy.com/media/pMzEfC42AYlqT2WPaf/giphy.gif" &
$DL "$ASSETS_DIR/mona/hearts.gif"    "https://media.giphy.com/media/wJBYx2Yh84XS4sTzmz/giphy.gif" &
$DL "$ASSETS_DIR/mona/sick.gif"      "https://media.giphy.com/media/nfL2nlWacI8d9jgVXb/giphy.gif" &
$DL "$ASSETS_DIR/mona/tech.gif"      "https://media.giphy.com/media/cDZJ17fbzWVle68VCB/giphy.gif" &
$DL "$ASSETS_DIR/mona/ducks.gif"     "https://media.giphy.com/media/QxT6pLq6ekKiCkLkf0/giphy.gif" &

# Kuromi (Sanrio) — from official Sanrio Korea GIPHY channel
$DL "$ASSETS_DIR/kuromi/bling.gif"    "https://media.giphy.com/media/JNxq0xOWfidCDzqUH3/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/charming.gif" "https://media.giphy.com/media/gkLG3Ki3OTXDwVb4rY/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/kuromi.gif"   "https://media.giphy.com/media/MphoCSnXeA6wR4L8IS/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/lilrya.gif"   "https://media.giphy.com/media/4G0nkrrXm8Xe1VgPzF/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/jump.gif"     "https://media.giphy.com/media/cQSjIBgUC2NbMKEm9q/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/sleeping.gif" "https://media.giphy.com/media/ZIskbLAG8Qeiq2dbV5/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/heart.gif"    "https://media.giphy.com/media/VpCEcS3ZJ4qlKcD8LF/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/think.gif"    "https://media.giphy.com/media/dCRVRbdbZUlNt1sRPd/giphy.gif" &
$DL "$ASSETS_DIR/kuromi/angry.gif"    "https://media.giphy.com/media/Qtvvgwbl1svKYcUGIT/giphy.gif" &

wait

# Check how many succeeded
FAIL=0
for f in "$ASSETS_DIR"/mona/*.gif "$ASSETS_DIR"/kuromi/*.gif; do
  if [ ! -s "$f" ]; then
    echo "WARNING: Failed to download $(basename "$f")" >&2
    rm -f "$f"
    FAIL=$((FAIL + 1))
  fi
done

if [ $FAIL -gt 0 ]; then
  echo "WARNING: $FAIL GIF(s) failed to download. Mona/Kuromi may show broken images." >&2
else
  echo "$CURRENT_VERSION" > "$VERSION_FILE"
  echo "GIFs downloaded to $ASSETS_DIR"
fi
