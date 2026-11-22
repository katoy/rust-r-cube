import {
  type CellArrowInfo,
  getCellArrowInfo,
  countPieceOrientations,
} from "./model";

export function setupArrowLegend() {
  const legendBtn = document.getElementById("arrow-legend-btn");
  const popover = document.getElementById("arrow-legend-popover");
  const closeBtn = document.getElementById("arrow-legend-close");
  if (!legendBtn || !popover) return;

  const togglePopover = (open?: boolean) => {
    const willOpen = open ?? popover.hidden;
    popover.hidden = !willOpen;
    legendBtn.setAttribute("aria-expanded", String(willOpen));
  };

  legendBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    togglePopover();
  });

  closeBtn?.addEventListener("click", (e) => {
    e.stopPropagation();
    togglePopover(false);
  });

  document.addEventListener("click", (e) => {
    if (!popover.hidden && !popover.contains(e.target as Node)) {
      togglePopover(false);
    }
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !popover.hidden) {
      togglePopover(false);
      legendBtn.focus();
    }
  });
}

export function updateArrowLegend(
  state: string,
  centerRotations: number[],
  is2x2: boolean,
  precomputedArrowInfo?: CellArrowInfo,
) {
  const legend = document.getElementById("arrow-legend");
  if (!legend) return;
  if (is2x2) {
    legend.hidden = true;
    return;
  }
  legend.hidden = false;

  const arrowInfo =
    precomputedArrowInfo ?? getCellArrowInfo(state, centerRotations);
  const counts = countPieceOrientations(arrowInfo);

  const countNormal = document.getElementById("legend-count-normal");
  const countCorner = document.getElementById("legend-count-corner");
  const countEdge = document.getElementById("legend-count-edge");
  const countCenter = document.getElementById("legend-count-center");

  if (countNormal) countNormal.textContent = String(counts.normal);
  if (countCorner) countCorner.textContent = String(counts.cornerTwist);
  if (countEdge) countEdge.textContent = String(counts.edgeFlip);
  if (countCenter) countCenter.textContent = String(counts.centerRotate);

  const btn = document.getElementById("arrow-legend-btn");
  if (btn) {
    const hasIssues =
      counts.cornerTwist > 0 || counts.edgeFlip > 0 || counts.centerRotate > 0;
    btn.classList.toggle("has-issues", hasIssues);
    btn.classList.toggle("is-all-normal", !hasIssues);
  }

  const pillCorner = legend.querySelector(".pill-corner");
  const pillEdge = legend.querySelector(".pill-edge");
  const pillCenter = legend.querySelector(".pill-center");

  pillCorner?.classList.toggle("has-count", counts.cornerTwist > 0);
  pillEdge?.classList.toggle("has-count", counts.edgeFlip > 0);
  pillCenter?.classList.toggle("has-count", counts.centerRotate > 0);
}
