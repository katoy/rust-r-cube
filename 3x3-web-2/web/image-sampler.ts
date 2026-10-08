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

export interface ColorAdaptation {
  whiteSaturationThreshold?: number;
  darkValueThreshold?: number;
}

export function classifyColor(
  r: number,
  g: number,
  b: number,
  adaptation?: ColorAdaptation,
): string {
  const rf = r / 255;
  const gf = g / 255;
  const bf = b / 255;
  const max = Math.max(rf, gf, bf);
  const min = Math.min(rf, gf, bf);
  const d = max - min;
  const s = max === 0 ? 0 : d / max;
  const v = max;

  const darkV = adaptation?.darkValueThreshold ?? 0.18;
  const whiteS = adaptation?.whiteSaturationThreshold ?? 0.28;

  let h = 0;
  if (max !== min) {
    if (max === rf) {
      h = (gf - bf) / d + (gf < bf ? 6 : 0);
    } else if (max === gf) {
      h = (bf - rf) / d + 2;
    } else {
      h = (rf - gf) / d + 4;
    }
    h *= 60;
  }

  // 1. 極端に暗いピクセル（黒プラスチック目地、完全な暗闇など）
  if (v < darkV || (s < 0.25 && v < 0.45)) {
    return "?";
  }

  // 2. 白（低彩度かつ十分な明度、または暖色照明下での微弱な偏り）
  if (
    (s < whiteS && v >= 0.45) ||
    (s < 0.38 && v >= 0.85 && h >= 18 && h <= 55)
  ) {
    return "U";
  }

  // 3. 有彩色（色相 H: 0〜360 による判定）
  // 影や室内照明の偏りにより彩度・明度がやや低下した場合でも、色相特性に基づいて堅牢に分類
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

const COLOR_TO_INDEX: Record<string, number> = {
  U: 0,
  R: 1,
  F: 2,
  D: 3,
  L: 4,
  B: 5,
  "?": 6,
};
const INDEX_TO_COLOR = ["U", "R", "F", "D", "L", "B", "?"] as const;

export function classify(
  data: ImageData,
  x: number,
  y: number,
  radius = 5,
  adaptation?: ColorAdaptation,
): string {
  const counts = [0, 0, 0, 0, 0, 0, 0];
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
        adaptation,
      );
      const idx = COLOR_TO_INDEX[c] ?? 6;
      counts[idx]++;
      if (idx < 6) validColors++;
    }
  }

  // 有効な色があれば、黒ノイズ（ロゴ文字や目地）を除外して最多の色を採用
  if (validColors === 0) return "?";

  let best = "?";
  let maxCount = 0;
  for (let i = 0; i < 6; i++) {
    if (counts[i] > maxCount) {
      maxCount = counts[i];
      best = INDEX_TO_COLOR[i];
    }
  }
  return best;
}

export function getPerspectiveTransform(points: Point[]) {
  if (points.length !== 4) {
    throw new Error("面の四隅を4点指定してください。");
  }
  const [p0, p1, p2, p3] = points;

  // 1. 凸性・縮退の検証 (各頂点での外積の符号が同一かつ非ゼロ)
  const edges = [
    { x: p1.x - p0.x, y: p1.y - p0.y },
    { x: p2.x - p1.x, y: p2.y - p1.y },
    { x: p3.x - p2.x, y: p3.y - p2.y },
    { x: p0.x - p3.x, y: p0.y - p3.y },
  ];
  const cross = [
    edges[0].x * edges[1].y - edges[0].y * edges[1].x,
    edges[1].x * edges[2].y - edges[1].y * edges[2].x,
    edges[2].x * edges[3].y - edges[2].y * edges[3].x,
    edges[3].x * edges[0].y - edges[3].y * edges[0].x,
  ];

  const allPositive = cross.every((c) => c > 1e-5);
  const allNegative = cross.every((c) => c < -1e-5);
  if (!allPositive && !allNegative) {
    throw new Error("有効な四角形（単純な凸四角形）を指定してください。");
  }

  // 2. 単位正方形 [0, 1]x[0, 1] から p0, p1, p2, p3 へのホモグラフィ行列係数を計算
  const x0 = p0.x,
    y0 = p0.y;
  const x1 = p1.x,
    y1 = p1.y;
  const x2 = p2.x,
    y2 = p2.y;
  const x3 = p3.x,
    y3 = p3.y;

  const dx1 = x1 - x2;
  const dx2 = x3 - x2;
  const dx3 = x0 - x1 + x2 - x3;
  const dy1 = y1 - y2;
  const dy2 = y3 - y2;
  const dy3 = y0 - y1 + y2 - y3;

  let a: number,
    b: number,
    c: number,
    d: number,
    e: number,
    f: number,
    g: number,
    h: number;

  if (Math.abs(dx3) < 1e-7 && Math.abs(dy3) < 1e-7) {
    // アフィン変換（平行四辺形）
    a = x1 - x0;
    b = x3 - x0;
    c = x0;
    d = y1 - y0;
    e = y3 - y0;
    f = y0;
    g = 0;
    h = 0;
  } else {
    // 幾何学的証明: det (= dx1 * dy2 - dx2 * dy1) はベクトル (p1 - p2) と (p3 - p2) の外積であり、
    // ステップ1の cross[1] と恒等的に一致する (det === -cross[1])。
    // ステップ1の凸性検証によりすべての頂点で |cross[i]| > 1e-5 が保証されているため、
    // ここで |det| > 1e-5 が数学的に 100% 成立し、ゼロ除算（特異行列）は完全に排除されている。
    const det = dx1 * dy2 - dx2 * dy1;
    g = (dx3 * dy2 - dx2 * dy3) / det;
    h = (dx1 * dy3 - dx3 * dy1) / det;
    a = x1 - x0 + g * x1;
    b = x3 - x0 + h * x3;
    c = x0;
    d = y1 - y0 + g * y1;
    e = y3 - y0 + h * y3;
    f = y0;
  }

  return (u: number, v: number): Point => {
    const w = g * u + h * v + 1;
    if (w <= 1e-5) {
      throw new Error("射影変換の分母が不正です。");
    }
    return {
      x: (a * u + b * v + c) / w,
      y: (d * u + e * v + f) / w,
    };
  };
}

export function getImagePixels(image: HTMLImageElement): ImageData {
  const canvas = document.createElement("canvas");
  canvas.width = image.naturalWidth;
  canvas.height = image.naturalHeight;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("画像を読み込めませんでした。");
  context.drawImage(image, 0, 0);
  return context.getImageData(0, 0, canvas.width, canvas.height);
}

export function sampleFaceFromPixels(
  pixels: ImageData,
  points: Point[],
  adaptation?: ColorAdaptation,
  size: 2 | 3 = 3,
): string {
  if (points.length !== 4) throw new Error("面の四隅を4点指定してください。");
  const transform = getPerspectiveTransform(points);

  const [topLeft, topRight] = points;
  // 四角形の辺の長さから適切なサンプリング半径を動的決定
  const edgeLen = Math.hypot(topRight.x - topLeft.x, topRight.y - topLeft.y);
  const radius = Math.max(3, Math.min(25, Math.round(edgeLen / (size * 8))));

  let result = "";
  for (let row = 0; row < size; row++) {
    for (let col = 0; col < size; col++) {
      const u = (col + 0.5) / size;
      const v = (row + 0.5) / size;
      const pt = transform(u, v);
      result += classify(pixels, pt.x, pt.y, radius, adaptation);
    }
  }
  return result;
}

export function sampleFace(
  image: HTMLImageElement,
  points: Point[],
  adaptation?: ColorAdaptation,
  size: 2 | 3 = 3,
): string {
  if (points.length !== 4) throw new Error("面の四隅を4点指定してください。");
  return sampleFaceFromPixels(getImagePixels(image), points, adaptation, size);
}

export function buildState(
  faces: Partial<Record<(typeof FACES)[number], string>>,
  cubeType: "2x2" | "3x3" = "3x3",
) {
  if (cubeType === "2x2") {
    return [...FACES]
      .map((face) => {
        const raw = faces[face];
        if (!raw) return "????";
        return raw.slice(0, 4);
      })
      .join("");
  }
  return [...FACES]
    .map((face) => {
      const raw = faces[face];
      if (!raw) return "?????????";
      return raw.slice(0, 4) + face + raw.slice(5);
    })
    .join("");
}
