# Contributing New Characters

Want to add a new character to Claude Status Pet? Here's how.

## Character Types

The pet supports three character formats:

| Format | Example | Best for |
|--------|---------|----------|
| **SVG images** | Ferris | Detailed vector illustrations |
| **GIF animations** | Mona (GitHub) | Animated mascots, stickers |
| **ASCII art** | Chonk, Cat, Ghost | Lightweight text art |

## Adding an SVG/GIF Character

### 1. Prepare your images

You need one image per state:

| State | When it shows | Suggested pose |
|-------|--------------|----------------|
| `idle` | Waiting for input | Relaxed, happy, waving |
| `thinking` | Processing a prompt | Curious, looking up, question mark |
| `working` | Using a tool | Busy, typing, focused |
| `delegating` | Spawning sub-agents | Pointing, directing |
| `offline` | Session ended | Sleeping, faded, zzz |

**Guidelines:**
- Transparent background (PNG or GIF with transparency)
- Square aspect ratio works best (the display area is ~140x140px)
- Keep file sizes reasonable (<2MB per image, <10MB total per character)
- SVGs are preferred for static images (infinitely scalable, tiny file size)
- GIFs are great for animated characters

### 2. Add your images

Create a directory under `pet-app/src/`:

```
pet-app/src/my-character/
├── idle.gif       (or .svg / .png)
├── thinking.gif
├── working.gif
├── delegating.gif
└── offline.gif
```

### 3. Register in app.js

Add a state-to-image mapping:

```javascript
const MY_CHARACTER_MAP = {
  idle: 'my-character/idle.gif',
  thinking: 'my-character/thinking.gif',
  working: 'my-character/working.gif',
  delegating: 'my-character/delegating.gif',
  offline: 'my-character/offline.gif',
  unknown: 'my-character/idle.gif',
};
```

Add the rendering branch in `updateStatus()`:

```javascript
} else if (mode === 'my-character') {
    showFerris(); // reuses the <img> element
    const img = MY_CHARACTER_MAP[state] || MY_CHARACTER_MAP.idle;
    if (!ferrisImg.src.endsWith(img)) {
      ferrisImg.style.opacity = '0';
      setTimeout(() => {
        ferrisImg.src = img;
        ferrisImg.style.opacity = '1';
      }, 150);
    }
```

Add to the right-click menu in `buildMenu()`:

```javascript
addMenuItem(charMenu, 'My Character', () => selectChar('my-character'),
  mode === 'my-character' ? 'active' : '');
```

### 4. Test

```bash
cd pet-app && npx tauri build
```

Then right-click the pet and select your character.

---

## Adding an ASCII Art Character

### 1. Define your frames

Each state needs 1-3 animation frames. Each frame is 5 lines of 12 characters:

```javascript
const MY_ASCII = {
  name: 'My Character',
  idle: [
    // Frame 1
    ['            ', '   .----.   ', '  ( {E}  {E} )  ', '  (      )  ', '   `----´   '],
    // Frame 2 (optional)
    ['            ', '   .----.   ', '  ( {E}  {E} )  ', '  (  --  )  ', '   `----´   '],
  ],
  thinking: [ /* frames */ ],
  working:  [ /* frames */ ],
  offline:  [ /* frames */ ],
};
```

`{E}` is replaced with the user's chosen eye style (·, ✦, ×, ◉, @, °).

### 2. Register in app.js

Add your character to the `ASCII_SPECIES` object:

```javascript
ASCII_SPECIES.mychar = {
  name: 'My Character',
  idle: [ /* ... */ ],
  thinking: [ /* ... */ ],
  working: [ /* ... */ ],
  offline: [ /* ... */ ],
};
```

It will automatically appear in the right-click menu.

---

## Submitting Your Character

1. Fork the repo
2. Add your character files and code changes
3. Open a PR with:
   - Screenshot/GIF of your character in action
   - Credit/attribution for the art (if not original)
   - License compatibility (must be compatible with MIT)

## Art Credits

Please ensure you have the right to use the images:
- **CC0 / Public Domain** — always ok
- **CC-BY** — ok with attribution (add to Credits section in README)
- **Original art** — always welcome!
- **Copyrighted** — not allowed without explicit permission
