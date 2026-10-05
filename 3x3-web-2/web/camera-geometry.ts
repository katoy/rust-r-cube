export type Point = { x: number; y: number };

export function intersectLines(
  p1: Point,
  p2: Point,
  p3: Point,
  p4: Point,
): Point {
  const denom = (p4.y - p3.y) * (p2.x - p1.x) - (p4.x - p3.x) * (p2.y - p1.y);
  if (Math.abs(denom) < 1e-6) {
    return {
      x: (p1.x + p2.x + p3.x + p4.x) / 4,
      y: (p1.y + p2.y + p3.y + p4.y) / 4,
    };
  }
  const ua =
    ((p4.x - p3.x) * (p1.y - p3.y) - (p4.y - p3.y) * (p1.x - p3.x)) / denom;
  return {
    x: p1.x + ua * (p2.x - p1.x),
    y: p1.y + ua * (p2.y - p1.y),
  };
}

export function computeCenter(points: Point[]): Point {
  const [p1, p2, p3, p4, p5, p6] = points;
  const c1 = intersectLines(p1, p4, p2, p5);
  const c2 = intersectLines(p1, p4, p3, p6);
  const c3 = intersectLines(p2, p5, p3, p6);
  return {
    x: (c1.x + c2.x + c3.x) / 3,
    y: (c1.y + c2.y + c3.y) / 3,
  };
}

/**
 * 6つの外周頂点を重心まわりの時計回り角度（最上部をP1として0°）で正規化する。
 * 反時計回り入力や順序違いの再追加（M8, M9）を吸収する。
 */
export function normalizeOutlinePoints(points: Point[]): Point[] {
  if (points.length !== 6) return points;
  const cx = points.reduce((sum, p) => sum + p.x, 0) / 6;
  const cy = points.reduce((sum, p) => sum + p.y, 0) / 6;

  // 重心周りの偏角でソート（画面座標系で時計回りに並べる）
  const sorted = [...points].sort((a, b) => {
    return Math.atan2(a.y - cy, a.x - cx) - Math.atan2(b.y - cy, b.x - cx);
  });

  // 最も上にある点（y が最小の点）を P1（てっぺん）として先頭にシフト
  let topIdx = 0;
  for (let i = 1; i < sorted.length; i++) {
    if (sorted[i].y < sorted[topIdx].y) {
      topIdx = i;
    }
  }

  return [...sorted.slice(topIdx), ...sorted.slice(0, topIdx)];
}

export function detectCubeOutline(
  canvas: HTMLCanvasElement,
  _image?: HTMLImageElement,
): Point[] {
  const w = canvas.width;
  const h = canvas.height;
  const cx = w / 2;
  const cy = h / 2;

  // 典型的なアイソメトリック比率（高さの1/3が各軸長）
  const L = Math.round(Math.min(w, h) / 3);
  const cos30 = Math.cos(Math.PI / 6);
  const sin30 = Math.sin(Math.PI / 6);

  const defaultPoints: Point[] = [
    { x: Math.round(cx), y: Math.round(cy - L) }, // P1: てっぺん
    { x: Math.round(cx + L * cos30), y: Math.round(cy - L * sin30) }, // P2: 右上
    { x: Math.round(cx + L * cos30), y: Math.round(cy + L * sin30) }, // P3: 右下
    { x: Math.round(cx), y: Math.round(cy + L) }, // P4: 底
    { x: Math.round(cx - L * cos30), y: Math.round(cy + L * sin30) }, // P5: 左下
    { x: Math.round(cx - L * cos30), y: Math.round(cy - L * sin30) }, // P6: 左上
  ];

  try {
    const ctx = canvas.getContext("2d");
    if (!ctx) return defaultPoints;
    const imgData = ctx.getImageData(0, 0, w, h);
    const data = imgData.data;

    const dirAngles = [
      -Math.PI / 2,
      -Math.PI / 6,
      Math.PI / 6,
      Math.PI / 2,
      (5 * Math.PI) / 6,
      (-5 * Math.PI) / 6,
    ];

    const detected: Point[] = [];
    for (let i = 0; i < 6; i++) {
      const ang = dirAngles[i];
      const cosA = Math.cos(ang);
      const sinA = Math.sin(ang);

      let maxGrad = 0;
      let bestR = L;
      // ステッカー内部境界（約0.67L）を誤認しないよう、外周周辺（0.88L〜1.12L）に絞って走査
      const minR = Math.round(L * 0.88);
      const maxR = Math.round(Math.min(L * 1.12, Math.min(w, h) * 0.49));

      let prevLum = -1;
      for (let r = minR; r <= maxR; r += 1) {
        const px = Math.round(cx + r * cosA);
        const py = Math.round(cy + r * sinA);
        if (px < 1 || px >= w - 1 || py < 1 || py >= h - 1) break;
        const idx = (py * w + px) * 4;
        const lum =
          0.299 * data[idx] + 0.587 * data[idx + 1] + 0.114 * data[idx + 2];
        if (prevLum >= 0) {
          const grad = Math.abs(lum - prevLum);
          if (grad > maxGrad && grad > 25) {
            maxGrad = grad;
            bestR = r;
          }
        }
        prevLum = lum;
      }

      detected.push({
        x: Math.round(cx + bestR * cosA),
        y: Math.round(cy + bestR * sinA),
      });
    }
    return detected;
  } catch {
    return defaultPoints;
  }
}
