import type { Point } from "./camera-geometry";

/**
 * HTMLCanvasElement 上の CSS 表示座標（clientX, clientY）を
 * 内部解像度（canvas.width, canvas.height）に合わせた座標に変換します。
 * object-fit: contain やレターボックス余白（黒帯）を考慮します。
 */
export function toCanvasCoords(
  canvas: HTMLCanvasElement,
  clientX: number,
  clientY: number,
  rect?: DOMRect,
): Point {
  const r = rect ?? canvas.getBoundingClientRect();
  if (r.width <= 0 || r.height <= 0) return { x: 0, y: 0 };

  const elemRatio = r.width / r.height;
  const canvasRatio = canvas.width / canvas.height;

  let drawWidth = r.width;
  let drawHeight = r.height;
  let drawLeft = r.left;
  let drawTop = r.top;

  if (elemRatio > canvasRatio) {
    // 横長：左右にレターボックス余白がある場合
    drawWidth = r.height * canvasRatio;
    drawLeft = r.left + (r.width - drawWidth) / 2;
  } else {
    // 縦長：上下にレターボックス余白がある場合
    drawHeight = r.width / canvasRatio;
    drawTop = r.top + (r.height - drawHeight) / 2;
  }

  const scaleX = canvas.width / drawWidth;
  const scaleY = canvas.height / drawHeight;

  return {
    x: Math.max(0, Math.min(canvas.width, (clientX - drawLeft) * scaleX)),
    y: Math.max(0, Math.min(canvas.height, (clientY - drawTop) * scaleY)),
  };
}

/**
 * クリック／ホバーされた座標が、6頂点または中心ハンドルのヒット半径内にあるかを判定します。
 * @returns 0..5: points[i], 6: centerPoint, -1: ヒットなし
 */
export function findHitTarget(
  canvas: HTMLCanvasElement,
  points: Point[],
  centerPoint: Point | undefined,
  x: number,
  y: number,
  computeCenterFn: (pts: Point[]) => Point,
  rect?: DOMRect,
): number {
  const r = rect ?? canvas.getBoundingClientRect();
  const scaleX = canvas.width / Math.max(1, r.width);
  // 画面上で最低30px相当の当たり判定領域を確保
  const hitRadius = Math.max(30, 30 * scaleX);

  if (points.length === 6) {
    const center = centerPoint ?? computeCenterFn(points);
    if (Math.hypot(center.x - x, center.y - y) <= hitRadius) {
      return 6;
    }
  }
  return points.findIndex((p) => Math.hypot(p.x - x, p.y - y) <= hitRadius);
}

/**
 * 枠の頂点を指定ステップ数（時計回り60°ごと）だけローテーションします。
 */
export function rotatePointsArray(points: Point[], step = 1): Point[] {
  if (points.length !== 6) return [...points];
  const s = ((step % 6) + 6) % 6;
  return [...points.slice(6 - s), ...points.slice(0, 6 - s)];
}
