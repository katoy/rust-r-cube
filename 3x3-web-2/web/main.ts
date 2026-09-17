import "./style.css";
import init, * as cubeStudio from "../pkg/cube_studio";
const { apply_moves, validate, scramble } = cubeStudio;
import wasmUrl from "../pkg/cube_studio_bg.wasm?url";
import { SOLVED, FACES, inverse, instruction, type ResultData } from "./model";
import { mount, icon, net } from "./view";
import { CubeScene } from "./scene";
import { ColorEditor } from "./editor";
import { SolverClient } from "./solver-client";
import { TwoViewCamera } from "./camera";
import { CubeStore } from "./cube-store";
import {
  automaticCenters,
  centerTurns,
  centersFromInput,
  rotateCenters,
} from "./centers";

import { registerServiceWorker } from "./pwa";

const store = new CubeStore();
(window as any).cube_store = store;

mount();
registerServiceWorker();
const $ = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
const storageKey = "cube-studio-v1";
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
const reduced = $<HTMLInputElement>("reduced-motion");
reduced.checked = matchMedia("(prefers-reduced-motion: reduce)").matches;
const includeOrientation = $<HTMLInputElement>("include-orientation");
includeOrientation.checked = true;
function message(text = "") {
  $("message").textContent = text;
}
function stop() {
  playing = false;
  playbackRun++;
  motion++;
  scene?.finish();
  inMotion = false;
}
function cancelSearch() {
  if (solving) {
    solving = false;
    clearInterval(interval);
    solver?.cancel();
  }
}
function persist() {
  try {
    localStorage.setItem(
      storageKey,
      JSON.stringify({
        version: 1,
        ...store.getSnapshot(),
        reducedMotion: reduced.checked,
        speed: $<HTMLSelectElement>("speed").value,
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
function refresh() {
  const state = store.getState();
  const solution = store.getSolution();
  const step = store.getStep();
  const centerRotations = store.getCenterRotations();
  const next = solution?.moves[step] || "";
  if (scene) {
    scene.centerRotations = [...centerRotations];
    if (!inMotion) scene.show(state, next);
  }
  net($("fallback-net"), state, false, undefined, -1, store.getCenterTurns());
  $("cube-status").textContent =
    state === SOLVED
      ? store.getCenterTurns().some((t) => t !== 0)
        ? "色は完成・センターの向きあり"
        : "完成状態"
      : solution
        ? `${step} / ${solution.moves.length} 手`
        : "スクランブル状態";
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
      "[data-move],#scramble,#reset,#apply-algorithm,#edit-colors,#camera-colors,#save,#load",
    )
    .forEach((b) => (b.disabled = !mainReady));
  $("solution-empty").hidden = !!solution;
  $("solution-content").hidden = !solution;
  $<HTMLButtonElement>("copy").disabled = !solution;
  if (solution) {
    $("move-count").textContent = `${solution.moves.length} 手`;
    $("solve-time").textContent =
      `${solution.elapsed_ms < 1000 ? `${Math.round(solution.elapsed_ms)} ms` : `${(solution.elapsed_ms / 1000).toFixed(2)} 秒`} · 検証済み`;
    const list = $("move-list");
    list.replaceChildren();
    solution.moves.forEach((move, i) => {
      const button = document.createElement("button");
      button.className = `solution-move ${i < step ? "done" : ""} ${i === step ? "current" : ""}`;
      button.textContent = move;
      button.setAttribute("aria-label", `${i + 1}手目 ${move} の直後へ移動`);
      if (i === step) button.setAttribute("aria-current", "step");
      button.onclick = () => {
        stop();
        void seek(i + 1, false);
      };
      list.append(button);
    });
    $("next-symbol").textContent = next || "✓";
    $("next-instruction").textContent = next
      ? instruction(next)
      : "6面が揃いました。おつかれさまでした。";
    $("step-count").textContent = `${step} / ${solution.moves.length}`;
    $("play").innerHTML = icon(playing ? "pause" : "play");
    $("play").setAttribute("aria-label", playing ? "一時停止" : "自動再生");
    $<HTMLButtonElement>("play").disabled = solution.moves.length === 0;
    $<HTMLButtonElement>("prev").disabled = step === 0;
    $<HTMLButtonElement>("next").disabled = step === solution.moves.length;
    $<HTMLInputElement>("timeline").max = String(solution.moves.length);
    $<HTMLInputElement>("timeline").value = String(step);
  }
}
store.subscribe((_s, { type }) => {
  if (type === "modifier") {
    const mod = store.getModifier();
    $("prime").setAttribute("aria-pressed", String(mod === "'"));
    $("double").setAttribute("aria-pressed", String(mod === "2"));
    return;
  }
  if (type !== "solution") {
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
  document.querySelector<HTMLElement>(".gesture")!.hidden = true;
}
try {
  scene = new CubeScene($("scene"));
  (window as any).cube_scene = scene;
  $("scene").addEventListener("render-failed", fallback);
} catch {
  fallback();
}
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
  if (token === motion) {
    inMotion = false;
    refresh();
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
    );

    if (store.getRevision() !== at) return;
    store.setSolution(result);
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

const editor = new ColorEditor(
  (s) => validate(s),
  (s, centers) => replace(s, true, centers),
);
const camera = new TwoViewCamera((s) => {
  let centers = [0, 0, 0, 0, 0, 0];
  try {
    centers = automaticCenters(s);
  } catch {}
  editor.open(s, centers);
});
$("edit-colors").onclick = () => {
  stop();
  refresh();
  editor.open(store.getState(), store.getCenterRotations());
};
$("camera-colors").onclick = () => camera.open();
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
$("view-reset").onclick = () => scene?.resetView();
$("prev").onclick = () => {
  stop();
  void seek(store.getStep() - 1);
};
$("next").onclick = () => {
  stop();
  void seek(store.getStep() + 1);
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
      const tabs = Array.from(
        document.querySelectorAll<HTMLButtonElement>("[data-tab]"),
      );
      const target =
        tabs[(tabs.indexOf(button) + (event.key === "ArrowRight" ? 1 : 2)) % 3];
      target.click();
      target.focus();
    }
  };
});
$("save").onclick = () => {
  const blob = new Blob(
    [JSON.stringify({ version: 1, ...store.getSnapshot() }, null, 2)],
    {
      type: "application/json",
    },
  );
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "cube-studio.json";
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};
$("load").onclick = () => $<HTMLInputElement>("file").click();
$<HTMLInputElement>("file").onchange = async () => {
  const file = $<HTMLInputElement>("file").files?.[0];
  if (!file) return;
  const at = store.getRevision();
  try {
    if (file.size > 65536)
      throw new Error("ファイルは64KB以内にしてください。");
    const data: unknown = JSON.parse(await file.text());
    if (
      !data ||
      typeof data !== "object" ||
      !("version" in data) ||
      data.version !== 1 ||
      !("state" in data) ||
      typeof data.state !== "string"
    )
      throw new Error("Cube Studio v1 のJSONファイルを選んでください。");
    validate(data.state);
    if (at !== store.getRevision())
      throw new Error(
        "読込中にキューブが変更されました。もう一度読み込んでください。",
      );
    replace(
      data.state,
      true,
      centersFromInput(
        data.state,
        "centerTurns" in data ? data.centerTurns : undefined,
      ),
    );
  } catch (error) {
    message(String(error));
  } finally {
    $<HTMLInputElement>("file").value = "";
  }
};
document.addEventListener("keydown", (event) => {
  if (
    event.ctrlKey ||
    event.metaKey ||
    event.altKey ||
    document.querySelector("dialog[open]") ||
    event.target instanceof HTMLInputElement ||
    event.target instanceof HTMLTextAreaElement ||
    event.target instanceof HTMLSelectElement
  )
    return;
  if (!mainReady) return;
  const face = event.key.toUpperCase();
  if (FACES.includes(face) && face.length === 1) {
    event.preventDefault();
    if (!event.repeat)
      void applyAlgorithm(face + (event.shiftKey ? "'" : store.getModifier()));
  } else if (
    event.code === "Space" &&
    !(event.target instanceof HTMLButtonElement)
  ) {
    event.preventDefault();
    void play();
  } else if (event.key === "ArrowLeft") {
    event.preventDefault();
    stop();
    void seek(store.getStep() - 1);
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    stop();
    void seek(store.getStep() + 1);
  }
});
document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    stop();
    refresh();
  }
});
refresh();
async function start() {
  try {
    await init({ module_or_path: wasmUrl });
    (window as any).cube_studio = cubeStudio;
    mainReady = true;
    try {
      const raw = localStorage.getItem(storageKey);
      if (raw) {
        const data = JSON.parse(raw);
        if (data.version !== 1 || typeof data.state !== "string")
          throw new Error("format");
        validate(data.state);
        const restoredCenters = centersFromInput(data.state, data.centerTurns);
        store.replace(data.state, false, restoredCenters);
        if (typeof data.reducedMotion === "boolean")
          reduced.checked = data.reducedMotion;
        if (["1000", "500", "250"].includes(data.speed))
          $<HTMLSelectElement>("speed").value = data.speed;
      }
    } catch {
      message("保存状態を復元できなかったため、完成状態から開始しました。");
    }
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
  { id: "easy-5-moves", label: "簡単（5手）", emoji: "🟢" },
  { id: "t-perm", label: "T-Permutation", emoji: "🔄" },
  { id: "seed-1-scramble", label: "ランダム（seed=1）", emoji: "🎲" },
];

// プリセットボタンを生成
async function initializePresets() {
  const presetButtons = $("preset-buttons");
  const presetStatus = $("preset-status");

  try {
    for (const preset of presets) {
      const button = document.createElement("button");
      button.className = "secondary";
      button.textContent = `${preset.emoji} ${preset.label}`;
      button.onclick = async () => {
        try {
          presetStatus.textContent = "読み込み中…";
          const response = await fetch(`/cubes/${preset.id}.json`);
          if (!response.ok) {
            throw new Error(
              `HTTP ${response.status}: ファイルが見つかりません (${response.url})`,
            );
          }
          const data = await response.json();

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
