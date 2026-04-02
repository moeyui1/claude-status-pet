#!/usr/bin/env node
// Download character GIFs from GIPHY at runtime
// These are NOT bundled in the repo due to licensing
// Cross-platform: works on Windows, macOS, Linux without bash

const fs = require('fs');
const path = require('path');
const os = require('os');
const https = require('https');

const ASSETS_DIR = process.argv[2] || path.join(
  process.env.CLAUDE_PLUGIN_DATA || path.join(os.homedir(), '.claude', 'pet-data'),
  'assets'
);
const VERSION_FILE = path.join(ASSETS_DIR, '.gifs-version');
const CURRENT_VERSION = '2';

fs.mkdirSync(path.join(ASSETS_DIR, 'mona'), { recursive: true });
fs.mkdirSync(path.join(ASSETS_DIR, 'kuromi'), { recursive: true });

// Skip if already downloaded this version
if (fs.existsSync(VERSION_FILE) && fs.readFileSync(VERSION_FILE, 'utf8').trim() === CURRENT_VERSION) {
  process.exit(0);
}

console.log('Downloading character GIFs from GIPHY...');

const GIFS = {
  // Mona (GitHub mascot) — from official GitHub GIPHY channel
  'mona/love.gif':      'https://media.giphy.com/media/jrdgDVFrcgJpNlonWO/giphy.gif',
  'mona/angry.gif':     'https://media.giphy.com/media/kmCCrDo2vlIu6Kswop/giphy.gif',
  'mona/looking.gif':   'https://media.giphy.com/media/9f8mk4P3X2Nvch1z2o/giphy.gif',
  'mona/mona.gif':      'https://media.giphy.com/media/OFEabGCcVqsckIGn8G/giphy.gif',
  'mona/tongue.gif':    'https://media.giphy.com/media/WcYnTzdrjQphdu33xs/giphy.gif',
  'mona/shocked.gif':   'https://media.giphy.com/media/JdQFsdoJBcHaPOANdK/giphy.gif',
  'mona/smirk.gif':     'https://media.giphy.com/media/0vTOscboHgOyBSuK4r/giphy.gif',
  'mona/laugh.gif':     'https://media.giphy.com/media/RgutegYIHk2Nhxj4m5/giphy.gif',
  'mona/ohbrother.gif': 'https://media.giphy.com/media/pMzEfC42AYlqT2WPaf/giphy.gif',
  'mona/hearts.gif':    'https://media.giphy.com/media/wJBYx2Yh84XS4sTzmz/giphy.gif',
  'mona/sick.gif':      'https://media.giphy.com/media/nfL2nlWacI8d9jgVXb/giphy.gif',
  'mona/tech.gif':      'https://media.giphy.com/media/cDZJ17fbzWVle68VCB/giphy.gif',
  'mona/ducks.gif':     'https://media.giphy.com/media/QxT6pLq6ekKiCkLkf0/giphy.gif',
  // Kuromi (Sanrio) — from official Sanrio Korea GIPHY channel
  'kuromi/bling.gif':    'https://media.giphy.com/media/JNxq0xOWfidCDzqUH3/giphy.gif',
  'kuromi/charming.gif': 'https://media.giphy.com/media/gkLG3Ki3OTXDwVb4rY/giphy.gif',
  'kuromi/kuromi.gif':   'https://media.giphy.com/media/MphoCSnXeA6wR4L8IS/giphy.gif',
  'kuromi/lilrya.gif':   'https://media.giphy.com/media/4G0nkrrXm8Xe1VgPzF/giphy.gif',
  'kuromi/jump.gif':     'https://media.giphy.com/media/cQSjIBgUC2NbMKEm9q/giphy.gif',
  'kuromi/sleeping.gif': 'https://media.giphy.com/media/ZIskbLAG8Qeiq2dbV5/giphy.gif',
  'kuromi/heart.gif':    'https://media.giphy.com/media/VpCEcS3ZJ4qlKcD8LF/giphy.gif',
  'kuromi/think.gif':    'https://media.giphy.com/media/dCRVRbdbZUlNt1sRPd/giphy.gif',
  'kuromi/angry.gif':    'https://media.giphy.com/media/Qtvvgwbl1svKYcUGIT/giphy.gif',
};

function download(url, dest) {
  return new Promise((resolve) => {
    const file = fs.createWriteStream(dest);
    const request = (u) => {
      https.get(u, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          request(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          file.close();
          try { fs.unlinkSync(dest); } catch(e) {}
          resolve(false);
          return;
        }
        res.pipe(file);
        file.on('finish', () => { file.close(); resolve(true); });
      }).on('error', () => {
        file.close();
        try { fs.unlinkSync(dest); } catch(e) {}
        resolve(false);
      });
    };
    request(url);
  });
}

async function main() {
  const results = await Promise.all(
    Object.entries(GIFS).map(([name, url]) =>
      download(url, path.join(ASSETS_DIR, name)).then(ok => ({ name, ok }))
    )
  );

  const failed = results.filter(r => !r.ok);
  if (failed.length > 0) {
    for (const f of failed) {
      console.error(`WARNING: Failed to download ${f.name}`);
    }
    console.error(`WARNING: ${failed.length} GIF(s) failed to download.`);
    process.exit(1);
  } else {
    fs.writeFileSync(VERSION_FILE, CURRENT_VERSION);
    console.log(`GIFs downloaded to ${ASSETS_DIR}`);
  }
}

main();
