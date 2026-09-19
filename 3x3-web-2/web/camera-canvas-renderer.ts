import type { Point } from "./camera-geometry";

export interface CanvasOverlayOptions {
  activeImage?: HTMLImageElement;
  points: Point[];
  centerPoint?: Point;
  hoverIndex: number;
  draggingIndex: number;
  computeCenterFn: (pts: Point[]) => Point;
}

/**
 * キャンバス上に背景画像、キューブ外周枠、中心点、Y字境界線、各頂点ハンドルを描画します。
 */
export function renderCanvasOverlay(
  canvas: HTMLCanvasElement,
  options: CanvasOverlayOptions,
): void {
  const context = canvas.getContext("2d");
  if (!context) return;

  const {
    activeImage,
    points,
    centerPoint,
    hoverIndex,
    draggingIndex,
    computeCenterFn,
  } = options;

  context.clearRect(0, 0, canvas.width, canvas.height);
  if (activeImage) {
    context.drawImage(activeImage, 0, 0, canvas.width, canvas.height);
  }

  context.lineWidth = 3;
  context.font = "bold 16px sans-serif";

  // 2点以上の場合は外周線を引く
  if (points.length > 1) {
    context.strokeStyle = "#c4ed94";
    context.beginPath();
    context.moveTo(points[0].x, points[0].y);
    for (let i = 1; i < points.length; i++) {
      context.lineTo(points[i].x, points[i].y);
    }
    if (points.length === 6) {
      context.closePath();
    }
    context.stroke();
  }

  // 6点揃ったら、中央点と各面の境界線を描画
  if (points.length === 6) {
    const center = centerPoint ?? computeCenterFn(points);
    const isCenterHovered = hoverIndex === 6 || draggingIndex === 6;

    // 境界Y字線 (center -> P2, center -> P4, center -> P6)
    context.strokeStyle = "#facc15";
    context.lineWidth = 2;
    context.beginPath();
    context.moveTo(center.x, center.y);
    context.lineTo(points[1].x, points[1].y); // P2 (右上)
    context.moveTo(center.x, center.y);
    context.lineTo(points[3].x, points[3].y); // P4 (底)
    context.moveTo(center.x, center.y);
    context.lineTo(points[5].x, points[5].y); // P6 (左上)
    context.stroke();

    // 中央点ハンドル
    context.beginPath();
    context.arc(center.x, center.y, isCenterHovered ? 12 : 9, 0, Math.PI * 2);
    context.fillStyle = isCenterHovered
      ? "rgba(250, 204, 21, 0.4)"
      : "rgba(250, 204, 21, 0.25)";
    context.fill();

    context.beginPath();
    context.arc(center.x, center.y, isCenterHovered ? 7 : 5, 0, Math.PI * 2);
    context.fillStyle = "#facc15";
    context.fill();
    context.strokeStyle = "#1e261e";
    context.lineWidth = 2;
    context.stroke();

    context.fillStyle = "#ffffff";
    context.fillText("中心", center.x + 10, center.y + 5);
  }

  // 各頂点の描画（ドラッグハンドル）
  points.forEach((point, index) => {
    const isHovered = hoverIndex === index || draggingIndex === index;

    // 外側リング
    context.beginPath();
    context.arc(point.x, point.y, isHovered ? 12 : 9, 0, Math.PI * 2);
    context.fillStyle = isHovered
      ? "rgba(250, 204, 21, 0.4)"
      : "rgba(196, 237, 148, 0.3)";
    context.fill();

    // 内側サークル
    context.beginPath();
    context.arc(point.x, point.y, isHovered ? 7 : 5, 0, Math.PI * 2);
    context.fillStyle = isHovered ? "#facc15" : "#c4ed94";
    context.fill();
    context.strokeStyle = "#1e261e";
    context.lineWidth = 2;
    context.stroke();

    // 番号
    context.fillStyle = "#ffffff";
    context.fillText(String(index + 1), point.x + 12, point.y - 8);
  });
}
