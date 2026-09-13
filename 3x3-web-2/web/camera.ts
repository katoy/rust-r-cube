import { buildState, sampleFace } from "./image-sampler";
import { FACES } from "./model";

type Point = { x: number; y: number };
type Apply = (state: string) => void;

export class TwoViewCamera {
  private image?: HTMLImageElement;
  private points: Point[] = [];
  private faces: Partial<Record<(typeof FACES)[number], string>> = {};
  private sourceUrl?: string;
  private source = "";
  private face = "U";
  constructor(private apply: Apply) {
    const canvas = this.canvas;
    canvas.addEventListener("click", (event) => {
      if (!this.image) return;
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
      this.points = [];
      this.draw();
      this.update();
    };
    for (const id of ["camera-file-a", "camera-file-b"]) {
      $(id).onchange = (event) => this.load(event.target as HTMLInputElement);
    }
    $("camera-capture").onclick = () => this.capture();
    $("camera-apply").onclick = () => {
      const state = buildState(this.faces);
      this.close();
      this.apply(state);
    };
  }
  open() {
    this.faces = {};
    this.points = [];
    this.face = "U";
    this.source = "";
    this.image = undefined;
    this.renderImage();
    this.update();
    ($("camera-editor") as HTMLDialogElement).showModal();
  }
  private async load(input: HTMLInputElement) {
    const file = input.files?.[0];
    if (!file) return;
    this.sourceUrl && URL.revokeObjectURL(this.sourceUrl);
    this.sourceUrl = URL.createObjectURL(file);
    const image = new Image();
    image.onload = () => {
      this.image = image;
      this.source = input.id.endsWith("a") ? "A" : "B";
      this.points = [];
      this.renderImage();
      this.update();
    };
    image.onerror = () => this.error("画像を読み込めませんでした。");
    image.src = this.sourceUrl;
  }
  private capture() {
    if (!this.image || this.points.length !== 4) return;
    try {
      const scaleX = this.image.naturalWidth / this.canvas.width;
      const scaleY = this.image.naturalHeight / this.canvas.height;
      const imagePoints = this.points.map((point) => ({
        x: point.x * scaleX,
        y: point.y * scaleY,
      }));
      this.faces[this.face as (typeof FACES)[number]] = sampleFace(
        this.image,
        imagePoints,
      );
      this.points = [];
      this.draw();
      this.update();
    } catch (error) {
      this.error(String(error));
    }
  }
  private renderImage() {
    const canvas = this.canvas;
    if (!this.image) {
      canvas.width = 640;
      canvas.height = 480;
      this.draw();
      return;
    }
    const scale = Math.min(
      640 / this.image.naturalWidth,
      480 / this.image.naturalHeight,
    );
    canvas.width = Math.max(1, Math.round(this.image.naturalWidth * scale));
    canvas.height = Math.max(1, Math.round(this.image.naturalHeight * scale));
    this.draw();
  }
  private draw() {
    const context = this.canvas.getContext("2d");
    if (!context) return;
    context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    if (this.image)
      context.drawImage(
        this.image,
        0,
        0,
        this.canvas.width,
        this.canvas.height,
      );
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
      !this.image || this.points.length !== 4,
    );
    $("camera-apply").toggleAttribute(
      "disabled",
      Object.keys(this.faces).length !== 6,
    );
    $("camera-help").textContent = this.image
      ? `${this.source}画像の${this.face}面を指定中：四隅を左上→右上→右下→左下の順でクリックしてください。`
      : "画像Aまたは画像Bを選択してください。";
    $("camera-error").textContent = "";
  }
  private error(message: string) {
    $("camera-error").textContent = message;
  }
  private close() {
    ($("camera-editor") as HTMLDialogElement).close();
    this.sourceUrl && URL.revokeObjectURL(this.sourceUrl);
    this.sourceUrl = undefined;
  }
  private get canvas() {
    return $("camera-canvas") as HTMLCanvasElement;
  }
}

const $ = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
