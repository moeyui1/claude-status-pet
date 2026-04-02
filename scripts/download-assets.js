#!/usr/bin/env node
// Download pet assets (character images) from GitHub Releases
// Cross-platform: works on Windows, macOS, Linux without bash/curl/gh

const fs = require('fs');
const path = require('path');
const os = require('os');
const https = require('https');
const { execSync } = require('child_process');

const ASSETS_DIR = process.argv[2] || path.join(
  process.env.CLAUDE_PLUGIN_DATA || path.join(os.homedir(), '.claude', 'pet-data'),
  'assets'
);
const VERSION_FILE = path.join(ASSETS_DIR, '.version');
const REPO = 'moeyui1/claude-status-pet';

fs.mkdirSync(ASSETS_DIR, { recursive: true });

function httpsGet(url) {
  return new Promise((resolve, reject) => {
    const request = (u) => {
      https.get(u, { headers: { 'User-Agent': 'claude-status-pet' } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          request(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`HTTP ${res.statusCode} for ${u}`));
          return;
        }
        const chunks = [];
        res.on('data', (c) => chunks.push(c));
        res.on('end', () => resolve(Buffer.concat(chunks)));
      }).on('error', reject);
    };
    request(url);
  });
}

async function main() {
  // Get latest release tag
  const apiData = await httpsGet(`https://api.github.com/repos/${REPO}/releases/latest`);
  const release = JSON.parse(apiData.toString());
  const latest = release.tag_name;
  if (!latest) {
    console.error('ERROR: Failed to fetch latest release tag');
    process.exit(1);
  }

  // Check if already downloaded this version
  if (fs.existsSync(VERSION_FILE) && fs.readFileSync(VERSION_FILE, 'utf8').trim() === latest) {
    process.exit(0);
  }

  console.log(`Downloading pet assets ${latest}...`);

  // Download pet-assets.zip
  const zipUrl = `https://github.com/${REPO}/releases/download/${latest}/pet-assets.zip`;
  const zipData = await httpsGet(zipUrl);
  const tmpZip = path.join(os.tmpdir(), 'pet-assets.zip');
  fs.writeFileSync(tmpZip, zipData);

  // Extract — use platform-appropriate method
  const isWin = os.platform() === 'win32';
  try {
    if (isWin) {
      execSync(`powershell -Command "Expand-Archive -Path '${tmpZip}' -DestinationPath '${ASSETS_DIR}' -Force"`, { stdio: 'pipe' });
    } else {
      execSync(`unzip -o "${tmpZip}" -d "${ASSETS_DIR}"`, { stdio: 'pipe' });
    }
  } catch(e) {
    console.error('ERROR: Failed to extract pet-assets.zip');
    process.exit(1);
  } finally {
    try { fs.unlinkSync(tmpZip); } catch(e) {}
  }

  fs.writeFileSync(VERSION_FILE, latest);
  console.log(`Assets ${latest} installed to ${ASSETS_DIR}`);
}

main().catch((e) => {
  console.error(`ERROR: ${e.message}`);
  process.exit(1);
});
