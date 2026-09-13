import { buildState, sampleFace } from "./image-sampler";
import { FACES } from "./model";

type Point = { x: number; y: number };
type Apply = (state: string) => void;

function intersectLines(p1: Point, p2: Point, p3: Point, p4: Point): Point {
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

export function detectCubeOutline(
  canvas: HTMLCanvasElement,
  image: HTMLImageElement,
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

export class TwoViewCamera {
  private imageA?: HTMLImageElement;
  private imageB?: HTMLImageElement;
  private sourceUrlA?: string;
  private sourceUrlB?: string;
  private currentView: "A" | "B" = "A";
  private points: Point[] = [];
  private faces: Partial<Record<(typeof FACES)[number], string>> = {};
  private draggingIndex = -1;
  private hoverIndex = -1;
  private dragMoved = false;

  constructor(private apply: Apply) {
    const canvas = this.canvas;

    const toCanvasCoords = (clientX: number, clientY: number) => {
      const rect = canvas.getBoundingClientRect();
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;
      return {
        x: Math.max(0, Math.min(canvas.width, (clientX - rect.left) * scaleX)),
        y: Math.max(0, Math.min(canvas.height, (clientY - rect.top) * scaleY)),
      };
    };

    const handlePointerDown = (clientX: number, clientY: number) => {
      const activeImage = this.activeImage;
      if (!activeImage) return;
      const { x, y } = toCanvasCoords(clientX, clientY);
      this.dragMoved = false;

      // 既存のポイントで半径22px以内にあるものを探索
      const hit = this.points.findIndex(
        (p) => Math.hypot(p.x - x, p.y - y) <= 22,
      );
      if (hit !== -1) {
        this.draggingIndex = hit;
        canvas.style.cursor = "grabbing";
        this.draw();
      } else {
        this.draggingIndex = -1;
      }
    };

    const handlePointerMove = (clientX: number, clientY: number) => {
      const activeImage = this.activeImage;
      if (!activeImage) return;
      const { x, y } = toCanvasCoords(clientX, clientY);

      if (this.draggingIndex !== -1) {
        this.dragMoved = true;
        this.points[this.draggingIndex] = { x, y };
        this.draw();
        this.update();
      } else {
        const hit = this.points.findIndex(
          (p) => Math.hypot(p.x - x, p.y - y) <= 22,
        );
        this.hoverIndex = hit;
        canvas.style.cursor = hit !== -1 ? "grab" : "crosshair";
        this.draw();
      }
    };

    const handlePointerUp = (clientX: number, clientY: number) => {
      if (this.draggingIndex !== -1) {
        this.draggingIndex = -1;
        canvas.style.cursor = this.hoverIndex !== -1 ? "grab" : "crosshair";
        this.draw();
        return;
      }

      // ドラッグせずにクリックした場合
      if (!this.dragMoved && this.activeImage) {
        const { x, y } = toCanvasCoords(clientX, clientY);
        if (this.points.length < 6) {
          this.points.push({ x, y });
          this.draw();
          this.update();
        }
      }
    };

    // マウスイベント
    canvas.addEventListener("mousedown", (e) =>
      handlePointerDown(e.clientX, e.clientY),
    );
    window.addEventListener("mousemove", (e) => {
      if (this.draggingIndex !== -1) {
        handlePointerMove(e.clientX, e.clientY);
      }
    });
    canvas.addEventListener("mousemove", (e) => {
      if (this.draggingIndex === -1) {
        handlePointerMove(e.clientX, e.clientY);
      }
    });
    window.addEventListener("mouseup", (e) => {
      if (this.draggingIndex !== -1) {
        handlePointerUp(e.clientX, e.clientY);
      }
    });
    canvas.addEventListener("mouseup", (e) => {
      if (this.draggingIndex === -1) {
        handlePointerUp(e.clientX, e.clientY);
      }
    });
    canvas.addEventListener("mouseleave", () => {
      if (this.draggingIndex === -1) {
        this.hoverIndex = -1;
        this.draw();
      }
    });

    // タッチイベント
    canvas.addEventListener(
      "touchstart",
      (e) => {
        if (e.touches.length > 0) {
          e.preventDefault();
          handlePointerDown(e.touches[0].clientX, e.touches[0].clientY);
        }
      },
      { passive: false },
    );
    canvas.addEventListener(
      "touchmove",
      (e) => {
        if (e.touches.length > 0) {
          e.preventDefault();
          handlePointerMove(e.touches[0].clientX, e.touches[0].clientY);
        }
      },
      { passive: false },
    );
    canvas.addEventListener("touchend", (e) => {
      const touch = e.changedTouches[0];
      if (touch) {
        handlePointerUp(touch.clientX, touch.clientY);
      }
    });

    $("camera-close").onclick = () => this.close();

    const faceSelect = $("camera-face") as HTMLSelectElement | null;
    if (faceSelect) {
      faceSelect.onchange = (event) => {
        const val = (event.target as HTMLSelectElement).value;
        const targetView = ["U", "R", "F", "A"].includes(val) ? "A" : "B";
        this.switchView(targetView);
      };
    }

    const detectButton = $("camera-detect");
    if (detectButton) {
      detectButton.onclick = () => this.autoDetectOutline();
    }

    const clearButton = $("camera-clear-points");
    if (clearButton) {
      clearButton.onclick = () => {
        this.points = [];
        this.draw();
        this.update();
      };
    }

    const viewAButton = $("camera-view-a");
    const viewBButton = $("camera-view-b");
    if (viewAButton) viewAButton.onclick = () => this.switchView("A");
    if (viewBButton) viewBButton.onclick = () => this.switchView("B");

    const inputA = $("camera-file-a") as HTMLInputElement;
    const inputB = $("camera-file-b") as HTMLInputElement;
    if (inputA)
      inputA.onchange = (e) => this.loadFile(e.target as HTMLInputElement, "A");
    if (inputB)
      inputB.onchange = (e) => this.loadFile(e.target as HTMLInputElement, "B");

    this.setupDropZone($("camera-drop-a"), inputA, "A");
    this.setupDropZone($("camera-drop-b"), inputB, "B");
    this.setupCanvasDrop(canvas);

    $("camera-capture").onclick = () => this.capture();
    $("camera-apply").onclick = () => {
      const state = buildState(this.faces);
      this.close();
      this.apply(state);
    };
  }

  private setupDropZone(
    dropZone: HTMLElement | null,
    input: HTMLInputElement | null,
    view: "A" | "B",
  ) {
    if (!dropZone) return;
    dropZone.addEventListener("dragover", (e) => {
      e.preventDefault();
      dropZone.classList.add("dragover");
    });
    dropZone.addEventListener("dragleave", () => {
      dropZone.classList.remove("dragover");
    });
    dropZone.addEventListener("drop", (e) => {
      e.preventDefault();
      dropZone.classList.remove("dragover");
      const file = e.dataTransfer?.files?.[0];
      if (file) {
        this.processFile(file, view);
      }
    });
  }

  private setupCanvasDrop(canvas: HTMLCanvasElement) {
    canvas.addEventListener("dragover", (e) => {
      e.preventDefault();
      canvas.classList.add("dragover");
    });
    canvas.addEventListener("dragleave", () => {
      canvas.classList.remove("dragover");
    });
    canvas.addEventListener("drop", (e) => {
      e.preventDefault();
      canvas.classList.remove("dragover");
      const file = e.dataTransfer?.files?.[0];
      if (file) {
        this.processFile(file, this.currentView);
      }
    });
  }

  open() {
    this.faces = {};
    this.points = [];
    this.currentView = "A";
    this.imageA = undefined;
    this.imageB = undefined;
    if (this.sourceUrlA) URL.revokeObjectURL(this.sourceUrlA);
    if (this.sourceUrlB) URL.revokeObjectURL(this.sourceUrlB);
    this.sourceUrlA = undefined;
    this.sourceUrlB = undefined;

    const inputA = $("camera-file-a") as HTMLInputElement | null;
    const inputB = $("camera-file-b") as HTMLInputElement | null;
    if (inputA) inputA.value = "";
    if (inputB) inputB.value = "";

    this.renderImage();
    this.update();
    ($("camera-editor") as HTMLDialogElement).showModal();
  }

  private switchView(view: "A" | "B") {
    if (this.currentView === view && this.points.length === 0) return;
    this.currentView = view;
    this.points = [];
    this.renderImage();
    this.autoDetectOutline();
  }

  private async loadFile(input: HTMLInputElement, view: "A" | "B") {
    const file = input.files?.[0];
    if (!file) return;
    await this.processFile(file, view);
    input.value = "";
  }

  private async processFile(file: File, view: "A" | "B"): Promise<void> {
    if (view === "A") {
      this.sourceUrlA && URL.revokeObjectURL(this.sourceUrlA);
      this.sourceUrlA = URL.createObjectURL(file);
    } else {
      this.sourceUrlB && URL.revokeObjectURL(this.sourceUrlB);
      this.sourceUrlB = URL.createObjectURL(file);
    }
    const url = view === "A" ? this.sourceUrlA : this.sourceUrlB;
    return new Promise((resolve) => {
      const image = new Image();
      image.onload = () => {
        if (view === "A") {
          this.imageA = image;
        } else {
          this.imageB = image;
        }
        this.currentView = view;
        this.renderImage();
        this.autoDetectOutline();
        resolve();
      };
      image.onerror = () => {
        this.error(`画像${view}を読み込めませんでした。`);
        resolve();
      };
      image.src = url!;
    });
  }

  private autoDetectOutline() {
    const activeImage = this.activeImage;
    if (!activeImage) {
      this.points = [];
      this.draw();
      this.update();
      return;
    }
    // まず画像をCanvasに描画した上で検出
    this.draw();
    this.points = detectCubeOutline(this.canvas, activeImage);
    this.draw();
    this.update();
  }

  private capture() {
    const activeImage = this.activeImage;
    if (!activeImage || this.points.length !== 6) return;
    try {
      const scaleX = activeImage.naturalWidth / this.canvas.width;
      const scaleY = activeImage.naturalHeight / this.canvas.height;
      const pts = this.points.map((point) => ({
        x: point.x * scaleX,
        y: point.y * scaleY,
      }));
      const center = computeCenter(pts);
      const [p1, p2, p3, p4, p5, p6] = pts;

      if (this.currentView === "A") {
        // 画像A: 上面=U, 前面左=F, 前面右=R
        // U面: 左上=P1, 右上=P2, 右下=center, 左下=P6
        this.faces["U"] = sampleFace(activeImage, [p1, p2, center, p6]);
        // F面: 左上=P6, 右上=center, 右下=P4, 左下=P5
        this.faces["F"] = sampleFace(activeImage, [p6, center, p4, p5]);
        // R面: 左上=center, 右上=P2, 右下=P3, 左下=P4
        this.faces["R"] = sampleFace(activeImage, [center, p2, p3, p4]);
      } else {
        // 画像B: 上面=D, 前面左=L, 前面右=B
        // D面: 左上=P6, 右上=P1, 右下=P2, 左下=center
        this.faces["D"] = sampleFace(activeImage, [p6, p1, p2, center]);
        // L面: 左上=P4, 右上=P5, 右下=P6, 左下=center
        this.faces["L"] = sampleFace(activeImage, [p4, p5, p6, center]);
        // B面: 左上=P3, 右上=P4, 右下=center, 左下=P2
        this.faces["B"] = sampleFace(activeImage, [p3, p4, center, p2]);
      }

      this.points = [];
      this.draw();
      this.update();

      // 自動で画像Bに切り替える（画像A読取完了時かつ画像Bが未読取の場合）
      if (this.currentView === "A" && !this.faces["D"]) {
        this.switchView("B");
      }
    } catch (error) {
      this.error(String(error));
    }
  }

  private get activeImage(): HTMLImageElement | undefined {
    return this.currentView === "A" ? this.imageA : this.imageB;
  }

  private renderImage() {
    const canvas = this.canvas;
    const activeImage = this.activeImage;
    if (!activeImage) {
      canvas.width = 640;
      canvas.height = 480;
      this.draw();
      return;
    }
    const scale = Math.min(
      640 / activeImage.naturalWidth,
      480 / activeImage.naturalHeight,
    );
    canvas.width = Math.max(1, Math.round(activeImage.naturalWidth * scale));
    canvas.height = Math.max(1, Math.round(activeImage.naturalHeight * scale));
    this.draw();
  }

  private draw() {
    const context = this.canvas.getContext("2d");
    if (!context) return;
    context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    const activeImage = this.activeImage;
    if (activeImage) {
      context.drawImage(
        activeImage,
        0,
        0,
        this.canvas.width,
        this.canvas.height,
      );
    }
    context.lineWidth = 3;
    context.font = "bold 16px sans-serif";

    // 2点以上の場合は外周線を引く
    if (this.points.length > 1) {
      context.strokeStyle = "#c4ed94";
      context.beginPath();
      context.moveTo(this.points[0].x, this.points[0].y);
      for (let i = 1; i < this.points.length; i++) {
        context.lineTo(this.points[i].x, this.points[i].y);
      }
      if (this.points.length === 6) {
        context.closePath();
      }
      context.stroke();
    }

    // 6点揃ったら、中央点と各面の境界線を描画
    if (this.points.length === 6) {
      const center = computeCenter(this.points);

      // 中央点
      context.fillStyle = "#facc15";
      context.beginPath();
      context.arc(center.x, center.y, 6, 0, Math.PI * 2);
      context.fill();
      context.fillText("中心", center.x + 10, center.y + 5);

      // 境界Y字線 (center -> P2, center -> P4, center -> P6)
      context.strokeStyle = "#facc15";
      context.lineWidth = 2;
      context.beginPath();
      context.moveTo(center.x, center.y);
      context.lineTo(this.points[1].x, this.points[1].y); // P2 (右上)
      context.moveTo(center.x, center.y);
      context.lineTo(this.points[3].x, this.points[3].y); // P4 (底)
      context.moveTo(center.x, center.y);
      context.lineTo(this.points[5].x, this.points[5].y); // P6 (左上)
      context.stroke();
    }

    // 各頂点の描画（ドラッグハンドル）
    this.points.forEach((point, index) => {
      const isHovered =
        this.hoverIndex === index || this.draggingIndex === index;

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

  private update() {
    $("camera-progress").textContent =
      `${Object.keys(this.faces).length} / 6 面`;
    $("camera-capture").toggleAttribute(
      "disabled",
      !this.activeImage || this.points.length !== 6,
    );
    $("camera-apply").toggleAttribute(
      "disabled",
      Object.keys(this.faces).length !== 6,
    );

    const faceSelect = $("camera-face") as HTMLSelectElement | null;
    if (faceSelect) {
      if (
        !["U", "R", "F"].includes(faceSelect.value) &&
        this.currentView === "A"
      ) {
        faceSelect.value = "U";
      } else if (
        !["D", "L", "B"].includes(faceSelect.value) &&
        this.currentView === "B"
      ) {
        faceSelect.value = "D";
      }
    }

    const tabA = $("camera-view-a");
    const tabB = $("camera-view-b");
    if (tabA)
      tabA.setAttribute("aria-selected", String(this.currentView === "A"));
    if (tabB)
      tabB.setAttribute("aria-selected", String(this.currentView === "B"));

    const statusA = $("camera-status-a");
    const statusB = $("camera-status-b");
    if (statusA) {
      statusA.textContent = this.imageA
        ? `読込完了 (${this.imageA.naturalWidth}×${this.imageA.naturalHeight})`
        : "未選択（クリックまたはドロップ）";
    }
    if (statusB) {
      statusB.textContent = this.imageB
        ? `読込完了 (${this.imageB.naturalWidth}×${this.imageB.naturalHeight})`
        : "未選択（クリックまたはドロップ）";
    }

    const cardA = $("camera-drop-a");
    const cardB = $("camera-drop-b");
    if (cardA) cardA.classList.toggle("has-file", !!this.imageA);
    if (cardB) cardB.classList.toggle("has-file", !!this.imageB);

    if (!this.activeImage) {
      $("camera-help").textContent =
        `画像${this.currentView}が未選択です。画像Aまたは画像Bを選択してください。`;
    } else if (this.points.length < 6) {
      $("camera-help").textContent =
        `画像${this.currentView}：上面のてっぺんから時計回りにキューブ外周の6角をクリックしてください (${this.points.length}/6点)。`;
    } else {
      const facesText =
        this.currentView === "A" ? "上面・右面・前面" : "下面・左面・背面";
      $("camera-help").textContent =
        `画像${this.currentView}の6角を自動検出しました（各角をドラッグして微調整可能）。「この画像を読み取る」を押すと3面（${facesText}）を一括認識します。`;
    }
    $("camera-error").textContent = "";
  }

  private error(message: string) {
    $("camera-error").textContent = message;
  }

  private close() {
    ($("camera-editor") as HTMLDialogElement).close();
    if (this.sourceUrlA) URL.revokeObjectURL(this.sourceUrlA);
    if (this.sourceUrlB) URL.revokeObjectURL(this.sourceUrlB);
    this.sourceUrlA = undefined;
    this.sourceUrlB = undefined;
  }

  private get canvas() {
    return $("camera-canvas") as HTMLCanvasElement;
  }
}

const $ = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
