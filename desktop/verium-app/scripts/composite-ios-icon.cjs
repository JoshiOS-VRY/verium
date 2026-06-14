#!/usr/bin/env node
/**
 * Composite the 3D logo onto the premium gradient background for iOS icons.
 * Env: LOGO, BG_PNG, OUT, LOGO_SIZE, PAD
 */
const sharp = require('sharp');

const logoPath = process.env.LOGO;
const bgPath = process.env.BG_PNG;
const outPath = process.env.OUT;
const logoSize = Number(process.env.LOGO_SIZE);
const pad = Number(process.env.PAD);

if (!logoPath || !bgPath || !outPath || !logoSize || Number.isNaN(pad)) {
  console.error('error: LOGO, BG_PNG, OUT, LOGO_SIZE, and PAD env vars are required');
  process.exit(1);
}

async function keyLogo(input, size) {
  const { data, info } = await sharp(input)
    .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });

  for (let i = 0; i < data.length; i += 4) {
    const r = data[i];
    const g = data[i + 1];
    const b = data[i + 2];
    const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if (lum < 16) {
      data[i + 3] = 0;
    } else if (lum < 40) {
      data[i + 3] = Math.round(((lum - 16) / 24) * 255);
    }
  }

  return sharp(data, {
    raw: { width: info.width, height: info.height, channels: 4 },
  })
    .png()
    .toBuffer();
}

(async () => {
  const keyed = await keyLogo(logoPath, logoSize);
  const bloom = await sharp(keyed)
    .blur(16)
    .modulate({ brightness: 1.15, saturation: 1.1 })
    .toBuffer();

  await sharp(bgPath)
    .composite([
      { input: bloom, left: pad, top: pad, blend: 'screen' },
      { input: keyed, left: pad, top: pad },
    ])
    .flatten({ background: '#020617' })
    .png()
    .toFile(outPath);
})();
