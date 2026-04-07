---
description: "Bump version across all files, generate changelog, commit and tag for release"
agent: "agent"
argument-hint: "version (e.g. 1.1.0) or leave blank to bump patch"
---

Release a new version of claude-status-pet.

## Version

- Read current version from [plugin.json](../../claude/.claude-plugin/plugin.json)
- If a version was specified use that, otherwise bump the patch number (e.g. 1.0.5 → 1.0.6)

## Update version in ALL 6 files

These must stay in sync:

1. `claude/.claude-plugin/plugin.json` → `"version"` field
2. `copilot/plugin.json` → `"version"` field
3. `vscode/plugin.json` → `"version"` field
4. `pet-app/src-tauri/tauri.conf.json` → `"version"` field
5. `pet-app/src-tauri/Cargo.toml` → `version` field
6. `pet-app/package.json` → `"version"` field

## Generate changelog

```
git log --oneline <last-tag>..HEAD
```

Group by: **Features**, **Fixes**, **Docs**, **Other**.

## Commit, tag, and push

Commit message:

```
Release v<version>

<changelog summary>
```

Include trailer: `Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>`

Then:

```
git tag v<version>
git push origin main --tags
```

CI auto-builds binaries — do NOT manually build. Pre-release tags (`-rc`, `-beta`) are marked as pre-release.
