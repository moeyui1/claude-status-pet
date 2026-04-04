# Manual Installation Steps

> Referenced by [INSTALL.md](INSTALL.md). These are the individual steps for downloading and configuring the pet binary, hooks, and assets.

---

## Manual: Download Binary

**macOS / Linux / Git Bash:**

```bash
INSTALL_DIR="$HOME/.claude/pet-data/bin"
mkdir -p "$INSTALL_DIR"

OS=$(uname -s); ARCH=$(uname -m)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*|*_NT-*) ASSET="claude-status-pet-windows-x64.exe" ;;
  Darwin) [ "$ARCH" = "arm64" ] && ASSET="claude-status-pet-macos-arm64" || ASSET="claude-status-pet-macos-x64" ;;
  Linux) ASSET="claude-status-pet-linux-x64" ;;
esac

curl -sLo "$INSTALL_DIR/$ASSET" "https://github.com/moeyui1/claude-status-pet/releases/latest/download/$ASSET"
chmod +x "$INSTALL_DIR/$ASSET" 2>/dev/null || true
```

**Windows PowerShell:**

```powershell
$installDir = "$env:USERPROFILE\.claude\pet-data\bin"
New-Item -ItemType Directory -Path $installDir -Force | Out-Null
$asset = "claude-status-pet-windows-x64.exe"
Invoke-WebRequest -Uri "https://github.com/moeyui1/claude-status-pet/releases/latest/download/$asset" -OutFile "$installDir\$asset"
```

## Manual: Install Copilot Hooks

**macOS / Linux / Git Bash:**

```bash
RAW="https://raw.githubusercontent.com/moeyui1/claude-status-pet/main"

HOOKS_DIR="$HOME/.copilot/hooks"
mkdir -p "$HOOKS_DIR"
curl -sLo "$HOOKS_DIR/status-pet.json" "$RAW/copilot/hooks.json"

SCRIPTS_DIR="$HOME/.claude/pet-data/scripts"
mkdir -p "$SCRIPTS_DIR"
curl -sLo "$SCRIPTS_DIR/copilot-hook.sh" "$RAW/copilot/scripts/hook.sh"
curl -sLo "$SCRIPTS_DIR/copilot-hook.ps1" "$RAW/copilot/scripts/hook.ps1"
chmod +x "$SCRIPTS_DIR/copilot-hook.sh" 2>/dev/null || true
```

**Windows PowerShell:**

```powershell
$RAW = "https://raw.githubusercontent.com/moeyui1/claude-status-pet/main"

$hooksDir = "$env:USERPROFILE\.copilot\hooks"
New-Item -ItemType Directory -Path $hooksDir -Force | Out-Null
Invoke-WebRequest -Uri "$RAW/copilot/hooks.json" -OutFile "$hooksDir\status-pet.json"

$scriptsDir = "$env:USERPROFILE\.claude\pet-data\scripts"
New-Item -ItemType Directory -Path $scriptsDir -Force | Out-Null
Invoke-WebRequest -Uri "$RAW/copilot/scripts/hook.sh" -OutFile "$scriptsDir\copilot-hook.sh"
Invoke-WebRequest -Uri "$RAW/copilot/scripts/hook.ps1" -OutFile "$scriptsDir\copilot-hook.ps1"
```

## Manual: Download Assets

**macOS / Linux / Git Bash:**

```bash
ASSETS_DIR="$HOME/.claude/pet-data/assets"
mkdir -p "$ASSETS_DIR"
curl -sLo /tmp/pet-assets.zip "https://github.com/moeyui1/claude-status-pet/releases/latest/download/pet-assets.zip"
unzip -o /tmp/pet-assets.zip -d "$ASSETS_DIR"
rm -f /tmp/pet-assets.zip
```

**Windows PowerShell:**

```powershell
$assetsDir = "$env:USERPROFILE\.claude\pet-data\assets"
New-Item -ItemType Directory -Path $assetsDir -Force | Out-Null
Invoke-WebRequest -Uri "https://github.com/moeyui1/claude-status-pet/releases/latest/download/pet-assets.zip" -OutFile "$env:TEMP\pet-assets.zip"
Expand-Archive -Path "$env:TEMP\pet-assets.zip" -DestinationPath $assetsDir -Force
Remove-Item "$env:TEMP\pet-assets.zip"
```
