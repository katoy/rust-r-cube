import {
  type ResultData,
  type SolverAlgorithm,
  SOLVER_META_BY_TYPE,
} from "./model";
import type { AppStateMachine } from "./app-state";
import type { CubeStore } from "./cube-store";
import type { SolverClient } from "./solver-client";

export type CompareEntry =
  | { status: "pending" }
  | { status: "done"; result: ResultData }
  | { status: "error"; error: string };

export interface CompareControllerDeps {
  store: CubeStore;
  appState: AppStateMachine;
  getSolver: () => SolverClient | undefined;
  ensureSolverReady: () => Promise<boolean>;
  validateState: (state: string) => boolean;
  stop: () => void;
  message: (text?: string) => void;
  persist: () => void;
  refresh: () => void;
  updateSolverNote: () => void;
  isSolving: () => boolean;
  setSolving: (val: boolean) => void;
  setIntervalId: (id: number) => void;
  clearIntervalId: () => void;
  getIncludeOrientation: () => boolean;
  getSolverAlgoEl: () => HTMLSelectElement;
  getSolverNoteEl: () => HTMLElement;
  getExtendedEl: () => HTMLElement;
  getComparePanelEl: () => HTMLElement | null;
  getCompareListEl: () => HTMLElement | null;
  getCompareSummaryEl: () => HTMLElement | null;
}

export function createCompareController(deps: CompareControllerDeps) {
  const compareEntries = new Map<SolverAlgorithm, CompareEntry>();
  let activeComparedAlgo: SolverAlgorithm | undefined = undefined;
  let compareVersion = 0;
  let lastRenderedCompareVersion = -1;

  function hasComparison(): boolean {
    return compareEntries.size > 0;
  }

  function clearComparison() {
    if (compareEntries.size === 0 && activeComparedAlgo === undefined) return;
    compareEntries.clear();
    activeComparedAlgo = undefined;
    compareVersion++;
  }

  function selectComparedSolver(algo: SolverAlgorithm) {
    if (deps.isSolving()) return;
    const entry = compareEntries.get(algo);
    if (!entry || entry.status !== "done") return;
    deps.stop();
    deps.message();
    activeComparedAlgo = algo;
    deps.getSolverAlgoEl().value = algo;
    deps.updateSolverNote();
    deps.store.restoreBaseSnapshot();
    deps.store.setSolution(entry.result);
    deps.appState.setPreviewing();
    if (entry.result.moves.length === 0) {
      deps.message("すでに6面が揃っています。");
    }
    deps.getSolverNoteEl().textContent = `${entry.result.nodes.toLocaleString()} ノードを探索 · 完成を検証`;
    deps.persist();
    deps.refresh();
  }

  function renderComparisonPanel() {
    const panel = deps.getComparePanelEl();
    const list = deps.getCompareListEl();
    const summary = deps.getCompareSummaryEl();
    if (!panel || !list || !summary) return;

    if (compareEntries.size === 0) {
      panel.hidden = true;
      list.replaceChildren();
      summary.textContent = "";
      lastRenderedCompareVersion = compareVersion;
      return;
    }

    panel.hidden = false;
    const cubeType = deps.store.getCubeType();
    const metas = SOLVER_META_BY_TYPE[cubeType];

    let doneCount = 0;
    let minMoves = Infinity;
    for (const meta of metas) {
      const entry = compareEntries.get(meta.id);
      if (entry?.status === "done") {
        doneCount++;
        if (entry.result.moves.length < minMoves) {
          minMoves = entry.result.moves.length;
        }
      }
    }

    const solving = deps.isSolving();
    summary.textContent = solving
      ? `${doneCount} / ${metas.length} 完了…`
      : `${doneCount} / ${metas.length} 解法 · 行をクリックして再生`;

    if (lastRenderedCompareVersion !== compareVersion) {
      lastRenderedCompareVersion = compareVersion;
      list.replaceChildren();
      for (const meta of metas) {
        const entry = compareEntries.get(meta.id) ?? {
          status: "pending" as const,
        };
        const button = document.createElement("button");
        button.type = "button";
        button.className = "solver-compare-row";
        button.dataset.solver = meta.id;

        const info = document.createElement("div");
        info.className = "compare-algo-info";

        const nameRow = document.createElement("span");
        nameRow.className = "compare-algo-name";
        nameRow.textContent = meta.label;

        if (entry.status === "done" && entry.result.moves.length === minMoves) {
          const badge = document.createElement("span");
          badge.className = "compare-badge-shortest";
          badge.textContent = "最短";
          nameRow.append(badge);
        }

        const desc = document.createElement("span");
        desc.className = "compare-algo-desc";
        desc.textContent = meta.shortDesc;
        info.append(nameRow, desc);

        const stats = document.createElement("div");
        stats.className = "compare-algo-stats";
        if (entry.status === "pending") {
          const pendingSpan = document.createElement("span");
          pendingSpan.className = "compare-status-pending";
          pendingSpan.textContent = "計算中…";
          stats.append(pendingSpan);
        } else if (entry.status === "error") {
          const errSpan = document.createElement("span");
          errSpan.className = "compare-status-error";
          errSpan.textContent = "失敗";
          errSpan.title = entry.error;
          stats.append(errSpan);
        } else {
          const movesSpan = document.createElement("span");
          movesSpan.className = "compare-moves";
          movesSpan.textContent = `${entry.result.moves.length} 手`;

          const timeStr =
            entry.result.elapsed_ms < 1000
              ? `${Math.round(entry.result.elapsed_ms)} ms`
              : `${(entry.result.elapsed_ms / 1000).toFixed(2)} s`;
          const metricsSpan = document.createElement("span");
          metricsSpan.className = "compare-metrics";
          metricsSpan.textContent = `${timeStr} · ${entry.result.nodes.toLocaleString()} nodes`;

          stats.append(movesSpan, metricsSpan);
        }

        button.append(info, stats);
        if (entry.status === "done") {
          button.onclick = () => selectComparedSolver(meta.id);
        }
        list.append(button);
      }
    }

    list
      .querySelectorAll<HTMLButtonElement>(".solver-compare-row")
      .forEach((button) => {
        const algo = button.dataset.solver as SolverAlgorithm;
        const entry = compareEntries.get(algo);
        const isActive =
          entry?.status === "done" && activeComparedAlgo === algo;
        button.classList.toggle("is-active", isActive);
        button.setAttribute("aria-pressed", String(isActive));
        button.disabled = solving || entry?.status !== "done";
      });
  }

  async function compareAllSolvers(budget = 5000) {
    if (!(await deps.ensureSolverReady())) return;
    const solver = deps.getSolver();
    if (!solver) return;
    if (!deps.appState.startSolving()) return;
    deps.setSolving(true);
    deps.stop();
    deps.message();
    deps.getExtendedEl().hidden = true;
    if (!deps.store.restoreBaseSnapshot()) {
      deps.store.setSolution(undefined);
    }
    clearComparison();
    try {
      const currentState = deps.store.getState();
      deps.validateState(currentState);
      const at = deps.store.getRevision();
      const cubeType = deps.store.getCubeType();
      const centers = deps.store.getCenterRotations();
      const metas = SOLVER_META_BY_TYPE[cubeType];
      for (const meta of metas) {
        compareEntries.set(meta.id, { status: "pending" });
      }
      compareVersion++;
      const start = performance.now();
      let currentIdx = 0;
      let currentLabel = metas[0]?.label || "";
      deps.setIntervalId(
        window.setInterval(() => {
          const elapsed = performance.now() - start;
          deps.getSolverNoteEl().textContent = `比較中 (${currentIdx + 1}/${metas.length}: ${currentLabel}) · ${(elapsed / 1000).toFixed(1)} 秒`;
        }, 100),
      );
      deps.refresh();

      const includeOrientation = deps.getIncludeOrientation();
      for (let i = 0; i < metas.length; i++) {
        const meta = metas[i];
        currentIdx = i;
        currentLabel = meta.label;
        if (!deps.appState.isSolving() || deps.store.getRevision() !== at)
          return;
        try {
          const result = await solver.solve(
            currentState,
            at,
            budget,
            includeOrientation,
            centers,
            meta.id,
            cubeType,
          );
          if (!deps.appState.isSolving() || deps.store.getRevision() !== at)
            return;
          compareEntries.set(meta.id, { status: "done", result });
        } catch (error) {
          if (String(error).includes("cancelled")) {
            throw error;
          }
          if (!deps.appState.isSolving() || deps.store.getRevision() !== at)
            return;
          compareEntries.set(meta.id, {
            status: "error",
            error: error instanceof Error ? error.message : String(error),
          });
          compareVersion++;
          deps.refresh();
          // タイムアウト等で Worker が終了した場合に再起動して後続アルゴリズムを実行できるようにする
          if (!solver.ready) {
            try {
              await solver.restart();
            } catch {
              // 再起動失敗時は後続も失敗として扱う
            }
            if (!deps.appState.isSolving() || deps.store.getRevision() !== at)
              return;
          }
          continue;
        }
        compareVersion++;
        deps.refresh();
      }

      if (!deps.appState.isSolving() || deps.store.getRevision() !== at) return;
      const solverAlgo = deps.getSolverAlgoEl();
      const preferredId = solverAlgo.value as SolverAlgorithm;
      const preferredEntry = compareEntries.get(preferredId);
      let chosenAlgo: SolverAlgorithm | undefined;
      let chosenResult: ResultData | undefined;
      if (preferredEntry?.status === "done") {
        chosenAlgo = preferredId;
        chosenResult = preferredEntry.result;
      } else {
        for (const meta of metas) {
          const entry = compareEntries.get(meta.id);
          if (entry?.status === "done") {
            chosenAlgo = meta.id;
            chosenResult = entry.result;
            break;
          }
        }
      }

      if (chosenAlgo && chosenResult) {
        activeComparedAlgo = chosenAlgo;
        solverAlgo.value = chosenAlgo;
        deps.updateSolverNote();
        deps.store.setSolution(chosenResult);
        deps.appState.finishSolving(true);
        if (window.innerWidth <= 740) {
          window.scrollTo({ top: 0, behavior: "smooth" });
        }
        deps.getSolverNoteEl().textContent = `${chosenResult.nodes.toLocaleString()} ノードを探索 · 完成を検証`;
        if (chosenResult.moves.length === 0) {
          deps.message("すでに6面が揃っています。");
        }
      } else {
        deps.appState.finishSolving(false);
        const firstError = metas
          .map((m) => compareEntries.get(m.id))
          .find(
            (e): e is { status: "error"; error: string } =>
              e?.status === "error",
          );
        deps.message(firstError?.error || "すべての解法で探索に失敗しました。");
        deps.getSolverNoteEl().textContent = "探索できませんでした";
      }
    } catch (error) {
      if (String(error).includes("cancelled")) {
        clearComparison();
        deps.appState.cancelSolving();
        return;
      }
      clearComparison();
      deps.appState.finishSolving(false);
      deps.message(error instanceof Error ? error.message : String(error));
      deps.getExtendedEl().hidden = true;
      deps.getSolverNoteEl().textContent = "探索できませんでした";
    } finally {
      if (deps.appState.isSolving()) {
        deps.appState.cancelSolving();
      }
      deps.setSolving(false);
      deps.clearIntervalId();
      deps.refresh();
    }
  }

  return {
    hasComparison,
    clearComparison,
    renderComparisonPanel,
    selectComparedSolver,
    compareAllSolvers,
  };
}
