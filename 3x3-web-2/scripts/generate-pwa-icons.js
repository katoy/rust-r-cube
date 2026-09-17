import { createCanvas } from "canvas";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const publicDir = path.resolve(__dirname, "../public");

function drawCube(ctx, size) {
  ctx.fillStyle = "#141716";
  const r = size * 0.1875;
  // Rounded rect
  ctx.beginPath();
  ctx.moveTo(r, 0);
  ctx.lineTo(size - r, 0);
  ctx.quadraticCurveTo(size, 0, size, r);
  ctx.lineTo(size, size - r);
  ctx.quadraticCurveTo(size, size, size - r, size);
  ctx.lineTo(r, size);
  ctx.quadraticCurveTo(0, size, 0, size - r);
  ctx.lineTo(0, r);
  ctx.quadraticCurveTo(0, 0, r, 0);
  ctx.closePath();
  ctx.fill();

  ctx.save();
  ctx.translate(size / 2, size / 2);
  const s = (size / 512) * 11;
  ctx.scale(s, s);

  ctx.strokeStyle = "#c4ed94";
  ctx.lineWidth = 1.8;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";

  // Outer isometric hex
  ctx.beginPath();
  ctx.moveTo(0, -16);
  ctx.lineTo(14, -8);
  ctx.lineTo(14, 8);
  ctx.lineTo(0, 16);
  ctx.lineTo(-14, 8);
  ctx.lineTo(-14, -8);
  ctx.closePath();
  ctx.stroke();

  // Y-axis center lines
  ctx.beginPath();
  ctx.moveTo(-14, -8);
  ctx.lineTo(0, 0);
  ctx.lineTo(14, -8);
  ctx.moveTo(0, 0);
  ctx.lineTo(0, 16);
  ctx.stroke();

  // Internal grid lines
  ctx.beginPath();
  ctx.moveTo(-9.33, -5.33);
  ctx.lineTo(4.67, -13.33);
  ctx.moveTo(-4.67, -2.67);
  ctx.lineTo(9.33, -10.67);

  ctx.moveTo(9.33, -5.33);
  ctx.lineTo(-4.67, -13.33);
  ctx.moveTo(4.67, -2.67);
  ctx.lineTo(-9.33, -10.67);

  ctx.moveTo(4.67, 2.67);
  ctx.lineTo(4.67, 18.67);
  ctx.moveTo(9.33, 5.33);
  ctx.lineTo(9.33, 21.33);

  ctx.moveTo(0, 5.33);
  ctx.lineTo(14, -2.67);
  ctx.moveTo(0, 10.67);
  ctx.lineTo(14, 2.67);

  ctx.moveTo(-4.67, 2.67);
  ctx.lineTo(-4.67, 18.67);
  ctx.moveTo(-9.33, 5.33);
  ctx.lineTo(-9.33, 21.33);

  ctx.moveTo(0, 5.33);
  ctx.lineTo(-14, -2.67);
  ctx.moveTo(0, 10.67);
  ctx.lineTo(-14, 2.67);
  ctx.stroke();

  ctx.restore();
}

function generate(size, filename) {
  const canvas = createCanvas(size, size);
  const ctx = canvas.getContext("2d");
  drawCube(ctx, size);
  const buffer = canvas.toBuffer("image/png");
  const dest = path.join(publicDir, filename);
  fs.writeFileSync(dest, buffer);
  console.log(`Generated ${dest} (${size}x${size})`);
}

generate(192, "icon-192.png");
generate(512, "icon-512.png");
