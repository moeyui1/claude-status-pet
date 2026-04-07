# Claude Status Pet — GitHub Copilot CLI Plugin

## Quick Install (Plugin)

```
copilot plugin marketplace add moeyui1/claude-status-pet
copilot plugin install claude-status-pet@claude-status-pet
```

This installs the hooks and `/pet` skill automatically.

> **Note:** You still need to download the binary separately. Use `/pet update` after installing the plugin.

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
