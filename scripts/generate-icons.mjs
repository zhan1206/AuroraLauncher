/**
 * Generate Tauri app icons from the SVG source.
 * Produces: 32x32.png, 128x128.png, 128x128@2x.png (256x256), icon.ico, icon.icns
 */
import sharp from "sharp";
import { readFileSync, mkdirSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = resolve(__dirname, "..");
const ICONS_DIR = resolve(PROJECT_ROOT, "src-tauri", "icons");
const SVG_PATH = resolve(ICONS_DIR, "icon.svg");

if (!existsSync(SVG_PATH)) {
  console.error("icon.svg not found at", SVG_PATH);
  process.exit(1);
}

mkdirSync(ICONS_DIR, { recursive: true });

const svgBuffer = readFileSync(SVG_PATH);

async function generatePng(size, filename) {
  await sharp(svgBuffer)
    .resize(size, size)
    .png()
    .toFile(resolve(ICONS_DIR, filename));
  console.log(`✓ ${filename} (${size}x${size})`);
}

async function generateIco() {
  // ICO file: 16x16 + 32x32 + 48x48 + 256x256
  const sizes = [16, 32, 48, 256];
  const pngBuffers = [];

  for (const size of sizes) {
    const buf = await sharp(svgBuffer).resize(size, size).png().toBuffer();
    pngBuffers.push({ size, buf });
  }

  // ICO header: 6 bytes
  const numImages = pngBuffers.length;
  const headerSize = 6;
  const dirEntrySize = 16;
  const dirSize = dirEntrySize * numImages;

  // Calculate offsets for each image data
  let currentOffset = headerSize + dirSize;
  const entries = pngBuffers.map(({ size, buf }) => {
    const entry = {
      width: size >= 256 ? 0 : size,
      height: size >= 256 ? 0 : size,
      colorCount: 0,
      planes: 1,
      bitCount: 32,
      size: buf.length,
      offset: currentOffset,
    };
    currentOffset += buf.length;
    return entry;
  });

  const totalSize = currentOffset;
  const ico = Buffer.alloc(totalSize);
  let pos = 0;

  // ICO header
  ico.writeUInt16LE(0, pos); pos += 2; // reserved
  ico.writeUInt16LE(1, pos); pos += 2; // type: ICO
  ico.writeUInt16LE(numImages, pos); pos += 2; // number of images

  // Directory entries
  for (const e of entries) {
    ico.writeUInt8(e.width, pos); pos += 1;
    ico.writeUInt8(e.height, pos); pos += 1;
    ico.writeUInt8(e.colorCount, pos); pos += 1;
    ico.writeUInt8(0, pos); pos += 1; // reserved
    ico.writeUInt16LE(e.planes, pos); pos += 2;
    ico.writeUInt16LE(e.bitCount, pos); pos += 2;
    ico.writeUInt32LE(e.size, pos); pos += 4;
    ico.writeUInt32LE(e.offset, pos); pos += 4;
  }

  // Image data
  for (const { buf } of pngBuffers) {
    buf.copy(ico, pos);
    pos += buf.length;
  }

  const { writeFileSync } = await import("fs");
  writeFileSync(resolve(ICONS_DIR, "icon.ico"), ico);
  console.log(`✓ icon.ico (${numImages} images)`);
}

async function generateIcns() {
  // ICNS format: simple container with PNG data
  // We'll use 256x256 PNG as the icon data
  const png256 = await sharp(svgBuffer).resize(256, 256).png().toBuffer();

  // ic07 = 256x256 PNG
  const type = "ic07";
  const dataSize = png256.length;
  const entrySize = 4 + 4 + dataSize; // type(4) + size(4) + data

  const totalSize = 4 + 4 + entrySize; // magic(4) + total_size(4) + entry
  const icns = Buffer.alloc(totalSize);
  let pos = 0;

  // File header
  icns.write("icns", pos); pos += 4;
  icns.writeUInt32BE(totalSize, pos); pos += 4;

  // ic07 entry
  icns.write(type, pos); pos += 4;
  icns.writeUInt32BE(entrySize, pos); pos += 4;
  png256.copy(icns, pos);

  const { writeFileSync } = await import("fs");
  writeFileSync(resolve(ICONS_DIR, "icon.icns"), icns);
  console.log(`✓ icon.icns (256x256)`);
}

async function main() {
  console.log("Generating Tauri app icons from SVG...\n");

  await generatePng(32, "32x32.png");
  await generatePng(128, "128x128.png");
  await generatePng(256, "128x128@2x.png");
  await generateIco();
  await generateIcns();

  console.log("\n✅ All icons generated successfully!");
}

main().catch(console.error);
