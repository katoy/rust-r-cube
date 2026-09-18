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
  container.replaceChildren();

  const colors = [...FACES, "?"];
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
    };
    container.append(button);
  });
}

export interface ResultsOptions {
  host: HTMLElement | null;
  faces: Partial<Record<(typeof FACES)[number], string>>;
  currentView: "A" | "B";
  selectedColor: string;
  onUpdateSticker: (face: (typeof FACES)[number], index: number) => void;
}

/**
 * 展開図形式で読み取り結果の6面（各9ステッカー）を描画し、色の手動修正を可能にします。
 */
export function renderResultFaces(options: ResultsOptions): void {
  const { host, faces, currentView, onUpdateSticker } = options;
  if (!host) return;
  host.replaceChildren();

  // 展開図（cube-net）と同じ URFDLB 順
  const faceOrder = ["U", "R", "F", "D", "L", "B"] as const;

  faceOrder.forEach((face) => {
    const card = document.createElement("div");
    card.className = `net-face face-${face} camera-face-card`;
    card.id = `camera-face-card-${face}`;
    const viewOfFace = ["U", "R", "F"].includes(face) ? "A" : "B";
    if (currentView === viewOfFace) {
      card.classList.add("is-active-view");
    }

    const faceState = faces[face] ?? "?????????";
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
          onUpdateSticker(face, i);
        };
      }

      grid.append(cell);
    }

    card.append(grid);
    host.append(card);
  });
}
