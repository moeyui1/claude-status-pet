# Contributing New Characters

Want to add a new character to Claude Status Pet? Here's how.

## Character Types

| Format | Example | Config | Best for |
|--------|---------|--------|----------|
| **SVG images** | Ferris | `character.json` in bundled frontend | Detailed vector illustrations |
| **GIF animations** | Mona, Kuromi | `character.json` in assets dir | Animated mascots, stickers |
| **ASCII art** | Chonk, Cat, Ghost | Hardcoded in `app.js` | Lightweight text art |

## The `character.json` Format

Every SVG/GIF character is defined by a `character.json` file:

```json
{
  "name": "My Character",
  "type": "gif",
  "states": {
    "idle":       ["mychar/happy.gif", "mychar/wave.gif"],
    "thinking":   ["mychar/curious.gif"],
    "reading":    ["mychar/reading.gif"],
    "editing":    ["mychar/typing.gif"],
    "searching":  ["mychar/looking.gif"],
    "running":    ["mychar/busy.gif"],
    "delegating": ["mychar/pointing.gif"],
    "waiting":    ["mychar/waiting.gif"],
    "error":      ["mychar/sad.gif"],
    "offline":    ["mychar/sleeping.gif"],
    "unknown":    ["mychar/happy.gif"]
  }
}
```

**Fields:**
- **`name`** — Display name shown in the right-click menu
- **`type`** — `"gif"` or `"svg"` (determines rendering mode)
- **`states`** — Maps each pet state to an array of image paths (one is picked randomly)

**States reference:**

| State | When it shows | Suggested pose |
|-------|--------------|----------------|
| `idle` | Waiting for input | Relaxed, happy |
| `thinking` | Processing a prompt | Curious, looking up |
| `reading` | Reading files | Focused, calm |
| `editing` | Writing/editing files | Typing, busy |
| `searching` | Searching code | Looking around |
| `running` | Running commands | Energetic |
| `delegating` | Spawning sub-agents | Pointing, directing |
| `waiting` | Awaiting approval | Anxious, alert |
| `error` | Something failed | Sad, angry |
| `offline` | Session ended | Sleeping, faded |
| `unknown` | Unmapped state | Fallback (usually same as idle) |

Multiple images per state adds variety — the app randomly picks one each time.

## Adding a Bundled SVG Character

For characters bundled with the app (like Ferris):

1. Create a directory: `pet-app/src/mychar/`
2. Add SVG files: `mychar/1.svg`, `mychar/2.svg`, etc.
3. Create `pet-app/src/mychar/character.json`:

```json
{
  "name": "My Character",
  "type": "svg",
  "states": {
    "idle": ["mychar/1.svg"],
    "thinking": ["mychar/2.svg"],
    ...
  }
}
```

4. The app will auto-discover it from the bundled frontend.

## Adding a GIF DLC Character

For characters downloaded at runtime (like Mona, Kuromi):

### 1. Create a DLC config file

Create `dlc/mychar.json`:

```json
{
  "name": "My Character",
  "type": "gif",
  "downloads": [
    { "path": "mychar/happy.gif", "url": "https://example.com/happy.gif" },
    { "path": "mychar/typing.gif", "url": "https://example.com/typing.gif" }
  ],
  "states": {
    "idle": ["mychar/happy.gif"],
    "thinking": ["mychar/typing.gif"],
    "reading": ["mychar/happy.gif"],
    "editing": ["mychar/typing.gif"],
    "searching": ["mychar/typing.gif"],
    "running": ["mychar/typing.gif"],
    "delegating": ["mychar/happy.gif"],
    "waiting": ["mychar/happy.gif"],
    "error": ["mychar/happy.gif"],
    "offline": ["mychar/happy.gif"],
    "unknown": ["mychar/happy.gif"]
  }
}
```

**Fields:**
- **`name`** — Display name shown in the right-click menu
- **`type`** — `"gif"` or `"svg"`
- **`downloads`** — Array of `{path, url}` pairs. `path` is the local file path under the assets dir; `url` is where to download it from
- **`states`** — Maps each pet state to an array of image paths (same as `character.json`)

The app auto-discovers all `dlc/*.json` files and shows them in the DLC menu. No code changes needed.

### 2. Test

```bash
cd pet-app && npx tauri build
```

Right-click the pet → select your character from the DLC section.

**Image guidelines:**
- Transparent background (GIF with transparency)
- Square aspect ratio (~140x140px display area)
- Keep file sizes reasonable (<2MB per image, <10MB total)

---

## Installing a Custom Character Pack

Users can install character packs **without rebuilding** — just drop a folder into the custom characters directory:

```
~/.claude/pet-data/characters/my-pack/
├── character.json
├── idle.gif
├── thinking.gif
└── ...
```

The pet will auto-discover it on next launch and show it under **Custom** in the right-click menu.

> **Note:** You must restart the pet for new packs to appear. Close via right-click → Exit, then relaunch. Or use `/pet close` then `/pet open` if using Claude Code.

### Sharing Packs

To share a character pack:
1. Zip the character directory (e.g., `my-pack/`)
2. Share the zip file
3. Recipients extract it to `~/.claude/pet-data/characters/`

```bash
# Install a shared pack
unzip my-pack.zip -d ~/.claude/pet-data/characters/
# Restart pet to see it
```

### Directory Structure

```
~/.claude/pet-data/
├── assets/              ← DLC characters (mona, kuromi)
│   ├── mona/
│   │   ├── character.json
│   │   └── *.gif
│   └── kuromi/
│       ├── character.json
│       └── *.gif
├── characters/          ← Custom user packs (drop packs here)
│   └── my-pack/
│       ├── character.json
│       └── *.gif
└── bin/                 ← Pet binary
```

---

## Adding a New AI Agent Adapter

The pet supports any AI agent via the binary's CLI interface. There are three integration methods, from easiest to most optimized:

### Method 1: CLI Args (zero code, any agent)

Configure your agent's hooks to call the binary with generic arguments:

```
claude-status-pet write-status --event <event> --tool <tool> --detail "<text>" --session-id <id>
```

| `--event` | Maps to state |
|-----------|---------------|
| `prompt` | thinking |
| `tool` | depends on `--tool` (auto-mapped) |
| `done` | idle |
| `error` | error |
| `offline` | offline |

Tool names are fuzzy-matched: `edit`, `replace_string_in_file`, `my_editor` all → `editing`.

### Method 2: Built-in Adapter (Rust, PR required)

For deeper integration, create an adapter in `pet-app/src-tauri/src/adapter/`:

```
adapter/
├── mod.rs        ← register new adapter here
├── claude.rs     ← Claude Code adapter
├── copilot.rs    ← GitHub Copilot adapter
└── myagent.rs    ← your adapter
```

**1. Create `adapter/myagent.rs`:**

```rust
use super::{Adapter, NormalizedEvent, StdinInput};

pub struct MyAgentAdapter;

impl Adapter for MyAgentAdapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent> {
        // Parse your agent's JSON format from stdin
        // Map events to normalized: prompt, tool, done, error, offline
        // Handle any agent-specific quirks here
        Some(NormalizedEvent {
            event: "tool".into(),
            tool: "edit".into(),
            detail: "Editing file.rs".into(),
            session_id: "my-session".into(),
            session_name: "My Project".into(),
            launch_only: false,
        })
    }
}
```

**2. Register in `adapter/mod.rs`:**

```rust
pub mod myagent;

pub fn get_adapter(name: &str) -> Option<Box<dyn Adapter>> {
    match name {
        "claude" => Some(Box::new(claude::ClaudeAdapter)),
        "copilot" => Some(Box::new(copilot::CopilotAdapter)),
        "myagent" => Some(Box::new(myagent::MyAgentAdapter)),
        _ => None,
    }
}
```

**3. Usage:**

```
claude-status-pet write-status --adapter myagent < stdin.json
```

**Key design rules for adapters:**
- All agent-specific quirks belong INSIDE the adapter (not in shared code)
- The adapter normalizes to generic events: `prompt`, `tool`, `done`, `error`, `offline`
- Tool→state mapping is shared (`status_map.rs`) — don't duplicate it
- Set `launch_only: true` if the event should only launch the GUI (not write status)
- Add tests in `tests.rs`

### Method 3: External Adapter Config (planned)

Future: JSON config files for custom adapters without Rust code.

---

## Adding an ASCII Art Character

ASCII characters are hardcoded in `app.js` (they're just text, no external files needed).

### 1. Define your frames

Each state needs 1-3 animation frames. Each frame is 5 lines of 12 characters:

```javascript
const MY_ASCII = {
  name: 'My Character',
  idle: [
    ['            ', '   .----.   ', '  ( {E}  {E} )  ', '  (      )  ', '   `----´   '],
    ['            ', '   .----.   ', '  ( {E}  {E} )  ', '  (  --  )  ', '   `----´   '],
  ],
  thinking: [ /* frames */ ],
  working:  [ /* frames */ ],
  offline:  [ /* frames */ ],
};
```

`{E}` is replaced with the user's chosen eye style (·, ✦, ×, ◉, @, °).

### 2. Register in `app.js`

Add your character to the `ASCII_SPECIES` object. It will automatically appear in the ASCII Buddies submenu.

---

## Submitting

1. Fork the repo
2. Add your files (character pack, adapter, or ASCII art)
3. Add tests for adapters (`tests.rs`)
4. Open a PR with:
   - Screenshot/GIF showing it working
   - Credit/attribution for art (if not original)
   - License compatibility

## Art Credits

Please ensure you have the right to use the images:
- **CC0 / Public Domain** — always ok
- **CC-BY** — ok with attribution (add to Credits in README)
- **Original art** — always welcome!
- **Copyrighted** — not allowed without explicit permission
