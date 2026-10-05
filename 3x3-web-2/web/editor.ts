import { FACES, NAMES, COLORS, FACE_NAMES, getErrorIndices } from "./model";
import { net } from "./view";
import { automaticCenters, centerTurns, centersFromInput } from "./centers";
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
  private turns = [0, 0, 0, 0, 0, 0];
  private manualCenters = false;
  private errorIndices: number[] = [];
  constructor(
    private validate: (s: string) => boolean,
    private apply: (s: string, centers: number[]) => void,
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
      const is2x2 = this.draft.length === 24;
      this.draft = is2x2
        ? "?".repeat(24)
        : [...FACES].map((f) => "????" + f + "????").join("");
      this.errorIndices = [];
      $("editor-error").textContent = "";
      this.render();
    };
    $("auto-centers").onclick = () => {
      try {
        this.turns = centerTurns(automaticCenters(this.draft));
        this.manualCenters = false;
        this.render();
      } catch (error) {
        $("editor-error").textContent = String(error);
      }
    };
    $("editor-apply").onclick = () => {
      try {
        const is2x2 = this.draft.length === 24;
        this.validate(this.draft);
        const centers = is2x2
          ? [0, 0, 0, 0, 0, 0]
          : centersFromInput(this.draft, this.turns);
        this.apply(this.draft, centers);
        $<HTMLDialogElement>("editor").close();
      } catch (error) {
        const msg = String(error);
        $("editor-error").textContent = msg;
        this.errorIndices = getErrorIndices(msg);
        this.render();
      }
    };
  }
  open(state: string, centers: number[]) {
    let fixedDraft = state;
    const is2x2 = state.length === 24;
    if (!is2x2) {
      for (let f = 0; f < 6; f++) {
        const centerIdx = f * 9 + 4;
        if (centerIdx < fixedDraft.length && fixedDraft[centerIdx] === "?") {
          fixedDraft =
            fixedDraft.slice(0, centerIdx) +
            FACES[f] +
            fixedDraft.slice(centerIdx + 1);
        }
      }
    }
    this.draft = fixedDraft;
    this.turns = centerTurns(centers);
    this.manualCenters = false;
    this.face = 0;
    this.errorIndices = [];
    $("editor-error").textContent = "";

    const centerSection = $("center-input-section");
    if (centerSection) {
      centerSection.hidden = is2x2;
    }
    const clearBtn = $("clear-colors");
    if (clearBtn) {
      clearBtn.textContent = is2x2
        ? "すべてのマスを未入力にする"
        : "センター以外を未入力にする";
    }
    const editorIntro = $("editor-intro");
    if (editorIntro) {
      editorIntro.textContent = is2x2
        ? "白が上、緑が前。各面を正面から見た色を入力してください。"
        : "白いセンターを上、緑を前に。各面を正面から見た色を入力してください。";
    }

    this.render();
    $<HTMLDialogElement>("editor").showModal();
  }
  private paint(index: number, source: string) {
    const is2x2 = this.draft.length === 24;
    const perFace = is2x2 ? 4 : 9;
    this.draft =
      this.draft.slice(0, index) + this.color + this.draft.slice(index + 1);
    this.face = Math.floor(index / perFace);
    this.errorIndices = [];
    $("editor-error").textContent = "";
    if (!is2x2 && !this.manualCenters) {
      try {
        this.turns = centerTurns(automaticCenters(this.draft));
      } catch {
        // 編集中で色が未完成な段階のエラーは無視し、適用時に一括バリデーションを実施
      }
    }
    this.render();
    document
      .querySelector<HTMLElement>(`#${source} [data-index="${index}"]`)
      ?.focus();
  }
  private render() {
    const is2x2 = this.draft.length === 24;
    const maxPerColor = is2x2 ? 4 : 9;
    const perFace = is2x2 ? 4 : 9;
    const size = is2x2 ? 2 : 3;

    const palette = $("palette");
    palette.replaceChildren();
    [...FACES].forEach((color) => {
      const count = [...this.draft].filter((f) => f === color).length;
      const button = document.createElement("button");
      button.className = `color-choice ${count > maxPerColor ? "over" : ""}`;
      button.setAttribute("role", "radio");
      button.setAttribute("aria-checked", String(this.color === color));
      button.setAttribute(
        "aria-label",
        `${NAMES[color]}を選択 残り${maxPerColor - count}枚`,
      );
      const swatch = document.createElement("i");
      swatch.style.background = COLORS[color];
      const text = document.createElement("span");
      text.textContent = `${NAMES[color]} ${maxPerColor - count}`;
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
      is2x2 ? undefined : this.turns,
      this.errorIndices,
    );
    $("guide-title").textContent =
      `${this.face + 1} / 6　${FACES[this.face]} · ${FACE_NAMES[FACES[this.face]]}`;
    $("guide-orientation").textContent = orientation[this.face];
    const grid = $("guide-grid");
    grid.className = is2x2 ? "face-grid grid-2x2" : "face-grid";
    grid.replaceChildren();
    const errorSet = new Set(this.errorIndices);
    for (let i = 0; i < perFace; i++) {
      const index = this.face * perFace + i;
      const cell = document.createElement("button");
      cell.className = "sticker";
      if (errorSet.has(index)) {
        cell.classList.add("is-error");
      }
      cell.dataset.color = this.draft[index];
      cell.dataset.index = String(index);
      cell.textContent =
        !is2x2 && i === 4 ? ["↑", "→", "↓", "←"][this.turns[this.face]] : "";
      cell.disabled = !is2x2 && i === 4;
      const row = Math.floor(i / size) + 1;
      const col = (i % size) + 1;
      cell.setAttribute(
        "aria-label",
        `${FACE_NAMES[FACES[this.face]]} ${row}行${col}列 ${NAMES[this.draft[index]] ?? "未入力"}`,
      );
      cell.onclick = () => this.paint(index, "guide-grid");
      grid.append(cell);
    }
    if (!is2x2) {
      const controls = $("center-controls");
      controls.replaceChildren();
      [...FACES].forEach((face, i) => {
        const label = document.createElement("label");
        label.textContent = `${face} · ${NAMES[face]}`;
        const select = document.createElement("select");
        select.id = `center-${face}`;
        select.setAttribute("aria-label", `${face} センターの向き`);
        for (let t = 0; t < 4; t++) {
          const option = document.createElement("option");
          option.value = String(t);
          option.textContent = `${["↑", "→", "↓", "←"][t]} ${t * 90}°`;
          select.append(option);
        }
        select.value = String(this.turns[i]);
        select.onchange = () => {
          this.turns[i] = Number(select.value);
          this.manualCenters = true;
          this.face = i;
          this.render();
          $(`center-${face}`).focus();
        };
        label.append(select);
        controls.append(label);
      });
      $("center-mode").textContent = this.manualCenters
        ? "手動入力：配色を変更しても指定した向きを保持します。"
        : "配色を変更すると、センターの向きも自動設定します。";
    }
    const count = [...this.draft].filter((c) => c !== "?").length;
    $("color-count").textContent = `${count} / ${is2x2 ? 24 : 54} マス入力済み`;
  }
}
