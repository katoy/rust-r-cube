import { COLORS, FACES } from "./model";

type Point = { x: number; y: number };
const labels = ["U", "R", "F", "D", "L", "B"] as const;
const rgb = (hex: string) => {
  const value = Number.parseInt(hex.slice(1), 16);
  return [value >> 16, (value >> 8) & 255, value & 255];
};
const references = labels.map((face) => rgb(COLORS[face]));

function classify(data: ImageData, x: number, y: number) {
  const radius = 3;
  let best = 0;
  let bestDistance = Number.POSITIVE_INFINITY;
  for (let face = 0; face < references.length; face++) {
    let distance = 0;
    for (let dy = -radius; dy <= radius; dy++) {
      for (let dx = -radius; dx <= radius; dx++) {
        const px = Math.max(0, Math.min(data.width - 1, Math.round(x + dx)));
        const py = Math.max(0, Math.min(data.height - 1, Math.round(y + dy)));
        const offset = (py * data.width + px) * 4;
        const ref = references[face];
        distance += (data.data[offset] - ref[0]) ** 2;
        distance += (data.data[offset + 1] - ref[1]) ** 2;
        distance += (data.data[offset + 2] - ref[2]) ** 2;
      }
    }
    if (distance < bestDistance) {
      bestDistance = distance;
      best = face;
    }
  }
  return labels[best];
}

export function sampleFace(image: HTMLImageElement, points: Point[]): string {
  if (points.length !== 4) throw new Error("面の四隅を4点指定してください。");
  const canvas = document.createElement("canvas");
  canvas.width = image.naturalWidth;
  canvas.height = image.naturalHeight;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("画像を読み込めませんでした。");
  context.drawImage(image, 0, 0);
  const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
  const [topLeft, topRight, bottomRight, bottomLeft] = points;
  let result = "";
  for (let row = 0; row < 3; row++) {
    for (let col = 0; col < 3; col++) {
      const u = (col + 0.5) / 3;
      const v = (row + 0.5) / 3;
      const top = {
        x: topLeft.x + (topRight.x - topLeft.x) * u,
        y: topLeft.y + (topRight.y - topLeft.y) * u,
      };
      const bottom = {
        x: bottomLeft.x + (bottomRight.x - bottomLeft.x) * u,
        y: bottomLeft.y + (bottomRight.y - bottomLeft.y) * u,
      };
      result += classify(
        pixels,
        top.x + (bottom.x - top.x) * v,
        top.y + (bottom.y - top.y) * v,
      );
    }
  }
  return result;
}

export function buildState(
  faces: Partial<Record<(typeof FACES)[number], string>>,
) {
  return [...FACES].map((face) => faces[face] ?? "?????????").join("");
}
