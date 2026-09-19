import { FACES } from "./model";

type Point = { x: number; y: number };

export function rgbToHsv(r: number, g: number, b: number) {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;
  let h = 0;
  const s = max === 0 ? 0 : d / max;
  const v = max;

  if (max !== min) {
    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h *= 60;
  }
  return { h, s, v };
}

export function classifyColor(r: number, g: number, b: number): string {
  const { h, s, v } = rgbToHsv(r, g, b);

  // 1. 極端に暗いピクセル（黒プラスチック目地、影など）
  if (v < 0.2 || (s < 0.3 && v < 0.45)) {
    return "?";
  }

  // 2. 白（低彩度かつ十分な明度）
  if (s < 0.3 && v >= 0.45) {
    return "U";
  }

  // 3. 有彩色（色相 H: 0〜360 による判定）
  if (h >= 75 && h <= 170) {
    return "F"; // 緑
  }
  if (h > 170 && h <= 265) {
    return "B"; // 青
  }
  if (h >= 40 && h < 75) {
    return "D"; // 黄
  }
  if (h >= 18 && h < 40) {
    return "L"; // 橙
  }
  if (h < 18 || h >= 340) {
    return "R"; // 赤
  }

  return "?";
}

export function classify(
  data: ImageData,
  x: number,
  y: number,
  radius = 5,
): string {
  const counts: Record<string, number> = {
    U: 0,
    R: 0,
    F: 0,
    D: 0,
    L: 0,
    B: 0,
    "?": 0,
  };
  let validColors = 0;

  for (let dy = -radius; dy <= radius; dy++) {
    for (let dx = -radius; dx <= radius; dx++) {
      const px = Math.max(0, Math.min(data.width - 1, Math.round(x + dx)));
      const py = Math.max(0, Math.min(data.height - 1, Math.round(y + dy)));
      const offset = (py * data.width + px) * 4;
      const c = classifyColor(
        data.data[offset],
        data.data[offset + 1],
        data.data[offset + 2],
      );
      counts[c]++;
      if (c !== "?") validColors++;
    }
  }

  // 有効な色があれば、黒ノイズ（ロゴ文字や目地）を除外して最多の色を採用
  let best = "?";
  let maxCount = 0;
  const targetList =
    validColors > 0
      ? (["U", "R", "F", "D", "L", "B"] as const)
      : (["?"] as const);
  for (const c of targetList) {
    if (counts[c] > maxCount) {
      maxCount = counts[c];
      best = c;
    }
  }
  return best;
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

  // 四角形の辺の長さから適切なサンプリング半径を動的決定
  const edgeLen = Math.hypot(topRight.x - topLeft.x, topRight.y - topLeft.y);
  const radius = Math.max(3, Math.min(25, Math.round(edgeLen / 25)));

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
        radius,
      );
    }
  }
  return result;
}

export function buildState(
  faces: Partial<Record<(typeof FACES)[number], string>>,
) {
  return [...FACES]
    .map((face) => {
      const raw = faces[face];
      if (!raw) return "?????????";
      if (raw[4] === "?") {
        return raw.slice(0, 4) + face + raw.slice(5);
      }
      return raw;
    })
    .join("");
}
