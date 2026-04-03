# Custom Character Packs

> This document is designed to be read by an AI coding assistant (Claude Code, GitHub Copilot, Cursor, etc.) to create, install, export, and manage custom character packs for Claude Status Pet.

## What You're Building

A character pack is a folder with images (GIF/PNG/SVG) and a `character.json` config that maps pet states to images. Once created, the pet will show your character in the right-click menu under **Custom**.

## Output Directory

Install the pack to:
- **Windows**: `$env:USERPROFILE\.claude\pet-data\characters\<pack-name>\`
- **macOS/Linux**: `~/.claude/pet-data/characters/<pack-name>/`

## Step 1: Create the directory

```bash
PACK_NAME="my-character"  # kebab-case, no spaces
PACK_DIR="$HOME/.claude/pet-data/characters/$PACK_NAME"
mkdir -p "$PACK_DIR"
```

## Step 2: Add images

Each pet state needs at least one image. Multiple images per state adds variety (one is picked randomly).

**Image requirements:**
- Format: GIF (animated or static), PNG, or SVG
- Transparent background strongly recommended
- Square aspect ratio (~140×140px display area)
- Keep file sizes reasonable (<2MB per image)

**Required states and suggested themes:**

| State | When it shows | Suggested pose/mood |
|-------|--------------|---------------------|
| `idle` | Waiting for input | Relaxed, happy, waving |
| `thinking` | Processing a prompt | Curious, looking up, pondering |
| `reading` | Reading files | Focused, calm, studying |
| `editing` | Writing/editing files | Typing, busy, concentrated |
| `searching` | Searching code | Looking around, scanning |
| `running` | Running commands | Energetic, active, running |
| `delegating` | Spawning sub-agents | Pointing, directing, multitasking |
| `waiting` | Awaiting approval | Anxious, alert, patient |
| `error` | Something failed | Sad, frustrated, alarmed |
| `offline` | Session ended | Sleeping, faded, resting |

> **Minimum:** You need at least `idle`, `thinking`, `working` (covers editing/running/searching), and `offline`. Missing states fall back to `idle`.

## Step 3: Create character.json

Create `<pack-dir>/character.json`:

```json
{
  "name": "My Character",
  "type": "gif",
  "states": {
    "idle":       ["<pack-name>/idle.gif"],
    "thinking":   ["<pack-name>/think.gif"],
    "reading":    ["<pack-name>/read.gif"],
    "editing":    ["<pack-name>/edit.gif"],
    "searching":  ["<pack-name>/search.gif"],
    "running":    ["<pack-name>/run.gif"],
    "delegating": ["<pack-name>/delegate.gif"],
    "waiting":    ["<pack-name>/wait.gif"],
    "error":      ["<pack-name>/error.gif"],
    "offline":    ["<pack-name>/sleep.gif"],
    "unknown":    ["<pack-name>/idle.gif"]
  }
}
```

**Important:**
- `type` must be `"gif"`, `"png"`, or `"svg"`
- Image paths are relative to the `assets/` or `characters/` parent directory, prefixed with the pack name
- Each state value is an **array** of paths (for random variety)
- `name` is what appears in the right-click menu

## Step 4: Verify

After creating the pack, tell the user:

> "Character pack installed! Right-click the pet → Exit, then run `/pet on` to restart. Your character will appear under **Custom** in the right-click menu."

## Example: Creating from GIPHY

If the user wants a character from GIPHY or another source:

1. Search GIPHY for appropriate GIFs for each state
2. Download each GIF to the pack directory
3. Create `character.json` mapping states to filenames

```bash
# Example: download a GIF
curl -sLo "$PACK_DIR/idle.gif" "https://media.giphy.com/media/XXXXX/giphy.gif"
```

## Example: Creating from AI-generated images

If the user wants original art:

1. Generate images for each state using an image generation tool
2. Save them as PNG/GIF with transparent backgrounds
3. Resize to ~140×140px
4. Create `character.json` mapping states to filenames

## Example: Minimal pack (4 states)

For a quick pack with only essential states:

```json
{
  "name": "Simple Buddy",
  "type": "png",
  "states": {
    "idle":     ["simple-buddy/happy.png"],
    "thinking": ["simple-buddy/hmm.png"],
    "editing":  ["simple-buddy/typing.png"],
    "offline":  ["simple-buddy/sleep.png"]
  }
}
```

Missing states (reading, searching, running, etc.) will fall back to `idle`.

## Installing a Pack from URL or Local Path

### From URL (zip):

**PowerShell:**
```powershell
$charsDir = "$env:USERPROFILE\.claude\pet-data\characters"
New-Item -ItemType Directory -Path $charsDir -Force | Out-Null
$tmp = "$env:TEMP\pet-pack.zip"
Invoke-WebRequest -Uri "<URL>" -OutFile $tmp
Expand-Archive -Path $tmp -DestinationPath $charsDir -Force
Remove-Item $tmp
Write-Host "Pack installed. Restart pet to see it."
```

**bash:**
```bash
CHARS_DIR="$HOME/.claude/pet-data/characters"
mkdir -p "$CHARS_DIR"
curl -sLo /tmp/pet-pack.zip "<URL>"
unzip -o /tmp/pet-pack.zip -d "$CHARS_DIR"
rm -f /tmp/pet-pack.zip
echo "Pack installed. Restart pet to see it."
```

### From local path:

**PowerShell:**
```powershell
Copy-Item -Recurse "<LOCAL_PATH>" "$env:USERPROFILE\.claude\pet-data\characters\"
Write-Host "Pack installed. Restart pet to see it."
```

**bash:**
```bash
cp -r "<LOCAL_PATH>" "$HOME/.claude/pet-data/characters/"
echo "Pack installed. Restart pet to see it."
```

## Removing a Pack

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data\characters\<PACK_NAME>"
if (Test-Path $dir) { Remove-Item $dir -Recurse -Force; Write-Host "Removed: <PACK_NAME>" }
else { Write-Host "Pack not found: <PACK_NAME>" }
```

**bash:**
```bash
rm -rf "$HOME/.claude/pet-data/characters/<PACK_NAME>" && echo "Removed" || echo "Pack not found"
```

## Listing Installed Packs

**PowerShell:**
```powershell
$dir = "$env:USERPROFILE\.claude\pet-data"
foreach ($sub in @("assets","characters")) {
    $d = "$dir\$sub"
    if (-not (Test-Path $d)) { continue }
    Get-ChildItem $d -Directory | Where-Object { Test-Path "$($_.FullName)\character.json" } | ForEach-Object {
        $cfg = Get-Content "$($_.FullName)\character.json" | ConvertFrom-Json
        $label = if ($sub -eq "assets") { "DLC" } else { "Custom" }
        Write-Host "${label}: $($cfg.name) ($($_.Name))"
    }
}
```

Tell the user to restart the pet after installing or removing packs: "Right-click → Exit, then `/pet on`."

## Pack Structure

```
~/.claude/pet-data/characters/my-character/
├── character.json
├── idle.gif
├── think.gif
├── edit.gif
├── search.gif
├── run.gif
├── error.gif
└── sleep.gif
```

## Exporting a Pack

To share a character pack with others, zip the pack directory:

**PowerShell:**
```powershell
$packName = "<PACK_NAME>"
$packDir = "$env:USERPROFILE\.claude\pet-data\characters\$packName"
$outFile = "$env:USERPROFILE\Desktop\$packName.zip"
if (Test-Path $packDir) {
    Compress-Archive -Path "$packDir\*" -DestinationPath $outFile -Force
    Write-Host "Exported to: $outFile"
} else { Write-Host "Pack not found: $packName" }
```

**bash:**
```bash
PACK_NAME="<PACK_NAME>"
PACK_DIR="$HOME/.claude/pet-data/characters/$PACK_NAME"
OUT_FILE="$HOME/Desktop/$PACK_NAME.zip"
if [ -d "$PACK_DIR" ]; then
    cd "$PACK_DIR" && zip -r "$OUT_FILE" . && echo "Exported to: $OUT_FILE"
else echo "Pack not found: $PACK_NAME"; fi
```

Tell the user: "Pack exported! Share the zip file. Recipients can install it with the import instructions below."

## Importing a Pack

To install a pack from a zip file shared by someone else:

**PowerShell:**
```powershell
$zipFile = "<PATH_TO_ZIP>"
$packName = [System.IO.Path]::GetFileNameWithoutExtension($zipFile)
$destDir = "$env:USERPROFILE\.claude\pet-data\characters\$packName"
New-Item -ItemType Directory -Path $destDir -Force | Out-Null
Expand-Archive -Path $zipFile -DestinationPath $destDir -Force
Write-Host "Imported: $packName — restart pet to see it."
```

**bash:**
```bash
ZIP_FILE="<PATH_TO_ZIP>"
PACK_NAME=$(basename "$ZIP_FILE" .zip)
DEST_DIR="$HOME/.claude/pet-data/characters/$PACK_NAME"
mkdir -p "$DEST_DIR"
unzip -o "$ZIP_FILE" -d "$DEST_DIR"
echo "Imported: $PACK_NAME — restart pet to see it."
```

Tell the user: "Pack imported! Right-click the pet → Exit, then `/pet on` to see it under Custom."
