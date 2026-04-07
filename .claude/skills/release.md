---
name: release
description: Bump version and release a new version with git tag
user-invocable: true
---

# /release command

Release a new version of claude-status-pet.

## Steps

1. Determine the next version:
   - Read current version from `.claude-plugin/plugin.json`
   - If user specified a version (e.g. `/release 0.6.0`), use that
   - Otherwise bump the patch version (e.g. 0.5.2 → 0.5.3)

2. Update version in ALL of these files (they must stay in sync):
   - `claude/.claude-plugin/plugin.json` → `"version"` field
   - `copilot/plugin.json` → `"version"` field
   - `vscode/plugin.json` → `"version"` field
   - `pet-app/src-tauri/tauri.conf.json` → `"version"` field
   - `pet-app/src-tauri/Cargo.toml` → `version` field
   - `pet-app/package.json` → `"version"` field

3. Generate a changelog from commits since the last tag:
   ```
   git log --oneline <last-tag>..HEAD
   ```

4. Commit with message:
   ```
   Release v<version>

   <changelog summary grouped by: Fixes, Features, Docs, Other>
   ```

5. Tag and push:
   ```
   git tag v<version>
   git push origin main --tags
   ```

6. Report the release version and confirm CI is building.

## Notes

- Always include `Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>` trailer
- CI (`.github/workflows/release.yml`) auto-builds binaries on version tags
- Do NOT manually build — CI handles cross-platform builds
