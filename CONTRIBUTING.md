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

### 1. Add GIF URLs to `scripts/download-gifs.js`

```javascript
const GIFS = {
  // ... existing entries ...
  'mychar/happy.gif': 'https://media.giphy.com/media/.../giphy.gif',
  'mychar/typing.gif': 'https://media.giphy.com/media/.../giphy.gif',
  // ...
};
```

### 2. Add character config in the same file

In the `characters` object inside `main()`:

```javascript
const characters = {
  // ... existing entries ...
  mychar: {
    name: 'My Character',
    type: 'gif',
    states: {
      idle: ['mychar/happy.gif'],
      thinking: ['mychar/curious.gif'],
      // ...
    }
  }
};
```

### 3. Register in the DLC menu

In `pet-app/src/app.js`, add to the `knownDlcs` array in `buildMenu()`:

```javascript
const knownDlcs = [
  ['mona', 'Mona (GitHub)'],
  ['kuromi', 'Kuromi (Sanrio)'],
  ['mychar', 'My Character'],  // ← add here
];
```

### 4. Bump the GIF version

Increment `CURRENT_VERSION` in `download-gifs.js` to force re-download:

```javascript
const CURRENT_VERSION = '3';  // was '2'
```

### 5. Test

```bash
cd pet-app && npx tauri build
```

Right-click the pet → select your character from the DLC section.

**Image guidelines:**
- Transparent background (GIF with transparency)
- Square aspect ratio (~140x140px display area)
- Keep file sizes reasonable (<2MB per image, <10MB total)

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

Add your character to the `ASCII_SPECIES` object. It will automatically appear in the menu.

---

## Submitting Your Character

1. Fork the repo
2. Add your character files and `character.json`
3. Open a PR with:
   - Screenshot/GIF of your character in action
   - Credit/attribution for the art (if not original)
   - License compatibility (must be compatible with the project license)

## Art Credits

Please ensure you have the right to use the images:
- **CC0 / Public Domain** — always ok
- **CC-BY** — ok with attribution (add to Credits in README)
- **Original art** — always welcome!
- **Copyrighted** — not allowed without explicit permission
