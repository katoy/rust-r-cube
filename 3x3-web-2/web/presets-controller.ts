import { SOLVED, type CubeType, type ResultData } from "./model";
import { SOLVED_2X2, PRESETS_2X2 } from "./model-2x2";
import { centersFromInput, rotateCenters } from "./centers";
import type { AppStateMachine } from "./app-state";
import type { CubeStore } from "./cube-store";

export const PRESETS_3X3 = [
  { id: "solved", label: "完成状態", emoji: "✅" },
  { id: "superflip", label: "スーパーフリップ", emoji: "⚡" },
  { id: "easy-5-moves", label: "簡単（3手）", emoji: "🟢" },
  { id: "t-perm", label: "T-Permutation", emoji: "🔄" },
  { id: "seed-1-scramble", label: "ランダム（seed=1）", emoji: "🎲" },
] as const;

export interface PresetData {
  scramble_seed?: number;
  scramble?: string;
  state?: string;
  centerTurns?: unknown;
}

export interface PresetsControllerDeps {
  store: CubeStore;
  appState: AppStateMachine;
  isMainReady: () => boolean;
  stop: () => void;
  cancelSearch: () => void;
  replace: (next: string, record?: boolean, centers?: number[]) => void;
  applyMoves: (state: string, moves: string) => string;
  validateState: (state: string) => boolean;
  scrambleMoves: (seed: number) => string;
  getScrambleTextEl: () => HTMLElement;
  getPresetButtonsEl: () => HTMLElement | null;
  getPresetStatusEl: () => HTMLElement | null;
}

export function createPresetsController(deps: PresetsControllerDeps) {
  let presetRequestId = 0;

  function applyPresetMoves(algorithm: string) {
    const result: ResultData = JSON.parse(deps.applyMoves(SOLVED, algorithm));
    deps.replace(
      result.state,
      true,
      rotateCenters([0, 0, 0, 0, 0, 0], result.moves),
    );
    deps.getScrambleTextEl().textContent = algorithm;
  }

  function applyPresetData(data: PresetData) {
    if (typeof data.scramble_seed === "number") {
      try {
        applyPresetMoves(deps.scrambleMoves(data.scramble_seed));
      } catch (e) {
        throw new Error(
          `シードスクランブル実行エラー: ${e instanceof Error ? e.message : String(e)}`,
        );
      }
    } else if (typeof data.scramble === "string" && data.scramble.trim()) {
      try {
        const cleaned = data.scramble.replace(/(\b[URFDLB])\s+(\d|')/g, "$1$2");
        applyPresetMoves(cleaned);
      } catch (e) {
        throw new Error(
          `スクランブル実行エラー: ${e instanceof Error ? e.message : String(e)}`,
        );
      }
    } else if (typeof data.state === "string" && data.state.trim()) {
      deps.validateState(data.state);
      deps.replace(
        data.state,
        true,
        centersFromInput(data.state, data.centerTurns),
      );
    } else {
      throw new Error(
        `無効なデータ形式: state=${data.state}, scramble=${data.scramble}, scramble_seed=${data.scramble_seed}`,
      );
    }
  }

  async function loadPreset(
    preset: (typeof PRESETS_3X3)[number],
    requestId: number,
    initialRevision: number,
  ) {
    const presetStatus = deps.getPresetStatusEl();
    if (!presetStatus) return;
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
    const data: PresetData = await response.json();

    if (requestId !== presetRequestId) return;

    if (
      !deps.appState.getState().canModifyCube() ||
      deps.store.getRevision() !== initialRevision
    ) {
      presetStatus.textContent =
        "⚠️ 読み込み中にキューブが操作されたため、現在の操作を優先しました";
      return;
    }

    deps.appState.runCubeMutation(() => {
      applyPresetData(data);
    });
    presetStatus.textContent = `✓ ${preset.label} を読み込みました`;
  }

  function renderPresetButtons(type: CubeType) {
    const presetButtons = deps.getPresetButtonsEl();
    const presetStatus = deps.getPresetStatusEl();
    if (!presetButtons || !presetStatus) return;
    presetButtons.replaceChildren();

    if (type === "2x2") {
      try {
        for (const preset of PRESETS_2X2) {
          const button = document.createElement("button");
          button.className = "secondary";
          button.textContent = `${preset.emoji} ${preset.label}`;
          button.disabled = !deps.isMainReady();
          button.onclick = () => {
            if (
              !deps.isMainReady() ||
              !deps.appState.getState().canModifyCube()
            )
              return;
            deps.stop();
            deps.cancelSearch();
            deps.appState.runCubeMutation(() => {
              if (preset.scramble) {
                const result: ResultData = JSON.parse(
                  deps.applyMoves(SOLVED_2X2, preset.scramble),
                );
                deps.replace(result.state, true, [0, 0, 0, 0, 0, 0]);
                deps.getScrambleTextEl().textContent = preset.scramble;
              } else if (preset.state) {
                deps.replace(preset.state, true, [0, 0, 0, 0, 0, 0]);
                deps.getScrambleTextEl().textContent = "";
              }
            });
            presetStatus.textContent = `✓ ${preset.label} を読み込みました`;
          };
          presetButtons.append(button);
        }
        presetStatus.textContent =
          "プリセットから選択してください（下のタブから 📌 プリセット）";
      } catch (error) {
        presetStatus.textContent = `初期化エラー: ${String(error)}`;
      }
    } else {
      try {
        for (const preset of PRESETS_3X3) {
          const button = document.createElement("button");
          button.className = "secondary";
          button.textContent = `${preset.emoji} ${preset.label}`;
          button.disabled = !deps.isMainReady();
          button.onclick = async () => {
            if (
              !deps.isMainReady() ||
              !deps.appState.getState().canModifyCube()
            )
              return;
            deps.stop();
            deps.cancelSearch();
            const requestId = ++presetRequestId;
            const initialRevision = deps.store.getRevision();
            try {
              await loadPreset(preset, requestId, initialRevision);
            } catch (error) {
              if (requestId === presetRequestId) {
                const errorMsg =
                  error instanceof Error ? error.message : String(error);
                presetStatus.textContent = `❌ 読み込み失敗 (${errorMsg})`;
              }
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
  }

  async function initializePresets() {
    renderPresetButtons(deps.store.getCubeType());
  }

  return { renderPresetButtons, initializePresets };
}
