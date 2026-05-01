#!/usr/bin/env node
// Generates tray icon PNGs from Lucide SVG paths.
// Run: node scripts/gen-tray-icons.mjs
// Output: src-tauri/icons/tray-{running,paused,warning}.png  (22x22 + 44x44 @2x)

import sharp from 'sharp';
import { writeFileSync, mkdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const iconsDir = resolve(__dirname, '../src-tauri/icons');
mkdirSync(iconsDir, { recursive: true });

// Lucide SVGs — black stroke on transparent background (macOS template-image style)
const icons = {
  'tray-running': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
    <circle cx="12" cy="12" r="3"/>
  </svg>`,

  'tray-paused': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="10"/>
    <line x1="10" x2="10" y1="15" y2="9"/>
    <line x1="14" x2="14" y1="15" y2="9"/>
  </svg>`,

  'tray-warning': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="10"/>
    <line x1="12" x2="12" y1="8" y2="12"/>
    <line x1="12" x2="12.01" y1="16" y2="16"/>
  </svg>`,
};

for (const [name, svg] of Object.entries(icons)) {
  const buf = Buffer.from(svg);

  // 1x — 22×22
  const out1x = resolve(iconsDir, `${name}.png`);
  await sharp(buf).resize(22, 22).png().toFile(out1x);
  console.log(`wrote ${out1x}`);

  // 2x — 44×44
  const out2x = resolve(iconsDir, `${name}@2x.png`);
  await sharp(buf).resize(44, 44).png().toFile(out2x);
  console.log(`wrote ${out2x}`);
}

console.log('done');
