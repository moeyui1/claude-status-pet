// Claude Status Pet — supports SVG, GIF, and ASCII art characters

const bubble = document.getElementById('speech-bubble');
const statusText = document.getElementById('status-text');
const stateLabel = document.getElementById('state-label');
const sessionNameEl = document.getElementById('session-name');
const container = document.getElementById('pet-container');
const imgWrapper = document.getElementById('ferris-wrapper');
const imgEl = document.getElementById('ferris-img');
const asciiPre = document.getElementById('ascii-art');
const charMenu = document.getElementById('char-menu');
const menuBackdrop = document.getElementById('menu-backdrop');

// ── Character data ──

const ASCII_SPECIES = {
  chonk: {
    name: 'Voidchisel (Chonk)',
    idle: [
      ['            ', '  /\\    /\\  ', ' ( {E}    {E} ) ', ' (   ..   ) ', '  `------´  '],
      ['            ', '  /\\    /|  ', ' ( {E}    {E} ) ', ' (   ..   ) ', '  `------´  '],
      ['            ', '  /\\    /\\  ', ' ( {E}    {E} ) ', ' (   ..   ) ', '  `------´~ '],
    ],
    thinking: [
      ['     ?      ', '  /\\    /\\  ', ' ( {E}    {E} ) ', ' (   ..   ) ', '  `------´  '],
      ['    ??      ', '  /\\    /\\  ', ' ( {E}    {E} ) ', ' (   ..   ) ', '  `------´  '],
    ],
    working: [
      ['            ', '  /\\    /\\  ', ' ( {E}    {E} ) ', ' (   ..   ) ', '  `------´  '],
      ['     *      ', '  /\\    /\\  ', ' ( {E}    {E} ) ', ' (   ><   ) ', '  `------´  '],
      ['    **      ', '  /\\    /|  ', ' ( {E}    {E} ) ', ' (   ><   ) ', '  `------´  '],
    ],
    offline: [
      ['    z z     ', '  /\\    /\\  ', ' ( -    - ) ', ' (   ..   ) ', '  `------´  '],
      ['   z z z    ', '  /\\    /\\  ', ' ( -    - ) ', ' (   ..   ) ', '  `------´  '],
    ],
  },
  cat: {
    name: 'Cat',
    idle: [
      ['            ', '   /\\_/\\    ', '  ( {E}   {E})  ', '  (  ω  )   ', '  (")_(")   '],
      ['            ', '   /\\_/\\    ', '  ( {E}   {E})  ', '  (  ω  )   ', '  (")_(")~  '],
    ],
    thinking: [
      ['     ?      ', '   /\\_/\\    ', '  ({E}    {E})  ', '  (  ω  )   ', '  (")_(")   '],
    ],
    working: [
      ['     *      ', '   /\\_/\\    ', '  ( {E}   {E})  ', '  (  >< )   ', '  (")_(")~  '],
    ],
    offline: [
      ['   z z z    ', '   /\\_/\\    ', '  ( -   -)  ', '  (  ω  )   ', '  (")_(")   '],
    ],
  },
  ghost: {
    name: 'Ghost',
    idle: [
      ['            ', '   .----.   ', '  / {E}  {E} \\  ', '  |      |  ', '  ~`~``~`~  '],
      ['            ', '   .----.   ', '  / {E}  {E} \\  ', '  |      |  ', '  `~`~~`~`  '],
    ],
    thinking: [
      ['    ~  ~    ', '   .----.   ', '  / {E}  {E} \\  ', '  |  ??  |  ', '  ~`~``~`~  '],
    ],
    working: [
      ['   ~ ~  ~   ', '   .----.   ', '  / {E}  {E} \\  ', '  |  **  |  ', '  ~~`~~`~~  '],
    ],
    offline: [
      ['            ', '   .----.   ', '  / -  - \\  ', '  |      |  ', '  ~`~``~`~  '],
    ],
  },
  robot: {
    name: 'Robot',
    idle: [
      ['            ', '   .[||].   ', '  [ {E}  {E} ]  ', '  [ ==== ]  ', '  `------´  '],
      ['            ', '   .[||].   ', '  [ {E}  {E} ]  ', '  [ -==- ]  ', '  `------´  '],
    ],
    thinking: [
      ['    * *     ', '   .[||].   ', '  [ {E}  {E} ]  ', '  [ ???? ]  ', '  `------´  '],
    ],
    working: [
      ['    ***     ', '   .[||].   ', '  [ {E}  {E} ]  ', '  [ <<>> ]  ', '  `------´  '],
    ],
    offline: [
      ['            ', '   .[||].   ', '  [ -  - ]  ', '  [ .... ]  ', '  `------´  '],
    ],
  },
  duck: {
    name: 'Duck',
    idle: [
      ['            ', '    __      ', '  <({E} )___  ', '   (  ._>   ', '    `--´    '],
      ['            ', '    __      ', '  <({E} )___  ', '   (  ._>   ', '    `--´~   '],
    ],
    thinking: [
      ['     ?      ', '    __      ', '  <({E} )___  ', '   (  ._>   ', '    `--´    '],
    ],
    working: [
      ['     !      ', '    __      ', '  <({E} )___  ', '   (  .__>  ', '    `--´~   '],
    ],
    offline: [
      ['   z z      ', '    __      ', '  <(- )___  ', '   (  ._>   ', '    `--´    '],
    ],
  },
  snail: {
    name: 'Snail',
    idle: [
      ['            ', ' {E}    .--.  ', '  \\  ( @ )  ', '   \\_`--´   ', '  ~~~~~~~   '],
      ['            ', '  {E}   .--.  ', '  |  ( @ )  ', '   \\_`--´   ', '  ~~~~~~~   '],
    ],
    thinking: [
      ['     ?      ', ' {E}    .--.  ', '  \\  ( @ )  ', '   \\_`--´   ', '  ~~~~~~~   '],
    ],
    working: [
      ['     !      ', ' {E}    .--.  ', '  \\  ( @  ) ', '   \\_`--´   ', '   ~~~~~~   '],
      ['    !!      ', '  {E}   .--.  ', '  |  ( @ )  ', '   \\_`--´   ', '  ~~~~~~~   '],
    ],
    offline: [
      ['   z z z    ', ' -    .--.  ', '  \\  ( @ )  ', '   \\_`--´   ', '  ~~~~~~~   '],
    ],
  },
  axolotl: {
    name: 'Axolotl',
    idle: [
      ['            ', '}~(______)~{', '}~({E} .. {E})~{', '  ( .--. )  ', '  (_/  \\_)  '],
      ['            ', '~}(______){~', '~}({E} .. {E}){~', '  ( .--. )  ', '  (_/  \\_)  '],
    ],
    thinking: [
      ['     ?      ', '}~(______)~{', '}~({E} .. {E})~{', '  ( .--. )  ', '  (_/  \\_)  '],
    ],
    working: [
      ['    !!      ', '~}(______){~', '~}({E} .. {E}){~', '  (  --  )  ', '  ~_/  \\_~  '],
    ],
    offline: [
      ['   z z z    ', '}~(______)~{', '}~(- .. -)~{', '  ( .--. )  ', '  (_/  \\_)  '],
    ],
  },
};

// GIF character maps loaded from character.json files (populated at init)
const GIF_MODES = {};

// Ferris SVG map loaded from character.json (populated at init, fallback to hardcoded)
let FERRIS_SVG_MAP = {
  idle: ['ferris/1.svg'], thinking: ['ferris/3.svg', 'ferris/14.svg'],
  reading: ['ferris/10.svg'], editing: ['ferris/19.svg'],
  searching: ['ferris/20.svg'], running: ['ferris/2.svg'],
  delegating: ['ferris/15.svg'], waiting: ['ferris/5.svg'],
  error: ['ferris/9.svg'], offline: ['ferris/7.svg'], unknown: ['ferris/1.svg'],
};

const ASCII_ANIM_SPEED = {
  working: 300, editing: 300, running: 300,
  thinking: 600, searching: 600,
};
const DEFAULT_ANIM_SPEED = 800;

// ── State ──
let mode = localStorage.getItem('petMode') || 'ferris';
let eye = localStorage.getItem('petEye') || '·';
let petColor = localStorage.getItem('petColor') || '';
let petBgColor = localStorage.getItem('petBgColor') || '';
let petFillColor = localStorage.getItem('petFillColor') || '';
let petTextColor = localStorage.getItem('petTextColor') || '';
let petSessionBg = localStorage.getItem('petSessionBg') || '';
let petFontSize = parseInt(localStorage.getItem('petFontSize') || '16');
let petScale = parseFloat(localStorage.getItem('petScale') || '1');
let currentState = 'idle';
let currentImgSrc = '';
let bubbleTimeout = null;
let asciiFrame = 0;
let asciiInterval = null;
let menuPage = 'main';
let appVersion = '0.0.0';
let customPacks = [];
let boundSessionId = '';
let sessionPoll = null;
const dlcInstalledCache = {};

function pickRandom(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function startDrag() {
  if (window.__TAURI__) window.__TAURI__.window.getCurrentWindow().startDragging();
}

// ── Apply visual config ──
function applyConfig() {
  const colorProps = [
    [asciiPre, 'color', petColor],
    [stateLabel, 'color', petColor],
    [sessionNameEl, 'background', petSessionBg],
    [sessionNameEl, 'color', petTextColor],
    [container, 'background', petBgColor],
    [container, 'borderRadius', petBgColor ? '12px' : ''],
    [asciiPre, 'background', petFillColor],
  ];
  for (const [el, prop, val] of colorProps) {
    el.style[prop] = val || '';
  }
  // Sync bubble border + tail with label background color
  const bubbleColor = petSessionBg && petSessionBg !== 'transparent' ? petSessionBg : '';
  bubble.style.setProperty('--bubble-color', bubbleColor);
  asciiPre.style.fontSize = petFontSize + 'px';
  container.style.width = Math.round(200 * petScale) + 'px';
  container.style.height = Math.round(240 * petScale) + 'px';

  // Scale inner elements
  imgWrapper.style.width = Math.round(140 * petScale) + 'px';
  imgWrapper.style.height = Math.round(140 * petScale) + 'px';
  statusText.style.fontSize = Math.round(13 * petScale) + 'px';
  sessionNameEl.style.fontSize = Math.round(12 * petScale) + 'px';
  stateLabel.style.fontSize = Math.round(12 * petScale) + 'px';
  bubble.style.maxWidth = Math.round(180 * petScale) + 'px';

  // Resize window to match
  if (window.__TAURI__) {
    const w = Math.round(200 * petScale);
    const h = Math.round(240 * petScale);
    const LS = window.__TAURI__.dpi?.LogicalSize || window.__TAURI__.window?.LogicalSize;
    if (LS) {
      window.__TAURI__.window.getCurrentWindow().setSize(new LS(w, h));
    }
  }
}

function saveConfig(key, value) {
  localStorage.setItem(key, value);
  applyConfig();
}

// ── Renderers ──

function showImage() {
  imgWrapper.style.display = 'flex';
  asciiPre.style.display = 'none';
}

function showAscii() {
  imgWrapper.style.display = 'none';
  asciiPre.style.display = 'block';
}

function setImage(src) {
  const resolved = assetUrl(src);
  if (resolved === currentImgSrc) return;
  // If asset not yet cached, try loading it on-demand (fixes race with preload)
  if (resolved === src && hasExternalAssets && window.__TAURI__) {
    loadAsset(src).then(url => {
      if (url !== src) setImage(src); // retry with cached version
    });
    return;
  }
  imgEl.style.opacity = '0';
  setTimeout(() => {
    imgEl.src = resolved;
    imgEl.style.opacity = '1';
  }, 150);
  currentImgSrc = resolved;
}

// Show ASCII 404 art when image fails to load
imgEl.addEventListener('error', () => {
  if (imgEl.src && imgEl.src !== window.location.href) {
    const name = decodeURIComponent(imgEl.src.split('/').pop().split('?')[0]);
    // Switch to ASCII 404 display
    showAscii();
    asciiPre.textContent = [
      '            ',
      '   4  0  4  ',
      '   ╭──────╮ ',
      '   │ ×  × │ ',
      '   │  __  │ ',
      '   ╰──────╯ ',
    ].join('\n');
    statusText.textContent = 'Image not found: ' + name;
    bubble.classList.remove('hidden');
    clearTimeout(bubbleTimeout);
    bubbleTimeout = setTimeout(() => bubble.classList.add('hidden'), 8000);
  }
});

function renderAsciiFrame(frames, frameIdx) {
  const frame = frames[frameIdx % frames.length];
  asciiPre.textContent = frame.map(line => line.replaceAll('{E}', eye)).join('\n');
}

function startAsciiAnimation(frames) {
  if (asciiInterval) clearInterval(asciiInterval);
  asciiFrame = 0;
  renderAsciiFrame(frames, 0);
  if (frames.length > 1) {
    const speed = ASCII_ANIM_SPEED[currentState] || DEFAULT_ANIM_SPEED;
    asciiInterval = setInterval(() => {
      asciiFrame++;
      renderAsciiFrame(frames, asciiFrame);
    }, speed);
  }
}

// ── Main update ──

function updateStatus(status) {
  const state = status.state || 'idle';
  const detail = status.detail || '';
  const sessionName = status.session_name || '';

  if (state === 'closed' && window.__TAURI__) {
    window.__TAURI__.window.getCurrentWindow().close();
    return;
  }

  if (sessionName) sessionNameEl.textContent = sessionName;

  if (mode === 'ferris') {
    showImage();
    const sprites = FERRIS_SVG_MAP[state] || FERRIS_SVG_MAP.idle;
    setImage(pickRandom(sprites));
  } else if (GIF_MODES[mode]) {
    showImage();
    const map = GIF_MODES[mode];
    setImage(pickRandom(map[state] || map.idle));
  } else {
    showAscii();
    const species = ASCII_SPECIES[mode];
    if (species) {
      const stateKey = (state === 'delegating') ? 'working' : state;
      startAsciiAnimation(species[stateKey] || species.idle);
    }
  }

  if (state !== currentState) {
    container.className = 'anim-appear';
    setTimeout(() => { container.className = `anim-${state}`; }, 400);
    currentState = state;
  }

  stateLabel.textContent = state;

  // Speech bubble — always visible for active states, 30s timeout for idle/offline
  if (detail && state !== 'offline') {
    if (statusText.textContent !== detail) {
      bubble.style.transition = 'none';
      bubble.style.transform = 'scale(0.95)';
      setTimeout(() => {
        bubble.style.transition = 'opacity 0.3s ease, transform 0.15s ease';
        bubble.style.transform = 'scale(1)';
      }, 50);
    }
    statusText.textContent = detail;
    bubble.classList.remove('hidden');
    clearTimeout(bubbleTimeout);
    if (state === 'idle') {
      bubbleTimeout = setTimeout(() => { bubble.classList.add('hidden'); }, 30000);
    }
  } else if (state === 'offline') {
    statusText.textContent = 'Zzz...';
    bubble.classList.remove('hidden');
    clearTimeout(bubbleTimeout);
    bubbleTimeout = setTimeout(() => { bubble.classList.add('hidden'); }, 30000);
  }
}

// ── Menu ──

function addMenuItem(parent, text, onclick, cls) {
  const el = document.createElement('div');
  el.className = 'menu-item' + (cls ? ' ' + cls : '');
  el.textContent = text;
  el.onclick = (e) => { e.stopPropagation(); onclick(); };
  parent.appendChild(el);
}

function addDivider(parent) {
  const d = document.createElement('div');
  d.className = 'menu-divider';
  parent.appendChild(d);
}

function addColorRow(parent, label, currentVal, defaultVal, onchange, allowTransparent) {
  const row = document.createElement('div');
  row.className = 'menu-config-row';
  const lbl = document.createElement('span');
  lbl.className = 'menu-config-label';
  lbl.textContent = label;
  const input = document.createElement('input');
  input.type = 'color';
  input.className = 'menu-color-input';
  input.value = (currentVal && currentVal !== 'transparent') ? currentVal : defaultVal;
  input.oninput = (e) => onchange(e.target.value);
  const reset = document.createElement('span');
  reset.className = 'menu-reset';
  reset.textContent = 'x';
  reset.title = 'Reset to default';
  reset.onclick = () => { input.value = defaultVal; onchange(''); };
  row.appendChild(lbl);
  row.appendChild(input);
  if (allowTransparent) {
    const tp = document.createElement('span');
    tp.className = 'menu-reset';
    tp.textContent = '◻';
    tp.title = 'Transparent';
    tp.onclick = () => onchange('transparent');
    row.appendChild(tp);
  }
  row.appendChild(reset);
  parent.appendChild(row);
}

function addSliderRow(parent, label, currentVal, min, max, step, onchange, unit) {
  const row = document.createElement('div');
  row.className = 'menu-config-row';
  const lbl = document.createElement('span');
  lbl.className = 'menu-config-label';
  lbl.textContent = label;
  const input = document.createElement('input');
  input.type = 'range';
  input.className = 'menu-slider-input';
  input.min = min; input.max = max; input.step = step;
  input.value = currentVal;
  const fmt = (v) => unit === 'px' ? Math.round(v) + 'px' : Math.round(v * 100) + '%';
  const valLabel = document.createElement('span');
  valLabel.className = 'menu-config-value';
  valLabel.textContent = fmt(currentVal);
  input.oninput = (e) => {
    const v = parseFloat(e.target.value);
    valLabel.textContent = fmt(v);
    onchange(v);
  };
  row.appendChild(lbl);
  row.appendChild(input);
  row.appendChild(valLabel);
  parent.appendChild(row);
}

function buildMenu() {
  charMenu.innerHTML = '';
  if (menuPage === 'config') { buildConfigPage(); return; }
  if (menuPage === 'ascii') { buildAsciiPage(); return; }

  // Bundled: Ferris
  addMenuItem(charMenu, 'Ferris (SVG)', () => selectChar('ferris'), mode === 'ferris' ? 'active' : '');
  addDivider(charMenu);

  // ASCII Buddies → submenu
  const asciiActive = Object.keys(ASCII_SPECIES).includes(mode);
  addMenuItem(charMenu, 'ASCII Buddies ' + (asciiActive ? '(' + ASCII_SPECIES[mode].name + ')' : '') + ' ▸', () => { menuPage = 'ascii'; buildMenu(); }, asciiActive ? 'active' : '');
  addDivider(charMenu);

  // DLC characters
  const dlcLabel = document.createElement('div');
  dlcLabel.className = 'menu-section-label';
  dlcLabel.textContent = 'DLC';
  charMenu.appendChild(dlcLabel);

  const knownDlcs = [['mona', 'Mona (GitHub)'], ['kuromi', 'Kuromi (Sanrio)']];
  for (const [key, fallbackLabel] of knownDlcs) {
    const installed = isDlcInstalled(key);
    const cls = mode === key ? 'active' : '';
    if (installed) {
      addMenuItem(charMenu, fallbackLabel, () => selectChar(key), cls);
    } else {
      addMenuItem(charMenu, fallbackLabel + ' ↓', () => downloadAndSelectDlc(key), cls);
    }
  }

  // Custom character packs
  if (customPacks.length > 0) {
    addDivider(charMenu);
    const customLabel = document.createElement('div');
    customLabel.className = 'menu-section-label';
    customLabel.textContent = 'Custom';
    charMenu.appendChild(customLabel);
    for (const pack of customPacks) {
      addMenuItem(charMenu, pack.name, () => selectChar(pack.id), mode === pack.id ? 'active' : '');
    }
  }

  addDivider(charMenu);
  addMenuItem(charMenu, 'Settings...', () => { menuPage = 'config'; buildMenu(); });
  addDivider(charMenu);
  addMenuItem(charMenu, 'Close', () => closeMenu());
  addMenuItem(charMenu, 'Exit', () => {
    closeMenu();
    if (window.__TAURI__) window.__TAURI__.window.getCurrentWindow().close();
  }, 'menu-item-danger');

  const verLabel = document.createElement('div');
  verLabel.className = 'menu-version';
  verLabel.textContent = 'v' + appVersion;
  charMenu.appendChild(verLabel);

  if (boundSessionId) {
    const sidLabel = document.createElement('div');
    sidLabel.className = 'menu-version';
    sidLabel.textContent = boundSessionId.length > 12
      ? boundSessionId.slice(0, 12) + '…'
      : boundSessionId;
    sidLabel.title = boundSessionId;
    charMenu.appendChild(sidLabel);
  }
}

function buildAsciiPage() {
  addMenuItem(charMenu, '← Back', () => { menuPage = 'main'; buildMenu(); });
  addDivider(charMenu);
  for (const [key, species] of Object.entries(ASCII_SPECIES)) {
    addMenuItem(charMenu, species.name, () => { selectChar(key); menuPage = 'main'; }, mode === key ? 'active' : '');
  }
}

function buildConfigPage() {
  addMenuItem(charMenu, '← Back', () => { menuPage = 'main'; buildMenu(); });
  addDivider(charMenu);
  addSliderRow(charMenu, 'Scale', petScale, 1, 2, 0.1, (v) => { petScale = v; saveConfig('petScale', String(v)); }, '%');
  addColorRow(charMenu, 'Text', petTextColor, '#ffffff', (v) => { petTextColor = v; saveConfig('petTextColor', v); });
  addColorRow(charMenu, 'Label', petSessionBg, '#e06c3c', (v) => { petSessionBg = v; saveConfig('petSessionBg', v); }, true);
  addColorRow(charMenu, 'ASCII Fill', petFillColor, '#ffffff', (v) => { petFillColor = v; saveConfig('petFillColor', v); });
  addColorRow(charMenu, 'Background', petBgColor, '#ffffff', (v) => { petBgColor = v; saveConfig('petBgColor', v); });
  addDivider(charMenu);
  addMenuItem(charMenu, 'Reset Default', () => {
    petScale = 1; petTextColor = ''; petSessionBg = ''; petFillColor = ''; petBgColor = '';
    localStorage.removeItem('petScale'); localStorage.removeItem('petTextColor');
    localStorage.removeItem('petSessionBg'); localStorage.removeItem('petFillColor');
    localStorage.removeItem('petBgColor');
    applyConfig();
    buildConfigPage();
  }, 'menu-item-danger');
}

function isDlcInstalled(dlcName) {
  // Check synchronously from cache first
  if (dlcInstalledCache[dlcName] !== undefined) return dlcInstalledCache[dlcName];
  return false;
}

async function downloadAndSelectDlc(dlcName) {
  if (!window.__TAURI__) return;
  closeMenu();

  // Show downloading state with animation
  stateLabel.textContent = 'downloading';
  container.className = 'anim-thinking';
  statusText.textContent = 'Downloading ' + dlcName + '...';
  bubble.classList.remove('hidden');

  try {
    await window.__TAURI__.core.invoke('download_dlc', { dlcName });
    dlcInstalledCache[dlcName] = true;
    // Re-check assets dir (download_dlc may have created it)
    try { hasExternalAssets = !!(await window.__TAURI__.core.invoke('get_assets_dir')); } catch(e) {}
    // Load character.json for the newly downloaded DLC
    try {
      const jsonStr = await window.__TAURI__.core.invoke('load_text_asset', { path: dlcName + '/character.json' });
      if (jsonStr) {
        const config = JSON.parse(jsonStr);
        GIF_MODES[dlcName] = config.states;
      }
    } catch(e) {}
    await selectChar(dlcName);
  } catch (e) {
    statusText.textContent = 'Download failed: ' + (e || 'unknown error');
    bubble.classList.remove('hidden');
    clearTimeout(bubbleTimeout);
    bubbleTimeout = setTimeout(() => bubble.classList.add('hidden'), 5000);
    container.className = 'anim-error';
    stateLabel.textContent = 'error';
  }
}

async function selectChar(newMode) {
  mode = newMode;
  localStorage.setItem('petMode', mode);
  closeMenu();
  currentImgSrc = '';
  imgEl.src = '';

  // Show the character immediately with whatever is available
  updateStatus({ state: currentState, detail: statusText.textContent });

  // Preload remaining assets in the background
  if (GIF_MODES[mode] && hasExternalAssets) {
    preloadAssets();
  }
}

function openMenu() {
  menuPage = 'main';
  buildMenu();
  charMenu.classList.remove('hidden');
  menuBackdrop.classList.remove('hidden');
}

function closeMenu() {
  charMenu.classList.add('hidden');
  menuBackdrop.classList.add('hidden');
}

// Right-click to toggle menu
container.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  e.stopPropagation();
  charMenu.classList.contains('hidden') ? openMenu() : closeMenu();
});

// Close menu on backdrop click, right-click, or window blur
menuBackdrop.addEventListener('click', closeMenu);
menuBackdrop.addEventListener('contextmenu', (e) => { e.preventDefault(); closeMenu(); });
window.addEventListener('blur', closeMenu);

// Drag — single handler for all draggable elements
for (const el of [imgWrapper, asciiPre, bubble, stateLabel]) {
  el.addEventListener('mousedown', startDrag);
}

// ── Session picker ──
async function showSessionPicker() {
  if (sessionPoll) { clearInterval(sessionPoll); sessionPoll = null; }
  const sessions = await window.__TAURI__.core.invoke('list_unlocked_sessions');
  if (sessions.length === 0) {
    statusText.textContent = 'No active sessions';
    bubble.classList.remove('hidden');
    stateLabel.textContent = 'waiting';
    sessionPoll = setInterval(async () => {
      const s = await window.__TAURI__.core.invoke('list_unlocked_sessions');
      if (s.length > 0) { clearInterval(sessionPoll); sessionPoll = null; showSessionPicker(); }
    }, 2000);
    return;
  }
  // Auto-bind to the most recently updated session; random if tied
  const maxMod = Math.max(...sessions.map(s => s.last_modified));
  const newest = sessions.filter(s => s.last_modified === maxMod);
  const pick = newest[Math.floor(Math.random() * newest.length)];
  await bindToSession(pick.session_id);
}

async function bindToSession(sessionId) {
  try {
    await window.__TAURI__.core.invoke('bind_session', { sessionId });
    boundSessionId = sessionId;
    charMenu.classList.add('hidden');
    menuBackdrop.classList.add('hidden');
  } catch (e) {
    statusText.textContent = 'Bind failed: ' + e;
    bubble.classList.remove('hidden');
  }
}

// ── Listen for status updates ──
if (window.__TAURI__) {
  window.__TAURI__.event.listen('status-update', (e) => updateStatus(e.payload));
  window.__TAURI__.core.invoke('get_session_id').then((sid) => {
    if (sid) {
      boundSessionId = sid;
    } else {
      // No explicit session — show session picker
      showSessionPicker();
    }
  });
  window.__TAURI__.core.invoke('get_status').then((s) => { if (s) updateStatus(s); });
} else {
  const demos = [
    { state: 'idle', detail: 'Waiting...' },
    { state: 'thinking', detail: 'Processing prompt...' },
    { state: 'editing', detail: 'Editing main.rs' },
    { state: 'searching', detail: 'Searching: TODO' },
    { state: 'delegating', detail: 'Spawning agent...' },
    { state: 'idle', detail: 'Done!' },
    { state: 'offline', detail: 'Session ended' },
  ];
  let i = 0;
  setInterval(() => updateStatus(demos[i++ % demos.length]), 2500);
}

// ── Asset loading ──
let hasExternalAssets = false;
const assetCache = {};

async function initAssets() {
  if (window.__TAURI__) {
    try {
      hasExternalAssets = !!(await window.__TAURI__.core.invoke('get_assets_dir'));
    } catch(e) {}
  }
}

async function loadAsset(path) {
  if (!hasExternalAssets || !window.__TAURI__) return path;
  if (assetCache[path]) return assetCache[path];
  try {
    // load_asset checks assets/ then characters/ dirs
    const dataUrl = await window.__TAURI__.core.invoke('load_asset', { path });
    if (dataUrl) { assetCache[path] = dataUrl; return dataUrl; }
  } catch(e) {}
  try {
    const dataUrl = await window.__TAURI__.core.invoke('load_custom_asset', { path });
    if (dataUrl) { assetCache[path] = dataUrl; return dataUrl; }
  } catch(e) {}
  return path;
}

function assetUrl(path) {
  return assetCache[path] || path;
}

// Preload only the active mode's assets
async function preloadAssets() {
  let paths = [];
  if (GIF_MODES[mode]) {
    for (const gifs of Object.values(GIF_MODES[mode])) for (const g of gifs) paths.push(g);
  } else if (mode === 'ferris') {
    for (const svgs of Object.values(FERRIS_SVG_MAP)) for (const s of svgs) paths.push(s);
  }
  await Promise.all([...new Set(paths)].map(p => loadAsset(p)));
}

// ── Init ──
(async () => {
  await initAssets();

  // Get app version
  if (window.__TAURI__) {
    try { appVersion = await window.__TAURI__.app.getVersion(); } catch(e) {}
  }

  // Load Ferris character.json (bundled with frontend)
  try {
    const resp = await fetch('ferris/character.json');
    if (resp.ok) {
      const config = await resp.json();
      FERRIS_SVG_MAP = config.states;
    }
  } catch(e) {}

  // Discover DLC and custom characters
  if (window.__TAURI__ && hasExternalAssets) {
    // DLC characters
    for (const dlcName of ['mona', 'kuromi']) {
      try {
        const installed = await window.__TAURI__.core.invoke('is_dlc_installed', { dlcName });
        dlcInstalledCache[dlcName] = installed;
        if (installed) {
          try {
            const jsonStr = await window.__TAURI__.core.invoke('load_text_asset', { path: dlcName + '/character.json' });
            if (jsonStr) {
              const config = JSON.parse(jsonStr);
              GIF_MODES[dlcName] = config.states;
            }
          } catch(e) {}
        }
      } catch(e) { dlcInstalledCache[dlcName] = false; }
    }

    // Custom character packs
    try {
      const packs = await window.__TAURI__.core.invoke('list_character_packs');
      customPacks = packs.filter(p => p.group === 'custom' && p.installed);
      for (const pack of customPacks) {
        try {
          const jsonStr = await window.__TAURI__.core.invoke('load_text_asset', { path: pack.id + '/character.json' });
          if (jsonStr) {
            const config = JSON.parse(jsonStr);
            GIF_MODES[pack.id] = config.states;
          }
        } catch(e) {}
      }
    } catch(e) {}
  }

  // If current mode is a DLC that's not installed, auto-download it
  const knownDlcs = ['mona', 'kuromi'];
  if (knownDlcs.includes(mode) && !dlcInstalledCache[mode] && window.__TAURI__) {
    statusText.textContent = 'Downloading ' + mode + '...';
    bubble.classList.remove('hidden');
    try {
      await window.__TAURI__.core.invoke('download_dlc', { dlcName: mode });
      dlcInstalledCache[mode] = true;
      // Re-check assets dir (download_dlc may have created it)
      try { hasExternalAssets = !!(await window.__TAURI__.core.invoke('get_assets_dir')); } catch(e) {}
      const jsonStr = await window.__TAURI__.core.invoke('load_text_asset', { path: mode + '/character.json' });
      if (jsonStr) {
        GIF_MODES[mode] = JSON.parse(jsonStr).states;
      } else {
        throw new Error('character.json not found after download');
      }
    } catch(e) {
      statusText.textContent = 'Download failed, using Ferris';
      mode = 'ferris';
      localStorage.setItem('petMode', mode);
    }
  }

  // If current mode is a DLC that's installed, preload it
  if (GIF_MODES[mode] && dlcInstalledCache[mode]) {
    await preloadAssets();
  }

  applyConfig();
  updateStatus({ state: 'idle', detail: '' });
})();
