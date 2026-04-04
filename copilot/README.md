# Claude Status Pet — GitHub Copilot CLI Plugin

## Quick Install (Plugin)

```
copilot plugin marketplace add moeyui1/claude-status-pet
copilot plugin install claude-status-pet-copilot
```

This installs the hooks and `/pet` skill automatically.

> **Note:** You still need to download the binary separately. Use `/pet update` after installing the plugin, or download it manually from [Releases](https://github.com/moeyui1/claude-status-pet/releases).

## Manual Install (Per-Repo)

If you prefer per-repo hooks instead of the plugin:

```bash
mkdir -p .github/hooks
curl -sLo .github/hooks/status-pet.json \
  https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/hooks.json
git add .github/hooks/status-pet.json
git commit -m "Add status pet hooks for Copilot"
```

## How It Works

Copilot hooks fire on lifecycle events:

| Copilot Hook | Pet State | Animation |
|-------------|-----------|-----------|
| `sessionStart` | thinking | Tilting |
| `userPromptSubmitted` | _(ignored)_ | — |
| `preToolUse` | reading/editing/searching/running | Varies by tool |
| `postToolUse` | thinking | Tilting |
| `stop` | idle | Floating |
| `errorOccurred` | error | Shake |
| `sessionEnd` | offline/idle | Varies by reason |

Sessions show "(Copilot)" next to the project name.

## Commands

After installing the plugin, use `/pet` in Copilot CLI:

- `/pet` or `/pet on` — Launch the pet
- `/pet update` — Update binary, hooks, skill, and assets
- `/pet set <character>` — Switch character
- `/pet status` — Show config and active sessions
- `/pet help` — Show all commands
