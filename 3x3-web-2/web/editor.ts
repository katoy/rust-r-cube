import { FACES, NAMES, COLORS, FACE_NAMES } from "./model";
import { net } from "./view";
const $ = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
const orientation = [
  "白い面を正面に。上の辺に青、下の辺に緑のセンターが来る向きです。",
  "赤い面を正面に。上は白、左は緑のセンターです。",
  "緑の面を正面に。上は白、右は赤のセンターです。",
  "黄色い面を正面に。上の辺に緑、下の辺に青のセンターが来る向きです。",
  "橙の面を正面に。上は白、右は緑のセンターです。",
  "青い面を正面に。上は白、右は橙のセンターです。",
];
export class ColorEditor {
  private draft = "";
  private color = "U";
  private face = 0;
  constructor(
    private validate: (s: string) => boolean,
    private apply: (s: string) => void,
  ) {
    $("editor-close").onclick = () => $<HTMLDialogElement>("editor").close();
    $("guide-prev").onclick = () => {
      this.face = (this.face + 5) % 6;
      this.render();
    };
    $("guide-next").onclick = () => {
      this.face = (this.face + 1) % 6;
      this.render();
    };
    $("clear-colors").onclick = () => {
      this.draft = [...FACES].map((f) => "????" + f + "????").join("");
      this.render();
    };
    $("editor-apply").onclick = () => {
      try {
        this.validate(this.draft);
        this.apply(this.draft);
        $<HTMLDialogElement>("editor").close();
      } catch (error) {
        $("editor-error").textContent = String(error);
      }
    };
  }
  open(state: string) {
    this.draft = state;
    this.face = 0;
    $("editor-error").textContent = "";
    this.render();
    $<HTMLDialogElement>("editor").showModal();
  }
  private paint(index: number, source: string) {
    this.draft =
      this.draft.slice(0, index) + this.color + this.draft.slice(index + 1);
    this.face = Math.floor(index / 9);
    this.render();
    document
      .querySelector<HTMLElement>(`#${source} [data-index="${index}"]`)
      ?.focus();
  }
  private render() {
    const palette = $("palette");
    palette.replaceChildren();
    [...FACES].forEach((color) => {
      const count = [...this.draft].filter((f) => f === color).length;
      const button = document.createElement("button");
      button.className = `color-choice ${count > 9 ? "over" : ""}`;
      button.setAttribute("aria-pressed", String(this.color === color));
      button.setAttribute(
        "aria-label",
        `${NAMES[color]}を選択 残り${9 - count}枚`,
      );
      const swatch = document.createElement("i");
      swatch.style.background = COLORS[color];
      const text = document.createElement("span");
      text.textContent = `${NAMES[color]} ${9 - count}`;
      button.append(swatch, text);
      button.onclick = () => {
        this.color = color;
        this.render();
        (palette.children[[...FACES].indexOf(color)] as HTMLElement).focus();
      };
      palette.append(button);
    });
    net(
      $("editor-net"),
      this.draft,
      true,
      (i) => this.paint(i, "editor-net"),
      this.face,
    );
    $("guide-title").textContent =
      `${this.face + 1} / 6　${FACES[this.face]} · ${FACE_NAMES[FACES[this.face]]}`;
    $("guide-orientation").textContent = orientation[this.face];
    const grid = $("guide-grid");
    grid.replaceChildren();
    for (let i = 0; i < 9; i++) {
      const index = this.face * 9 + i;
      const cell = document.createElement("button");
      cell.className = "sticker";
      cell.dataset.color = this.draft[index];
      cell.dataset.index = String(index);
      cell.textContent = i === 4 ? FACES[this.face] : "";
      cell.disabled = i === 4;
      cell.setAttribute(
        "aria-label",
        `${FACE_NAMES[FACES[this.face]]} ${Math.floor(i / 3) + 1}行${(i % 3) + 1}列 ${NAMES[this.draft[index]]}`,
      );
      cell.onclick = () => this.paint(index, "guide-grid");
      grid.append(cell);
    }
    const count = [...this.draft].filter((c) => c !== "?").length;
    $("color-count").textContent = `${count} / 54 マス入力済み`;
    $("editor-error").textContent = "";
  }
}
