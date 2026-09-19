import {
  type Point,
  computeCenter,
  detectCubeOutline,
} from "./camera-geometry";
import {
  toCanvasCoords,
  findHitTarget,
  rotatePointsArray,
} from "./camera-ui-helper";
import { renderCanvasOverlay } from "./camera-canvas-renderer";
import { renderPalette, renderResultFaces } from "./camera-results-ui";
import { buildState, sampleFace } from "./image-sampler";
import { FACES, FACE_NAMES, NAMES } from "./model";

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
  private mediaStream?: MediaStream;
  private isStreaming = false;
  private streamRafId?: number;
  private streamRequestId = 0;

  constructor(private apply: Apply) {
    const canvas = this.canvas;
    const dialog = $("camera-editor") as HTMLDialogElement;
    dialog.addEventListener("close", () => this.handleDialogClose());
    dialog.addEventListener("cancel", () => this.handleDialogClose());

    const getCanvasCoords = (clientX: number, clientY: number) =>
      toCanvasCoords(canvas, clientX, clientY);

    const checkHit = (x: number, y: number) =>
      findHitTarget(canvas, this.points, this.centerPoint, x, y, computeCenter);

    const handlePointerDown = (clientX: number, clientY: number) => {
      const activeImage = this.activeImage;
      if (!activeImage) return;
      const { x, y } = getCanvasCoords(clientX, clientY);
      this.dragMoved = false;

      const hit = checkHit(x, y);
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
      const { x, y } = getCanvasCoords(clientX, clientY);

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
        const hit = checkHit(x, y);
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
        const { x, y } = getCanvasCoords(clientX, clientY);
        // 既存の頂点の近くをクリックした場合は新規追加しない
        if (checkHit(x, y) === -1 && this.points.length < 6) {
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
      const { x, y } = getCanvasCoords(e.clientX, e.clientY);
      const hit = checkHit(x, y);
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

    const liveBtn = $("camera-live-stream");
    const takePhotoBtn = $("camera-take-photo");
    const stopStreamBtn = $("camera-stop-stream");

    if (liveBtn) liveBtn.onclick = () => void this.startLiveStream();
    if (takePhotoBtn) takePhotoBtn.onclick = () => this.captureLiveFrame();
    if (stopStreamBtn) stopStreamBtn.onclick = () => this.stopLiveStream();

    $("camera-capture").onclick = () => this.capture();
    $("camera-apply").onclick = () => {
      const state = buildState(this.faces);
      this.close();
      this.apply(state);
    };

    this.initPalette();
    this.renderResults();
    (window as any).__lastCamera = this;
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
    this.points = rotatePointsArray(this.points, step);
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
        const normalized = sampled.slice(0, 4) + targetFace + sampled.slice(5);
        this.faces[targetFace] = normalized;
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
    renderCanvasOverlay(this.canvas, {
      activeImage: this.activeImage,
      points: this.points,
      centerPoint: this.centerPoint,
      hoverIndex: this.hoverIndex,
      draggingIndex: this.draggingIndex,
      computeCenterFn: computeCenter,
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
    renderPalette({
      container: $("camera-palette"),
      selectedColor: this.selectedColor,
      onSelectColor: (c) => {
        this.selectedColor = c;
        this.initPalette();
      },
    });
  }

  private renderResults() {
    renderResultFaces({
      host: $("camera-result-faces"),
      faces: this.faces,
      currentView: this.currentView,
      selectedColor: this.selectedColor,
      onUpdateSticker: (face, i) => {
        const current = this.faces[face] ?? "?????????";
        const updated =
          current.substring(0, i) +
          this.selectedColor +
          current.substring(i + 1);
        this.faces[face] = updated;

        this.renderResults();
        this.update();
      },
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

  public async startLiveStream() {
    this.error("");
    if (!navigator.mediaDevices?.getUserMedia) {
      this.error(
        "お使いの環境ではライブカメラ（getUserMedia）をご利用いただけません。画像ファイルを選択してください。",
      );
      return;
    }
    const requestId = ++this.streamRequestId;
    const dialog = $("camera-editor") as HTMLDialogElement;
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: {
          facingMode: "environment",
          width: { ideal: 1280 },
          height: { ideal: 960 },
        },
        audio: false,
      });
      if (requestId !== this.streamRequestId || !dialog.open) {
        stream.getTracks().forEach((track) => track.stop());
        return;
      }
      this.mediaStream = stream;
      const video = $("camera-video") as HTMLVideoElement;
      video.srcObject = this.mediaStream;
      await video.play();
      if (requestId !== this.streamRequestId || !dialog.open) {
        this.stopLiveStream();
        return;
      }
      this.isStreaming = true;
      $("camera-live-stream").hidden = true;
      $("camera-take-photo").hidden = false;
      $("camera-stop-stream").hidden = false;
      $("camera-help").textContent =
        "キューブをカメラに向けて「📸 この映像で取り込む」をクリックしてください。";
      this.renderLiveStream();
    } catch {
      if (requestId === this.streamRequestId && dialog.open) {
        this.error(
          "カメラへのアクセスが拒否されたか、カメラを起動できませんでした。",
        );
      }
    }
  }

  private renderLiveStream() {
    if (!this.isStreaming) return;
    const video = $("camera-video") as HTMLVideoElement | null;
    if (video && video.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA) {
      const canvas = this.canvas;
      const ctx = canvas.getContext("2d");
      if (ctx) {
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
      }
    }
    this.streamRafId = requestAnimationFrame(() => this.renderLiveStream());
  }

  public captureLiveFrame() {
    const video = $("camera-video") as HTMLVideoElement | null;
    if (!this.isStreaming || !video) return;
    const offscreen = document.createElement("canvas");
    offscreen.width = video.videoWidth || 640;
    offscreen.height = video.videoHeight || 480;
    const ctx = offscreen.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(video, 0, 0, offscreen.width, offscreen.height);

    offscreen.toBlob(
      (blob) => {
        if (!blob) return;
        const file = new File([blob], `camera-${this.currentView}.jpg`, {
          type: "image/jpeg",
        });
        this.stopLiveStream();
        void this.processFile(file, this.currentView);
      },
      "image/jpeg",
      0.92,
    );
  }

  public stopLiveStream() {
    this.streamRequestId++;
    this.isStreaming = false;
    if (this.streamRafId !== undefined) {
      cancelAnimationFrame(this.streamRafId);
      this.streamRafId = undefined;
    }
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach((track) => track.stop());
      this.mediaStream = undefined;
    }
    const video = $("camera-video") as HTMLVideoElement | null;
    if (video) video.srcObject = null;
    const liveBtn = $("camera-live-stream");
    const takeBtn = $("camera-take-photo");
    const stopBtn = $("camera-stop-stream");
    if (liveBtn) liveBtn.hidden = false;
    if (takeBtn) takeBtn.hidden = true;
    if (stopBtn) stopBtn.hidden = true;
    this.draw();
  }

  private handleDialogClose() {
    this.stopLiveStream();
    if (this.sourceUrlA) URL.revokeObjectURL(this.sourceUrlA);
    if (this.sourceUrlB) URL.revokeObjectURL(this.sourceUrlB);
    this.sourceUrlA = undefined;
    this.sourceUrlB = undefined;
  }

  private close() {
    const dialog = $("camera-editor") as HTMLDialogElement;
    if (dialog.open) {
      dialog.close();
    }
    this.handleDialogClose();
  }

  private get canvas() {
    return $("camera-canvas") as HTMLCanvasElement;
  }
}

const $ = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
