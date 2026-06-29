const sharp = require('sharp')
const fs = require('fs')
const path = require('path')

const sizes = {
  '32x32.png': 32,
  '128x128.png': 128,
  '128x128@2x.png': 256,
  'icon.png': 512,
  'Square30x30Logo.png': 30,
  'Square44x44Logo.png': 44,
  'Square71x71Logo.png': 71,
  'Square89x89Logo.png': 89,
  'Square107x107Logo.png': 107,
  'Square142x142Logo.png': 142,
  'Square150x150Logo.png': 150,
  'Square284x284Logo.png': 284,
  'Square310x310Logo.png': 310,
  'StoreLogo.png': 50,
}

const iconsDir = process.argv[2] || 'src-tauri/icons'

async function main() {
  const svgPath = path.join(iconsDir, 'icon.svg')
  if (!fs.existsSync(svgPath)) {
    console.error('❌ icon.svg not found at', svgPath)
    process.exit(1)
  }
  const svgBuffer = fs.readFileSync(svgPath)
  for (const [filename, size] of Object.entries(sizes)) {
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(path.join(iconsDir, filename))
    console.log(`✅ ${filename} (${size}x${size})`)
  }
  console.log('🎉 Done')
}

main().catch(console.error)
