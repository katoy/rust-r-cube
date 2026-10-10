import { FACES, FACE_NAMES, NAMES, COLORS } from "./model";

export interface PaletteOptions {
  container: HTMLElement | null;
  selectedColor: string;
  onSelectColor: (color: string) => void;
}

/**
 * カラーパレット（U, R, F, D, L, B, ?）を生成・描画します。
 */
export function renderPalette(options: PaletteOptions): void {
  const { container, selectedColor, onSelectColor } = options;
  if (!container) return;

  const colors = [...FACES, "?"];
  const existingButtons = container.querySelectorAll<HTMLButtonElement>(
    "button.color-choice",
  );

  if (existingButtons.length === colors.length) {
    existingButtons.forEach((button, i) => {
      const c = colors[i];
      button.setAttribute("aria-checked", String(selectedColor === c));
    });
    return;
  }

  container.replaceChildren();

  colors.forEach((c) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "color-choice";
    button.setAttribute("role", "radio");
    button.setAttribute("aria-checked", String(selectedColor === c));
    button.setAttribute("aria-label", `${NAMES[c]}を選択`);

    const swatch = document.createElement("i");
    swatch.style.background = COLORS[c];

    const text = document.createElement("span");
    text.textContent = NAMES[c];

    button.append(swatch, text);
    button.onclick = () => {
      onSelectColor(c);
      button.focus();
    };
    container.append(button);
  });
}

/**
 * 面の色名（例: "赤"）を返す。3x3 はセンター色で面が決まる。
 * 2x2 はセンターが無く、最多色はスクランブル時に誤解を招くため色名を返さない（呼び出し側で位置名に固定する）。
 */
export function faceColorName(
  faceState: string | undefined,
  is2x2: boolean,
): string | undefined {
  if (is2x2 || !faceState) return undefined;
  const c = faceState[4];
  return c && c !== "?" ? NAMES[c] : undefined;
}

export interface ResultsOptions {
  host: HTMLElement | null;
  faces: Partial<Record<(typeof FACES)[number], string>>;
  currentView: "A" | "B";
  selectedColor: string;
  onUpdateSticker: (face: (typeof FACES)[number], index: number) => void;
  cubeType?: "2x2" | "3x3";
}

/**
 * 展開図形式で読み取り結果の6面を描画し、色の手動修正を可能にします。
 */
export function renderResultFaces(options: ResultsOptions): void {
  const {
    host,
    faces,
    currentView,
    onUpdateSticker,
    cubeType = "3x3",
  } = options;
  if (!host) return;

  const is2x2 = cubeType === "2x2";
  const perFace = is2x2 ? 4 : 9;
  const size = is2x2 ? 2 : 3;

  // 展開図（cube-net）と同じ URFDLB 順
  const faceOrder = ["U", "R", "F", "D", "L", "B"] as const;
  const existingCards =
    host.querySelectorAll<HTMLDivElement>(".camera-face-card");
  const existingCells = host.querySelectorAll<HTMLButtonElement>(".sticker");

  if (
    existingCards.length === faceOrder.length &&
    existingCells.length === faceOrder.length * perFace
  ) {
    existingCards.forEach((card, faceIdx) => {
      const face = faceOrder[faceIdx];
      const viewOfFace = ["U", "R", "F"].includes(face) ? "A" : "B";
      card.classList.toggle("is-active-view", currentView === viewOfFace);

      const faceState = faces[face] ?? (is2x2 ? "????" : "?????????");
      const colorName = faceColorName(faceState, is2x2);
      const centerName = colorName ? `${colorName}面` : FACE_NAMES[face];

      const title = card.querySelector(".camera-face-title");
      if (title) {
        title.textContent = `${face} · ${centerName}`;
      }

      const cells = card.querySelectorAll<HTMLButtonElement>(".sticker");
      cells.forEach((cell, i) => {
        const isCenter = !is2x2 && i === 4;
        const color = isCenter ? face : faceState[i];
        cell.dataset.color = color;
        const row = Math.floor(i / size) + 1;
        const col = (i % size) + 1;
        cell.setAttribute(
          "aria-label",
          `${FACE_NAMES[face]} ${row}行${col}列 ${NAMES[color] ?? "未入力"}${isCenter ? "（センター）" : ""}`,
        );
        if (!isCenter) {
          cell.onclick = () => {
            onUpdateSticker(face, i);
          };
        }
      });
    });
    return;
  }

  host.replaceChildren();

  faceOrder.forEach((face) => {
    const card = document.createElement("div");
    card.className = `net-face face-${face} camera-face-card`;
    card.id = `camera-face-card-${face}`;
    const viewOfFace = ["U", "R", "F"].includes(face) ? "A" : "B";
    if (currentView === viewOfFace) {
      card.classList.add("is-active-view");
    }

    const faceState = faces[face] ?? (is2x2 ? "????" : "?????????");
    const colorName = faceColorName(faceState, is2x2);
    const centerName = colorName ? `${colorName}面` : FACE_NAMES[face];

    const title = document.createElement("span");
    title.className = "net-label camera-face-title";
    title.textContent = `${face} · ${centerName}`;
    card.append(title);

    const grid = document.createElement("div");
    grid.className = is2x2 ? "face-grid grid-2x2" : "face-grid";

    for (let i = 0; i < perFace; i++) {
      const cell = document.createElement("button");
      cell.type = "button";
      cell.className = "sticker";
      const isCenter = !is2x2 && i === 4;
      const color = isCenter ? face : faceState[i];
      cell.dataset.color = color;
      cell.dataset.index = String(i);
      cell.dataset.face = face;
      if (isCenter) {
        cell.dataset.center = "true";
        cell.disabled = true;
        cell.textContent = face;
      }
      const row = Math.floor(i / size) + 1;
      const col = (i % size) + 1;
      cell.setAttribute(
        "aria-label",
        `${FACE_NAMES[face]} ${row}行${col}列 ${NAMES[color] ?? "未入力"}${isCenter ? "（センター）" : ""}`,
      );

      if (!isCenter) {
        cell.onclick = () => {
          onUpdateSticker(face, i);
        };
      }

      grid.append(cell);
    }

    card.append(grid);
    host.append(card);
  });
}
