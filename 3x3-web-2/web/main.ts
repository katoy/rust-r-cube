import "./style.css";
import init, * as cubeStudio from "../pkg/cube_studio";
const { apply_moves, validate, scramble } = cubeStudio;
import wasmUrl from "../pkg/cube_studio_bg.wasm?url";
import { SOLVED, FACES, inverse, instruction, type ResultData } from "./model";
import { mount, icon, net } from "./view";
import { CubeScene } from "./scene";
import { SolverClient } from "./solver-client";
import { CubeStore } from "./cube-store";
import {
  automaticCenters,
  centerTurns,
  centersFromInput,
  rotateCenters,
} from "./centers";

import { registerServiceWorker } from "./pwa";
import { sound } from "./sound";
import { analyzeMoves } from "./triggers";
import { setupKeyboardShortcuts } from "./keyboard-shortcuts";
import { parseUrlParams, buildShareUrl } from "./url-params";
import { validateAndParseCubeJson, createCubeJsonBlob } from "./file-io";

declare global {
  interface Window {
    cube_store?: CubeStore;
    cube_scene?: CubeScene;
    cube_studio?: typeof cubeStudio;
  }
}

const store = new CubeStore();
window.cube_store = store;

mount();
registerServiceWorker();
export function getScopedStorageKey(
  baseKey: string,
  customPath?: string,
): string {
  const p =
    customPath !== undefined
      ? customPath
      : typeof window !== "undefined"
        ? window.location.pathname
        : "/";
  const withoutIndex = p.replace(/\/index\.html$/i, "");
  const normalized = withoutIndex.replace(/\/+$/, "") || "/";
  return normalized !== "/" ? `${baseKey}:${normalized}` : baseKey;
}

const $ = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
const storageKey = getScopedStorageKey("cube-studio-v1");
let mainReady = false,
  engineError = false,
  solving = false;
let playing = false,
  motion = 0,
  inMotion = false,
  playbackRun = 0;
let scene: CubeScene | undefined;
let solver: SolverClient | undefined,
  interval = 0;
let restoring = true;
const reduced = $<HTMLInputElement>("reduced-motion");
reduced.checked = matchMedia("(prefers-reduced-motion: reduce)").matches;
const includeOrientation = $<HTMLInputElement>("include-orientation");
includeOrientation.checked = true;
const solverAlgo = $<HTMLSelectElement>("solver-algorithm");
solverAlgo.onchange = () => {
  persist();
};

const soundToggleBtn = $("sound-toggle");
function updateSoundButton() {
  const enabled = sound.isEnabled();
  soundToggleBtn.setAttribute("aria-pressed", String(enabled));
  soundToggleBtn.setAttribute(
    "aria-label",
    enabled ? "効果音をミュート" : "効果音を有効化",
  );
  soundToggleBtn.innerHTML = icon(enabled ? "volume" : "mute");
}
updateSoundButton();
soundToggleBtn.onclick = () => {
  sound.toggle();
  updateSoundButton();
};

function message(text = "") {
  $("message").textContent = text;
}
function stop() {
  const wasPlaying = playing;
  playing = false;
  playbackRun++;
  motion++;
  scene?.finish();
  inMotion = false;
  if (wasPlaying) {
    persist();
  }
}
function cancelSearch() {
  if (solving) {
    solving = false;
    clearInterval(interval);
    solver?.cancel();
  }
}
function persist() {
  if (restoring) return;
  try {
    localStorage.setItem(
      storageKey,
      JSON.stringify({
        version: 1,
        ...store.getSnapshot(),
        reducedMotion: reduced.checked,
        speed: $<HTMLSelectElement>("speed").value,
        solverAlgorithm: solverAlgo.value,
      }),
    );
  } catch {
    message(
      "ブラウザの保存領域を利用できません。必要な状態は「保存」でダウンロードしてください。",
    );
  }
}
function replace(
  next: string,
  record = true,
  centers = automaticCenters(next),
) {
  stop();
  cancelSearch();
  message();
  store.replace(next, record, centers);
}
let renderedSolution: ResultData | undefined = undefined;
let cachedAnalyzedMoves: ReturnType<typeof analyzeMoves> = [];
let lastRenderedStep: number | undefined = undefined;

function refresh() {
  const state = store.getState();
  const solution = store.getSolution();
  const step = store.getStep();
  const centerRotations = store.getCenterRotations();
  const next = solution?.moves[step] || "";
  const statusText =
    state === SOLVED
      ? store.getCenterTurns().some((t) => t !== 0)
        ? "色は完成・センターの向きあり"
        : "完成状態"
      : solution
        ? `${step} / ${solution.moves.length} 手`
        : "スクランブル状態";
  if (scene) {
    scene.centerRotations = [...centerRotations];
    if (!inMotion) scene.show(state, next);
    scene.updateAriaLabel(statusText);
  }
  if (!inMotion) {
    net($("fallback-net"), state, false, undefined, -1, store.getCenterTurns());
  }
  $("cube-status").textContent = statusText;
  $("scene").dataset.state = state;
  $<HTMLButtonElement>("solve").disabled =
    !mainReady || (!solver?.ready && !engineError) || solving;
  $("solve").hidden = solving;
  $("cancel").hidden = !solving;
  $("solve").innerHTML =
    `<span>${engineError ? "エンジンを再試行" : state === SOLVED ? "完成状態を確認" : "解法を探す"}</span>${icon("arrow")}`;
  $<HTMLButtonElement>("undo").disabled = !mainReady || !store.canUndo();
  $<HTMLButtonElement>("redo").disabled = !mainReady || !store.canRedo();
  document
    .querySelectorAll<HTMLButtonElement>(
      "[data-move],#scramble,#reset,#apply-algorithm,#edit-colors,#camera-colors,#save,#load,#preset-buttons button",
    )
    .forEach((b) => (b.disabled = !mainReady));
  $("solution-empty").hidden = !!solution;
  $("solution-content").hidden = !solution;
  document.body.classList.toggle("has-solution", !!solution);
  $<HTMLButtonElement>("copy").disabled = !solution;
  $("solution-close").hidden = !solution;
  if (solution) {
    $("move-count").textContent = `${solution.moves.length} 手`;
    $("solve-time").textContent =
      `${solution.elapsed_ms < 1000 ? `${Math.round(solution.elapsed_ms)} ms` : `${(solution.elapsed_ms / 1000).toFixed(2)} 秒`} · 検証済み`;
    const list = $("move-list");
    const solutionChanged = renderedSolution !== solution;
    if (solutionChanged) {
      renderedSolution = solution;
      cachedAnalyzedMoves = analyzeMoves(solution.moves, solution.phases);
      list.replaceChildren();
      cachedAnalyzedMoves.forEach((meta, i) => {
        if (
          i === 0 ||
          meta.phase !== cachedAnalyzedMoves[i - 1].phase ||
          meta.phaseLabel !== cachedAnalyzedMoves[i - 1].phaseLabel
        ) {
          const phaseBadge = document.createElement("span");
          phaseBadge.className = `phase-badge phase-${meta.phase}`;
          phaseBadge.textContent = meta.phaseLabel;
          list.append(phaseBadge);
        }
        const button = document.createElement("button");
        button.className = "solution-move";
        button.dataset.step = String(i);
        button.textContent = meta.move;
        const titleParts = [meta.phaseLabel];
        if (meta.trigger) titleParts.push(`[${meta.trigger}]`);
        button.title = titleParts.join(" ");
        button.setAttribute(
          "aria-label",
          `${i + 1}手目 ${meta.move} (${meta.phaseLabel}${meta.trigger ? `, ${meta.trigger}` : ""}) の直後へ移動`,
        );
        button.onclick = () => {
          stop();
          void seek(i + 1, false);
        };
        list.append(button);
      });
    }

    const buttons = list.querySelectorAll<HTMLButtonElement>(".solution-move");
    if (solutionChanged || lastRenderedStep === undefined) {
      buttons.forEach((button, i) => {
        button.classList.toggle("done", i < step);
        const isCurrent = i === step;
        button.classList.toggle("current", isCurrent);
        if (isCurrent) {
          button.setAttribute("aria-current", "step");
        } else {
          button.removeAttribute("aria-current");
        }
      });
    } else if (lastRenderedStep !== step) {
      const minStep = Math.min(lastRenderedStep, step);
      const maxStep = Math.max(lastRenderedStep, step);
      for (let i = minStep; i <= maxStep && i < buttons.length; i++) {
        const button = buttons[i];
        button.classList.toggle("done", i < step);
        const isCurrent = i === step;
        button.classList.toggle("current", isCurrent);
        if (isCurrent) {
          button.setAttribute("aria-current", "step");
        } else {
          button.removeAttribute("aria-current");
        }
      }
    }
    lastRenderedStep = step;

    $("next-symbol").textContent = next || "✓";
    const currentMeta = cachedAnalyzedMoves[step];
    const phasePrefix = currentMeta
      ? `【${currentMeta.phaseLabel}${currentMeta.trigger ? ` · ${currentMeta.trigger}` : ""}】 `
      : "";
    $("next-instruction").textContent = next
      ? `${phasePrefix}${instruction(next)}`
      : "6面が揃いました。おつかれさまでした。";
    $("step-count").textContent = `${step} / ${solution.moves.length}`;
    $("play").innerHTML = icon(playing ? "pause" : "play");
    $("play").setAttribute("aria-label", playing ? "一時停止" : "自動再生");
    $<HTMLButtonElement>("play").disabled = solution.moves.length === 0;
    $<HTMLButtonElement>("first").disabled = step === 0;
    $<HTMLButtonElement>("prev").disabled = step === 0;
    $<HTMLButtonElement>("next").disabled = step === solution.moves.length;
    $<HTMLButtonElement>("last").disabled = step === solution.moves.length;
    const timeline = $<HTMLInputElement>("timeline");
    timeline.max = String(solution.moves.length);
    timeline.value = String(step);
    timeline.setAttribute("aria-valuenow", String(step));
    const valuetext =
      step === 0
        ? "開始状態"
        : step === solution.moves.length
          ? `完成 (${solution.moves.length}手)`
          : `${step}手目: ${currentMeta?.move || ""} (${currentMeta?.phaseLabel || ""})`;
    timeline.setAttribute("aria-valuetext", valuetext);

    if (step === 0) {
      list.scrollTop = 0;
    } else {
      const currentButton = list.querySelector(
        `[data-step="${step}"]`,
      ) as HTMLElement | null;
      if (currentButton) {
        currentButton.scrollIntoView({ block: "nearest", inline: "nearest" });
      } else if (step === solution.moves.length && list.lastElementChild) {
        (list.lastElementChild as HTMLElement).scrollIntoView({
          block: "nearest",
          inline: "nearest",
        });
      }
    }
  } else {
    renderedSolution = undefined;
    cachedAnalyzedMoves = [];
    lastRenderedStep = undefined;
    const timeline = $<HTMLInputElement>("timeline");
    timeline.max = "0";
    timeline.value = "0";
    timeline.setAttribute("aria-valuenow", "0");
    timeline.removeAttribute("aria-valuetext");
  }
}
store.subscribe((_s, { type }) => {
  if (type === "modifier") {
    const mod = store.getModifier();
    $("prime").setAttribute("aria-pressed", String(mod === "'"));
    $("double").setAttribute("aria-pressed", String(mod === "2"));
    return;
  }
  if (
    type === "replace" ||
    type === "undo" ||
    type === "redo" ||
    type === "algorithm"
  ) {
    persist();
  }
  refresh();
});
function fallback() {
  scene?.dispose();
  scene = undefined;
  $("scene").hidden = true;
  $("fallback").hidden = false;
  $("view-reset").hidden = true;
  document
    .querySelector<HTMLElement>(".view-presets")
    ?.setAttribute("hidden", "");
  document.querySelector<HTMLElement>(".gesture")!.hidden = true;
}
function initScene(forceError = false) {
  try {
    if (forceError) throw new Error("forced scene error");
    scene = new CubeScene($("scene"));
    window.cube_scene = scene;
    $("scene").addEventListener("render-failed", fallback);
  } catch {
    fallback();
  }
}
initScene();
async function seek(target: number, animate = true) {
  const solution = store.getSolution();
  if (!solution) return;
  target = Math.max(0, Math.min(solution.moves.length, target));
  const old = store.getStep();
  const data = solution;
  const token = ++motion;
  scene?.finish();
  const nextState = data.states[target];
  const move =
    target === old + 1
      ? data.moves[old]
      : target === old - 1
        ? inverse(data.moves[target])
        : undefined;
  const traversed =
    target > old
      ? data.moves.slice(old, target)
      : data.moves.slice(target, old).reverse().map(inverse);
  const nextCenters = rotateCenters(store.getCenterRotations(), traversed);
  inMotion = !!(animate && move && scene && !reduced.checked);
  store.updateAfterSeek(nextState, nextCenters, target);
  if (inMotion && move)
    await scene!.turn(
      move,
      nextState,
      Number($<HTMLSelectElement>("speed").value),
    );
  else if (animate && move && !scene && !reduced.checked)
    await new Promise((resolve) =>
      setTimeout(resolve, Number($<HTMLSelectElement>("speed").value)),
    );
  if (move) sound.playMove();
  if (token === motion) {
    inMotion = false;
    if (!playing) {
      persist();
    }
    refresh();
    if (target === data.moves.length) {
      sound.playSuccess();
    }
  }
}
async function play() {
  const solution = store.getSolution();
  if (!solution) return;
  if (playing) {
    stop();
    refresh();
    return;
  }
  stop();
  if (store.getStep() === solution.moves.length) await seek(0, false);
  playing = true;
  refresh();
  const data = solution;
  const run = playbackRun;
  while (
    playing &&
    run === playbackRun &&
    store.getSolution() === data &&
    store.getStep() < data.moves.length
  ) {
    await seek(store.getStep() + 1);
    if (reduced.checked)
      await new Promise((resolve) =>
        setTimeout(
          resolve,
          Math.max(
            30,
            Math.round(Number($<HTMLSelectElement>("speed").value) / 10),
          ),
        ),
      );
  }
  if (store.getSolution() === data && run === playbackRun) {
    playing = false;
    persist();
    refresh();
  }
}
async function applyAlgorithm(algorithm: string, animate = true) {
  if (!mainReady) return;
  try {
    const result: ResultData = JSON.parse(
      apply_moves(store.getState(), algorithm),
    );
    stop();
    cancelSearch();
    message();
    const nextCenters = rotateCenters(store.getCenterRotations(), result.moves);
    const token = ++motion;
    inMotion = !!(
      animate &&
      result.moves.length === 1 &&
      scene &&
      !reduced.checked
    );
    store.applyAlgorithmResult(result.state, nextCenters);
    sound.playMove();
    if (inMotion) {
      await scene!.turn(
        result.moves[0],
        store.getState(),
        Number($<HTMLSelectElement>("speed").value),
      );
    }
    if (token === motion) {
      inMotion = false;
      refresh();
      if (result.state === SOLVED) {
        sound.playSuccess();
      }
    }
  } catch (error) {
    message(String(error));
  }
}
async function solve(budget = 5000) {
  if (engineError) {
    engineError = false;
    solver?.restart();
    refresh();
    return;
  }
  if (!solver?.ready || solving) return;
  stop();
  message();
  $("extended").hidden = true;
  try {
    const currentState = store.getState();
    validate(currentState);
    solving = true;
    const at = store.getRevision();
    const start = performance.now();
    refresh();
    interval = window.setInterval(() => {
      $("solver-note").textContent =
        `探索中 · ${((performance.now() - start) / 1000).toFixed(1)} 秒 / ${budget / 1000} 秒`;
    }, 100);
    const result = await solver.solve(
      currentState,
      at,
      budget,
      includeOrientation.checked,
      store.getCenterRotations(),
      solverAlgo.value as import("./model").SolverAlgorithm,
    );

    if (store.getRevision() !== at) return;
    store.setSolution(result);
    if (window.innerWidth <= 740) {
      window.scrollTo({ top: 0, behavior: "smooth" });
    }
    $("solver-note").textContent =
      `${result.nodes.toLocaleString()} ノードを探索 · 完成を検証`;
    if (result.moves.length === 0) message("すでに6面が揃っています。");
  } catch (error) {
    if (String(error).includes("cancelled")) return;
    message(error instanceof Error ? error.message : String(error));
    $("extended").hidden = false;
  } finally {
    solving = false;
    clearInterval(interval);
    refresh();
  }
}

function promptReloadForUpdate(reloadFn = () => window.location.reload()) {
  persist();
  const reload = window.confirm(
    "アプリの新しいバージョンがあります。ページを再読み込みして更新しますか？\n（現在のキューブ配置は自動保存されています）",
  );
  if (reload) {
    reloadFn();
  } else {
    message("アプリの更新があります。ページを再読み込みしてください。");
  }
}

let editorInstance: import("./editor").ColorEditor | undefined;
async function getEditor(
  loader: () => Promise<any> = () => import("./editor"),
): Promise<import("./editor").ColorEditor | undefined> {
  if (!editorInstance) {
    try {
      const { ColorEditor } = await loader();
      editorInstance = new ColorEditor(
        (s: string) => validate(s),
        (s: string, centers: number[]) => replace(s, true, centers),
      );
    } catch {
      promptReloadForUpdate();
      return undefined;
    }
  }
  return editorInstance;
}

let cameraInstance: import("./camera").TwoViewCamera | undefined;
async function getCamera(
  loader: () => Promise<any> = () => import("./camera"),
): Promise<import("./camera").TwoViewCamera | undefined> {
  if (!cameraInstance) {
    try {
      const { TwoViewCamera } = await loader();
      cameraInstance = new TwoViewCamera(async (s: string) => {
        let centers = [0, 0, 0, 0, 0, 0];
        try {
          centers = automaticCenters(s);
        } catch {}
        const ed = await getEditor();
        if (ed) ed.open(s, centers);
      });
    } catch {
      promptReloadForUpdate();
      return undefined;
    }
  }
  return cameraInstance;
}

$("edit-colors").onclick = async () => {
  stop();
  refresh();
  const ed = await getEditor();
  if (ed) {
    ed.open(store.getState(), store.getCenterRotations());
  }
};
$("camera-colors").onclick = async () => {
  const cam = await getCamera();
  if (cam) {
    cam.open();
  }
};
$("solve").onclick = () => void solve();
$("extended").onclick = () => void solve(30000);
$("cancel").onclick = () => {
  cancelSearch();
  message("探索を中止しました。");
  $("solver-note").textContent = "エンジンを再準備しています";
  refresh();
};
$("scramble").onclick = () => {
  if (!mainReady) return;
  const seed = crypto.getRandomValues(new Uint32Array(1))[0];
  const algorithm = scramble(seed);
  const result: ResultData = JSON.parse(apply_moves(SOLVED, algorithm));
  replace(result.state, true, rotateCenters([0, 0, 0, 0, 0, 0], result.moves));
  $("scramble-text").textContent = algorithm;
};
$("apply-algorithm").onclick = () =>
  void applyAlgorithm($<HTMLTextAreaElement>("algorithm").value);
document
  .querySelectorAll<HTMLButtonElement>("[data-move]")
  .forEach(
    (button) =>
      (button.onclick = (event) =>
        void applyAlgorithm(
          button.dataset.move! +
            ((event as MouseEvent).shiftKey ? "'" : store.getModifier()),
        )),
  );
function setModifier(value: "'" | "2") {
  store.toggleModifier(value);
}
$("prime").onclick = () => setModifier("'");
$("double").onclick = () => setModifier("2");
$("undo").onclick = () => {
  if (store.undo()) message();
};
$("redo").onclick = () => {
  if (store.redo()) message();
};
$("reset").onclick = () => replace(SOLVED);
const viewPresets = ["iso", "front", "top", "right"] as const;
function updateActivePreset(presetName: (typeof viewPresets)[number]) {
  viewPresets.forEach((name) => {
    $(`view-preset-${name}`).classList.toggle("active", name === presetName);
  });
}
viewPresets.forEach((preset) => {
  $(`view-preset-${preset}`).onclick = () => {
    scene?.setViewPreset(preset);
    updateActivePreset(preset);
  };
});
$("view-reset").onclick = () => {
  scene?.resetView();
  updateActivePreset("iso");
};
$("first").onclick = () => {
  stop();
  void seek(0, false);
};
$("prev").onclick = () => {
  stop();
  void seek(store.getStep() - 1);
};
$("next").onclick = () => {
  stop();
  void seek(store.getStep() + 1);
};
$("last").onclick = () => {
  const solution = store.getSolution();
  if (solution) {
    stop();
    void seek(solution.moves.length, false);
  }
};
$("play").onclick = () => void play();
$<HTMLInputElement>("timeline").oninput = () => {
  stop();
  void seek(Number($<HTMLInputElement>("timeline").value), false);
};
reduced.onchange = () => {
  stop();
  persist();
  refresh();
};
$<HTMLSelectElement>("speed").onchange = persist;
$("copy").onclick = async () => {
  const solution = store.getSolution();
  if (solution)
    try {
      await navigator.clipboard.writeText(solution.moves.join(" "));
      message("解法をコピーしました。");
    } catch {
      message("コピーできませんでした。解法を選択してコピーしてください。");
    }
};
$("solution-close").onclick = () => {
  stop();
  store.setSolution(undefined);
  persist();
  refresh();
};
$("help").onclick = () => {
  stop();
  refresh();
  $<HTMLDialogElement>("help-dialog").showModal();
};
$("help-close").onclick = () => $<HTMLDialogElement>("help-dialog").close();
document.querySelectorAll<HTMLButtonElement>("[data-tab]").forEach((button) => {
  button.onclick = () => {
    document
      .querySelectorAll<HTMLButtonElement>("[data-tab]")
      .forEach((tab) => {
        const active = button === tab;
        tab.setAttribute("aria-selected", String(active));
        tab.tabIndex = active ? 0 : -1;
        $(`${tab.dataset.tab}-panel`).hidden = !active;
      });
  };
  button.onkeydown = (event) => {
    if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
      event.preventDefault();
      event.stopPropagation();
      const tabs = Array.from(
        document.querySelectorAll<HTMLButtonElement>("[data-tab]"),
      );
      const step = event.key === "ArrowRight" ? 1 : tabs.length - 1;
      const target = tabs[(tabs.indexOf(button) + step) % tabs.length];
      target.click();
      target.focus();
    }
  };
});
$("share-link").onclick = async () => {
  const snapshot = store.getSnapshot();
  const shareUrl = buildShareUrl(
    window.location.href,
    store.getState(),
    snapshot.centerTurns,
  );
  try {
    await navigator.clipboard.writeText(shareUrl);
    message("共有リンクをクリップボードにコピーしました。");
  } catch {
    message(`共有リンク: ${shareUrl}`);
  }
};
$("save").onclick = () => {
  const blob = createCubeJsonBlob(store.getSnapshot());
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "cube-studio.json";
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};
$("load").onclick = () => $<HTMLInputElement>("file").click();
let fileLoadRequestId = 0;
$<HTMLInputElement>("file").onchange = async () => {
  const file = $<HTMLInputElement>("file").files?.[0];
  if (!file) return;
  const requestId = ++fileLoadRequestId;
  const at = store.getRevision();
  try {
    if (file.size > 65536) {
      throw new Error("ファイルは64KB以内にしてください。");
    }
    const content = await file.text();
    if (requestId !== fileLoadRequestId) return;
    const parsed = validateAndParseCubeJson(content, file.size);
    validate(parsed.state);
    if (at !== store.getRevision())
      throw new Error(
        "読込中にキューブが変更されました。もう一度読み込んでください。",
      );
    replace(
      parsed.state,
      true,
      centersFromInput(parsed.state, parsed.centerTurns),
    );
  } catch (error) {
    if (requestId === fileLoadRequestId) message(String(error));
  } finally {
    if (requestId === fileLoadRequestId) $<HTMLInputElement>("file").value = "";
  }
};
setupKeyboardShortcuts({
  isReady: () => mainReady,
  getModifier: () => store.getModifier(),
  onMove: (move) => {
    void applyAlgorithm(move);
  },
  onPlay: () => {
    void play();
  },
  onStop: () => stop(),
  onSeek: (step, animate) => {
    void seek(step, animate);
  },
  getCurrentStep: () => store.getStep(),
  getSolutionLength: () => store.getSolution()?.moves.length,
  onSuspend: () => {
    stop();
    persist();
    refresh();
  },
});
refresh();
async function start(forceError = false) {
  try {
    if (forceError) throw new Error("forced start error");
    if (!mainReady) {
      await init({ module_or_path: wasmUrl });
      window.cube_studio = cubeStudio;
      mainReady = true;
    }
    try {
      const raw = localStorage.getItem(storageKey);
      if (raw) {
        const data = JSON.parse(raw);
        if (data.version !== 1 || typeof data.state !== "string")
          throw new Error("format");
        validate(data.state);
        if (typeof data.reducedMotion === "boolean")
          reduced.checked = data.reducedMotion;
        if (["1000", "500", "250"].includes(data.speed))
          $<HTMLSelectElement>("speed").value = data.speed;
        if (
          ["kociemba", "cfop", "thistlethwaite", "korf"].includes(
            data.solverAlgorithm,
          )
        )
          solverAlgo.value = data.solverAlgorithm;
        const restoredCenters = centersFromInput(data.state, data.centerTurns);
        store.replace(data.state, false, restoredCenters);
      }
    } catch {
      message("保存状態を復元できなかったため、完成状態から開始しました。");
    }

    const parsedParams = parseUrlParams(window.location.search);
    if (parsedParams.solver) {
      solverAlgo.value = parsedParams.solver;
    }
    if (parsedParams.state) {
      try {
        validate(parsedParams.state);
        const restoredCenters = parsedParams.centers
          ? centersFromInput(parsedParams.state, parsedParams.centers)
          : automaticCenters(parsedParams.state);
        store.replace(parsedParams.state, false, restoredCenters);
      } catch {
        // 不正な state は無視
      }
    } else if (parsedParams.alg) {
      try {
        const cleanAlg = parsedParams.alg.replace(/[_+]/g, " ").trim();
        const result: ResultData = JSON.parse(apply_moves(SOLVED, cleanAlg));
        validate(result.state);
        const nextCenters = rotateCenters([0, 0, 0, 0, 0, 0], result.moves);
        store.replace(result.state, false, nextCenters);
      } catch {
        // 不正な alg は無視（保存状態の局面を維持）
      }
    }
    restoring = false;

    solver = new SolverClient((status, text) => {
      engineError = status === "error";
      $("engine-status").textContent =
        status === "ready"
          ? "● READY"
          : status === "error"
            ? "読み込み失敗"
            : "準備中";
      $("engine-status").classList.toggle("ready", status === "ready");
      if (!solving)
        $("solver-note").textContent =
          status === "ready" ? "ブラウザ内で計算 · 通常5秒以内" : text;
      if (engineError) message(text);
      queueMicrotask(refresh);
    });
    refresh();
  } catch {
    message(
      "アプリを読み込めませんでした。接続を確認し、ページを再読み込みしてください。",
    );
    $("engine-status").textContent = "読み込み失敗";
  }
}

// プリセット状態の定義
const presets = [
  { id: "solved", label: "完成状態", emoji: "✅" },
  { id: "superflip", label: "スーパーフリップ", emoji: "⚡" },
  { id: "easy-5-moves", label: "簡単（3手）", emoji: "🟢" },
  { id: "t-perm", label: "T-Permutation", emoji: "🔄" },
  { id: "seed-1-scramble", label: "ランダム（seed=1）", emoji: "🎲" },
];

// プリセットボタンを生成
async function initializePresets() {
  const presetButtons = $("preset-buttons");
  const presetStatus = $("preset-status");
  let presetRequestId = 0;

  try {
    for (const preset of presets) {
      const button = document.createElement("button");
      button.className = "secondary";
      button.textContent = `${preset.emoji} ${preset.label}`;
      button.disabled = !mainReady;
      button.onclick = async () => {
        if (!mainReady) return;
        stop();
        const requestId = ++presetRequestId;
        const initialRevision = store.getRevision();
        try {
          presetStatus.textContent = "読み込み中…";
          const baseUrl = import.meta.env.BASE_URL.endsWith("/")
            ? import.meta.env.BASE_URL
            : `${import.meta.env.BASE_URL}/`;
          const response = await fetch(`${baseUrl}cubes/${preset.id}.json`);
          if (!response.ok) {
            throw new Error(
              `HTTP ${response.status}: ファイルが見つかりません (${response.url})`,
            );
          }
          const data = await response.json();

          // 後から別のプリセットリクエストが発行されていた場合は破棄
          if (requestId !== presetRequestId) {
            return;
          }

          // 取得中にユーザーが手動でキューブを操作していた場合は上書きを防止
          if (store.getRevision() !== initialRevision) {
            presetStatus.textContent = `⚠️ 読み込み中にキューブが操作されたため、現在の操作を優先しました`;
            return;
          }

          // scramble_seed がある場合は WASM の scramble() で生成
          if (typeof data.scramble_seed === "number") {
            try {
              const algorithm = scramble(data.scramble_seed);
              const result: ResultData = JSON.parse(
                apply_moves(SOLVED, algorithm),
              );
              replace(
                result.state,
                true,
                rotateCenters([0, 0, 0, 0, 0, 0], result.moves),
              );
              $("scramble-text").textContent = algorithm;
            } catch (scrambleError) {
              throw new Error(
                `シードスクランブル実行エラー: ${scrambleError instanceof Error ? scrambleError.message : String(scrambleError)}`,
              );
            }
          }
          // scramble 文字列がある場合
          else if (typeof data.scramble === "string" && data.scramble.trim()) {
            try {
              const cleanedScramble = data.scramble.replace(
                /(\b[URFDLB])\s+(\d|')/g,
                "$1$2",
              );
              const result: ResultData = JSON.parse(
                apply_moves(SOLVED, cleanedScramble),
              );
              replace(
                result.state,
                true,
                rotateCenters([0, 0, 0, 0, 0, 0], result.moves),
              );
              $("scramble-text").textContent = cleanedScramble;
            } catch (scrambleError) {
              throw new Error(
                `スクランブル実行エラー: ${scrambleError instanceof Error ? scrambleError.message : String(scrambleError)}`,
              );
            }
          }
          // state 文字列がある場合
          else if (typeof data.state === "string" && data.state.trim()) {
            validate(data.state);
            replace(
              data.state,
              true,
              centersFromInput(data.state, data.centerTurns),
            );
          } else {
            throw new Error(
              `無効なデータ形式: state=${data.state}, scramble=${data.scramble}, scramble_seed=${data.scramble_seed}`,
            );
          }

          presetStatus.textContent = `✓ ${preset.label} を読み込みました`;
        } catch (error) {
          const errorMsg =
            error instanceof Error ? error.message : String(error);
          presetStatus.textContent = `❌ 読み込み失敗 (${errorMsg})`;
        }
      };
      presetButtons.append(button);
    }
    presetStatus.textContent =
      "プリセットから選択してください（下のタブから 📌 プリセット）";
  } catch (error) {
    presetStatus.textContent = `初期化エラー: ${String(error)}`;
  }
}

// プリセット初期化を開始
initializePresets();

void start();

if (
  import.meta.env.DEV &&
  typeof window !== "undefined" &&
  Boolean(navigator.webdriver)
) {
  (window as any).__cube_main_debug__ = {
    promptReloadForUpdate,
    cancelSearch,
    fallback,
    solve,
    play,
    stop,
    seek,
    applyAlgorithm,
    refresh,
    store,
    persist,
    replace,
    getEditor,
    getCamera,
    initializePresets,
    start,
    setEngineError: (val: boolean) => {
      engineError = val;
    },
    setSolving: (val: boolean) => {
      solving = val;
    },
    setPlaying: (val: boolean) => {
      playing = val;
    },
    setMainReady: (val: boolean) => {
      mainReady = val;
    },
    setScene: (val: any) => {
      scene = val;
    },
    resetInstances: () => {
      editorInstance = undefined;
      cameraInstance = undefined;
    },
    getSolver: () => solver,
    initScene: (forceError = false) => initScene(forceError),
  };
}
