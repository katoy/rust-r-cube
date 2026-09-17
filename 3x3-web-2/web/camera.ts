import {
  type Point,
  computeCenter,
  detectCubeOutline,
} from "./camera-geometry";
import { buildState, sampleFace } from "./image-sampler";
import { COLORS, FACES, FACE_NAMES, NAMES } from "./model";

export { computeCenter, detectCubeOutline, type Point };
type Apply = (state: string) => void;

export class TwoViewCamera {
  private imageA?: HTMLImageElement;
  private imageB?: HTMLImageElement;
  private sourceUrlA?: string;
  private sourceUrlB?: string;
  private currentView: "A" | "B" = "A";
  private points: Point[] = [];
  private centerPoint?: Point;
  private faces: Partial<Record<(typeof FACES)[number], string>> = {};
  private detectedLabels: { A?: string; B?: string } = {};
  private selectedColor = "U";
  private draggingIndex = -1; // 0..5: points, 6: centerPoint
  private hoverIndex = -1; // 0..5: points, 6: centerPoint
  private dragMoved = false;

  constructor(private apply: Apply) {
    const canvas = this.canvas;

    const toCanvasCoords = (clientX: number, clientY: number) => {
      const rect = canvas.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return { x: 0, y: 0 };

      const elemRatio = rect.width / rect.height;
      const canvasRatio = canvas.width / canvas.height;

      let drawWidth = rect.width;
      let drawHeight = rect.height;
      let drawLeft = rect.left;
      let drawTop = rect.top;

      if (elemRatio > canvasRatio) {
        // 横長：左右にレターボックス余白がある場合
        drawWidth = rect.height * canvasRatio;
        drawLeft = rect.left + (rect.width - drawWidth) / 2;
      } else {
        // 縦長：上下にレターボックス余白がある場合
        drawHeight = rect.width / canvasRatio;
        drawTop = rect.top + (rect.height - drawHeight) / 2;
      }

      const scaleX = canvas.width / drawWidth;
      const scaleY = canvas.height / drawHeight;

      return {
        x: Math.max(0, Math.min(canvas.width, (clientX - drawLeft) * scaleX)),
        y: Math.max(0, Math.min(canvas.height, (clientY - drawTop) * scaleY)),
      };
    };

    const findHitTarget = (x: number, y: number) => {
      const rect = canvas.getBoundingClientRect();
      const scaleX = canvas.width / Math.max(1, rect.width);
      // 画面上で最低30px相当の当たり判定領域を確保
      const hitRadius = Math.max(30, 30 * scaleX);

      if (this.points.length === 6) {
        const center = this.centerPoint ?? computeCenter(this.points);
        if (Math.hypot(center.x - x, center.y - y) <= hitRadius) {
          return 6;
        }
      }
      return this.points.findIndex(
        (p) => Math.hypot(p.x - x, p.y - y) <= hitRadius,
      );
    };

    const handlePointerDown = (clientX: number, clientY: number) => {
      const activeImage = this.activeImage;
      if (!activeImage) return;
      const { x, y } = toCanvasCoords(clientX, clientY);
      this.dragMoved = false;

      const hit = findHitTarget(x, y);
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
        if (this.draggingIndex === 6) {
          this.centerPoint = { x, y };
        } else {
          this.points[this.draggingIndex] = { x, y };
        }
        this.draw();
        this.update();
      } else {
        const hit = findHitTarget(x, y);
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
        if (this.points.length === 6) this.updateDetectedLabels();
        this.update();
        return;
      }

      // ドラッグせずにクリックした場合
      if (!this.dragMoved && this.activeImage) {
        const { x, y } = toCanvasCoords(clientX, clientY);
        // 既存の頂点の近くをクリックした場合は新規追加しない
        if (findHitTarget(x, y) === -1 && this.points.length < 6) {
          this.points.push({ x, y });
          this.draw();
          if (this.points.length === 6) this.updateDetectedLabels();
          this.update();
        }
      }
    };

    // ポインターイベント（マウス・トラックパッド・タッチ対応）
    canvas.addEventListener("pointerdown", (e) => {
      e.preventDefault();
      try {
        canvas.setPointerCapture(e.pointerId);
      } catch {}
      handlePointerDown(e.clientX, e.clientY);
    });
    window.addEventListener("pointermove", (e) => {
      if (this.draggingIndex !== -1) {
        handlePointerMove(e.clientX, e.clientY);
      }
    });
    canvas.addEventListener("pointermove", (e) => {
      if (this.draggingIndex === -1) {
        handlePointerMove(e.clientX, e.clientY);
      }
    });
    window.addEventListener("pointerup", (e) => {
      if (this.draggingIndex !== -1) {
        try {
          if (canvas.hasPointerCapture(e.pointerId)) {
            canvas.releasePointerCapture(e.pointerId);
          }
        } catch {}
        handlePointerUp(e.clientX, e.clientY);
      }
    });
    canvas.addEventListener("pointerup", (e) => {
      if (this.draggingIndex === -1) {
        try {
          if (canvas.hasPointerCapture(e.pointerId)) {
            canvas.releasePointerCapture(e.pointerId);
          }
        } catch {}
        handlePointerUp(e.clientX, e.clientY);
      }
    });
    canvas.addEventListener("pointercancel", (e) => {
      try {
        if (canvas.hasPointerCapture(e.pointerId)) {
          canvas.releasePointerCapture(e.pointerId);
        }
      } catch {}
      this.draggingIndex = -1;
      this.hoverIndex = -1;
      this.draw();
    });
    canvas.addEventListener("mouseleave", () => {
      if (this.draggingIndex === -1) {
        this.hoverIndex = -1;
        this.draw();
      }
    });

    // 右クリックで頂点を削除
    canvas.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      const { x, y } = toCanvasCoords(e.clientX, e.clientY);
      const hit = findHitTarget(x, y);
      if (hit >= 0 && hit < this.points.length) {
        this.points.splice(hit, 1);
        this.centerPoint = undefined;
        this.hoverIndex = -1;
        this.draw();
        this.update();
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

    const rotateButton = $("camera-rotate-points");
    if (rotateButton) {
      rotateButton.onclick = () => this.rotatePoints();
    }

    const clearButton = $("camera-clear-points");
    if (clearButton) {
      clearButton.onclick = () => {
        this.points = [];
        this.centerPoint = undefined;
        this.detectedLabels[this.currentView] = undefined;
        this.draw();
        this.update();
      };
    }

    // キーボードショートカット 'r' で枠を回転
    window.addEventListener("keydown", (e) => {
      const dialog = $("camera-editor") as HTMLDialogElement | null;
      if (!dialog?.open) return;
      if ((e.key === "r" || e.key === "R") && this.points.length === 6) {
        e.preventDefault();
        this.rotatePoints();
      }
    });

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

    this.initPalette();
    this.renderResults();
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
    this.detectedLabels = {};
    this.points = [];
    this.centerPoint = undefined;
    this.currentView = "A";
    this.selectedColor = "U";
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
    this.initPalette();
    this.renderResults();
    this.update();
    ($("camera-editor") as HTMLDialogElement).showModal();
  }

  private switchView(view: "A" | "B") {
    if (this.currentView === view && this.points.length === 0) return;
    this.currentView = view;
    this.points = [];
    this.centerPoint = undefined;
    this.renderImage();
    this.renderResults();
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
      this.centerPoint = undefined;
      this.draw();
      this.update();
      return;
    }
    // まず画像をCanvasに描画した上で検出
    this.draw();
    this.points = detectCubeOutline(this.canvas, activeImage);
    this.centerPoint = undefined;
    if (this.points.length === 6) this.updateDetectedLabels();
    this.draw();
    this.update();
  }

  rotatePoints(step = 1) {
    if (this.points.length !== 6) return;
    const s = ((step % 6) + 6) % 6;
    this.points = [...this.points.slice(6 - s), ...this.points.slice(0, 6 - s)];
    this.centerPoint = undefined;
    this.updateDetectedLabels();
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
      const rawCenter = this.centerPoint ?? computeCenter(this.points);
      const center = {
        x: rawCenter.x * scaleX,
        y: rawCenter.y * scaleY,
      };
      const [p1, p2, p3, p4, p5, p6] = pts;

      const rawQuads =
        this.currentView === "A"
          ? [
              { defaultFace: "U", quad: [p1, p2, center, p6] },
              { defaultFace: "R", quad: [center, p2, p3, p4] },
              { defaultFace: "F", quad: [p6, center, p4, p5] },
            ]
          : [
              { defaultFace: "D", quad: [p6, p1, p2, center] },
              { defaultFace: "L", quad: [p4, p5, p6, center] },
              { defaultFace: "B", quad: [p3, p4, center, p2] },
            ];

      // 3面の各面をサンプリング
      const sampledItems = rawQuads.map(({ defaultFace, quad }) => {
        const sampled = sampleFace(activeImage, quad);
        const centerChar = sampled[4];
        return { defaultFace, quad, sampled, centerChar };
      });

      // 3面の画像の各面のセンターを認識して、該当する面の色を設定する
      const usedFaces = new Set<string>();
      const faceAssignments: {
        targetFace: (typeof FACES)[number];
        sampled: string;
      }[] = [];

      // 1. 有効なセンター色で、まだこのキャプチャ内で重複していないものを優先割り当て
      const assignedIndices = new Set<number>();
      for (let i = 0; i < sampledItems.length; i++) {
        const item = sampledItems[i];
        if (
          FACES.includes(item.centerChar) &&
          !usedFaces.has(item.centerChar)
        ) {
          usedFaces.add(item.centerChar);
          faceAssignments.push({
            targetFace: item.centerChar as (typeof FACES)[number],
            sampled: item.sampled,
          });
          assignedIndices.add(i);
        }
      }

      // 2. センター色が '?' または重複している場合は、defaultFace または未割り当ての面から補填
      for (let i = 0; i < sampledItems.length; i++) {
        if (assignedIndices.has(i)) continue;
        const item = sampledItems[i];
        let targetFace = item.defaultFace;
        if (usedFaces.has(targetFace)) {
          const fallback = rawQuads
            .map((q) => q.defaultFace)
            .find((f) => !usedFaces.has(f));
          targetFace = fallback ?? item.defaultFace;
        }
        usedFaces.add(targetFace);
        faceAssignments.push({
          targetFace: targetFace as (typeof FACES)[number],
          sampled: item.sampled,
        });
        assignedIndices.add(i);
      }

      for (const { targetFace, sampled } of faceAssignments) {
        this.faces[targetFace] = sampled;
      }

      this.updateDetectedLabels();
      this.points = [];
      this.centerPoint = undefined;
      this.draw();
      this.renderResults();
      this.update();

      // 自動で画像Bに切り替える（画像A読取完了時かつ画像Bがロード済みで未読取の面がある場合）
      if (
        this.currentView === "A" &&
        this.imageB &&
        Object.keys(this.faces).length < 6
      ) {
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
      const center = this.centerPoint ?? computeCenter(this.points);
      const isCenterHovered = this.hoverIndex === 6 || this.draggingIndex === 6;

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

  private updateDetectedLabels() {
    const view = this.currentView;
    const keys =
      view === "A" ? (["U", "R", "F"] as const) : (["D", "L", "B"] as const);

    // 1. キャプチャ済みのデータがある場合はそのセンター色を使用
    const capturedNames = keys.map((key) => {
      const colorChar = this.faces[key]?.[4];
      return colorChar && colorChar !== "?" ? NAMES[colorChar] : undefined;
    });
    if (capturedNames.some((c) => c !== undefined)) {
      this.detectedLabels[view] = keys
        .map((key, i) =>
          capturedNames[i] ? `${capturedNames[i]}面` : FACE_NAMES[key],
        )
        .join("・");
      return;
    }

    // 2. 現在アクティブな画像と6点が揃っていればサンプリング
    const img = this.activeImage;
    if (img && this.points.length === 6) {
      try {
        const scaleX = img.naturalWidth / this.canvas.width;
        const scaleY = img.naturalHeight / this.canvas.height;
        const pts = this.points.map((point) => ({
          x: point.x * scaleX,
          y: point.y * scaleY,
        }));
        const rawCenter = this.centerPoint ?? computeCenter(this.points);
        const center = {
          x: rawCenter.x * scaleX,
          y: rawCenter.y * scaleY,
        };
        const [p1, p2, p3, p4, p5, p6] = pts;

        const quads =
          view === "A"
            ? [
                [p1, p2, center, p6],
                [center, p2, p3, p4],
                [p6, center, p4, p5],
              ]
            : [
                [p6, p1, p2, center],
                [p4, p5, p6, center],
                [p3, p4, center, p2],
              ];

        const sampledNames = quads.map((quad) => {
          const sampled = sampleFace(img, quad);
          const c = sampled[4];
          return c && c !== "?" ? NAMES[c] : undefined;
        });

        if (sampledNames.some((c) => c !== undefined)) {
          this.detectedLabels[view] = keys
            .map((key, i) =>
              sampledNames[i] ? `${sampledNames[i]}面` : FACE_NAMES[key],
            )
            .join("・");
          return;
        }
      } catch {
        // サンプリング失敗時はフォールバック
      }
    }

    this.detectedLabels[view] = undefined;
  }

  private getViewFacesLabel(view: "A" | "B"): string {
    if (this.detectedLabels[view]) {
      return this.detectedLabels[view]!;
    }
    const defaultKeys = view === "A" ? ["U", "R", "F"] : ["D", "L", "B"];
    return defaultKeys.map((k) => FACE_NAMES[k]).join("・");
  }

  private initPalette() {
    const palette = $("camera-palette");
    if (!palette) return;
    palette.replaceChildren();

    const colors = [...FACES, "?"];
    colors.forEach((c) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "color-choice";
      button.setAttribute("role", "radio");
      button.setAttribute("aria-checked", String(this.selectedColor === c));
      if (this.selectedColor === c) {
        button.setAttribute("aria-pressed", "true");
      }
      button.setAttribute("aria-label", `${NAMES[c]}を選択`);

      const swatch = document.createElement("i");
      swatch.style.background = COLORS[c];

      const text = document.createElement("span");
      text.textContent = NAMES[c];

      button.append(swatch, text);
      button.onclick = () => {
        this.selectedColor = c;
        this.initPalette();
      };
      palette.append(button);
    });
  }

  private renderResults() {
    const host = $("camera-result-faces");
    if (!host) return;
    host.replaceChildren();

    // 展開図（cube-net）と同じ URFDLB 順（CSS Gridにより U: (2,1), L: (1,2), F: (2,2), R: (3,2), B: (4,2), D: (2,3) に配置）
    const faces = ["U", "R", "F", "D", "L", "B"] as const;

    faces.forEach((face) => {
      const card = document.createElement("div");
      card.className = `net-face face-${face} camera-face-card`;
      card.id = `camera-face-card-${face}`;
      const viewOfFace = ["U", "R", "F"].includes(face) ? "A" : "B";
      if (this.currentView === viewOfFace) {
        card.classList.add("is-active-view");
      }

      const faceState = this.faces[face] ?? "?????????";
      const centerColor = faceState[4];
      const centerName =
        centerColor && centerColor !== "?"
          ? `${NAMES[centerColor]}面`
          : FACE_NAMES[face];

      const title = document.createElement("span");
      title.className = "net-label camera-face-title";
      title.textContent = `${face} · ${centerName}`;
      card.append(title);

      const grid = document.createElement("div");
      grid.className = "face-grid";

      for (let i = 0; i < 9; i++) {
        const cell = document.createElement("button");
        cell.type = "button";
        cell.className = "sticker";
        cell.dataset.color = faceState[i];
        cell.dataset.index = String(i);
        cell.dataset.face = face;
        if (i === 4) {
          cell.dataset.center = "true";
          cell.disabled = true;
          cell.textContent = face;
        }
        cell.setAttribute(
          "aria-label",
          `${FACE_NAMES[face]} ${Math.floor(i / 3) + 1}行${(i % 3) + 1}列 ${NAMES[faceState[i]]}${i === 4 ? "（センター）" : ""}`,
        );

        if (i !== 4) {
          cell.onclick = () => {
            const current = this.faces[face] ?? "?????????";
            const updated =
              current.substring(0, i) +
              this.selectedColor +
              current.substring(i + 1);
            this.faces[face] = updated;

            this.renderResults();
            this.update();
          };
        }

        grid.append(cell);
      }

      card.append(grid);
      host.append(card);
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

      const optU = faceSelect.querySelector('option[value="U"]');
      if (optU) {
        optU.textContent = `画像A（${this.getViewFacesLabel("A")}）`;
      }
      const optD = faceSelect.querySelector('option[value="D"]');
      if (optD) {
        optD.textContent = `画像B（${this.getViewFacesLabel("B")}）`;
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
    if (cardA) {
      cardA.classList.toggle("has-file", !!this.imageA);
      const titleA = cardA.querySelector(".file-card-title");
      if (titleA) {
        titleA.textContent = `画像A（${this.getViewFacesLabel("A")}）`;
      }
    }
    if (cardB) {
      cardB.classList.toggle("has-file", !!this.imageB);
      const titleB = cardB.querySelector(".file-card-title");
      if (titleB) {
        titleB.textContent = `画像B（${this.getViewFacesLabel("B")}）`;
      }
    }

    const rotateBtn = $("camera-rotate-points") as HTMLButtonElement | null;
    if (rotateBtn) {
      rotateBtn.disabled = this.points.length !== 6;
    }

    if (!this.activeImage) {
      $("camera-help").textContent =
        `画像${this.currentView}が未選択です。画像Aまたは画像Bを選択してください。`;
    } else if (this.points.length < 6) {
      $("camera-help").textContent =
        `画像${this.currentView}：上面のてっぺんから時計回りにキューブ外周の6角をクリックしてください (${this.points.length}/6点)。`;
    } else {
      const facesText = this.getViewFacesLabel(this.currentView);
      $("camera-help").textContent =
        `画像${this.currentView}の6角と中心点を自動検出しました（枠の向きが合わない場合は「🔄 枠を回転」で60°調整可能。頂点・中心ドラッグで微調整）。「この3面を読み取る」を押すと3面（${facesText}）を一括認識します。`;
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
