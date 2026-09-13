import { buildState, sampleFace } from "./image-sampler";
import { FACES } from "./model";

type Point = { x: number; y: number };
type Apply = (state: string) => void;

export class TwoViewCamera {
  private imageA?: HTMLImageElement;
  private imageB?: HTMLImageElement;
  private sourceUrlA?: string;
  private sourceUrlB?: string;
  private currentView: "A" | "B" = "A";
  private points: Point[] = [];
  private faces: Partial<Record<(typeof FACES)[number], string>> = {};
  private face = "U";

  constructor(private apply: Apply) {
    const canvas = this.canvas;
    canvas.addEventListener("click", (event) => {
      const activeImage = this.activeImage;
      if (!activeImage) return;
      const rect = canvas.getBoundingClientRect();
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;
      this.points.push({
        x: (event.clientX - rect.left) * scaleX,
        y: (event.clientY - rect.top) * scaleY,
      });
      this.draw();
      this.update();
    });

    $("camera-close").onclick = () => this.close();

    $("camera-face").onchange = (event) => {
      this.face = (event.target as HTMLSelectElement).value;
      const targetView = ["U", "R", "F"].includes(this.face) ? "A" : "B";
      if (this.currentView !== targetView) {
        this.switchView(targetView);
      } else {
        this.points = [];
        this.draw();
        this.update();
      }
    };

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
    this.face = "U";
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
    this.currentView = view;
    this.points = [];
    this.renderImage();
    this.update();
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
        this.points = [];
        this.renderImage();
        this.update();
        resolve();
      };
      image.onerror = () => {
        this.error(`画像${view}を読み込めませんでした。`);
        resolve();
      };
      image.src = url!;
    });
  }

  private capture() {
    const activeImage = this.activeImage;
    if (!activeImage || this.points.length !== 4) return;
    try {
      const scaleX = activeImage.naturalWidth / this.canvas.width;
      const scaleY = activeImage.naturalHeight / this.canvas.height;
      const imagePoints = this.points.map((point) => ({
        x: point.x * scaleX,
        y: point.y * scaleY,
      }));
      this.faces[this.face as (typeof FACES)[number]] = sampleFace(
        activeImage,
        imagePoints,
      );
      this.points = [];
      this.draw();
      this.update();
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
    context.fillStyle = "#c4ed94";
    context.strokeStyle = "#c4ed94";
    context.lineWidth = 3;
    this.points.forEach((point, index) => {
      context.beginPath();
      context.arc(point.x, point.y, 6, 0, Math.PI * 2);
      context.fill();
      context.fillText(String(index + 1), point.x + 8, point.y - 8);
    });
    if (this.points.length === 4) {
      context.beginPath();
      context.moveTo(this.points[0].x, this.points[0].y);
      this.points.slice(1).forEach((point) => context.lineTo(point.x, point.y));
      context.closePath();
      context.stroke();
    }
  }

  private update() {
    $("camera-progress").textContent =
      `${Object.keys(this.faces).length} / 6 面`;
    $("camera-capture").toggleAttribute(
      "disabled",
      !this.activeImage || this.points.length !== 4,
    );
    $("camera-apply").toggleAttribute(
      "disabled",
      Object.keys(this.faces).length !== 6,
    );

    const faceSelect = $("camera-face") as HTMLSelectElement | null;
    if (faceSelect && faceSelect.value !== this.face) {
      faceSelect.value = this.face;
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

    $("camera-help").textContent = this.activeImage
      ? `画像${this.currentView}の${this.face}面を指定中：四隅を左上→右上→右下→左下の順でクリックしてください。`
      : `画像${this.currentView}が未選択です。画像Aまたは画像Bを選択してください。`;
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
