# Claude Status Pet — GitHub Copilot Integration

## Quick Setup

Run in your repo directory:

```bash
bash <(curl -sL https://raw.githubusercontent.com/moeyui1/claude-status-pet/main/copilot/setup.sh)
```

This installs the hook script and adds `.github/hooks/status-pet.json` to your repo. Then commit it:

```bash
git add .github/hooks/status-pet.json
git commit -m "Add status pet hooks for Copilot"
```

If you already have the pet via the Claude Code plugin, you're done — the same binary works for both. If not, grab it from [Releases](https://github.com/moeyui1/claude-status-pet/releases).

## How It Works

Copilot hooks fire on the same lifecycle events:

| Copilot Hook | Pet State | Animation |
|-------------|-----------|-----------|
| `sessionStart` | idle | Floating |
| `userPromptSubmitted` | thinking | Tilting |
| `preToolUse` | reading/editing/searching/running | Varies by tool |
| `postToolUse` | (same) | Varies |
| `errorOccurred` | error | Shake |
| `sessionEnd` | (closes) | Auto-close |

Sessions show "(Copilot)" next to the name. Works alongside Claude Code — each gets its own pet.

## Note

Copilot hooks are per-repo (`.github/hooks/`), not global. You need to add the hooks file to each repo where you want the pet.
