// docs/review-evidence/acdb7f4/probe_kociemba_phase.mjs
// Kociemba のフェーズ境界情報欠落と、フロントエンドヒューリスティックによる誤判定実証
import init, { solve_with_algorithm, scramble, apply_moves } from "../../../pkg/cube_studio.js";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const G1_MOVES = new Set([
  "U", "U'", "U2", "D", "D'", "D2", "R2", "L2", "F2", "B2"
]);

// triggers.ts の現在のヒューリスティック
function guessPhase2Start(moves) {
  let phase2StartIndex = moves.length;
  for (let i = moves.length - 1; i >= 0; i--) {
    if (G1_MOVES.has(moves[i])) {
      phase2StartIndex = i;
    } else {
      break;
    }
  }
  return phase2StartIndex;
}

async function run() {
  console.log("=== Probe: Kociemba Phase Estimation Fallacy ===");
  const wasmPath = path.resolve(__dirname, "../../../pkg/cube_studio_bg.wasm");
  const wasmBuffer = fs.readFileSync(wasmPath);
  await init({ module_or_path: wasmBuffer });

  const SOLVED = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

  // 100 個のシードでスクランブルを解き、ヒューリスティックの破綻（Phase 1 手の巻き込み）を検証
  let evaluated = 0;
  let falsePhase2Count = 0;
  let examples = [];

  for (let seed = 1; seed <= 50; seed++) {
    const sc = scramble(seed);
    const scrRes = JSON.parse(apply_moves(SOLVED, sc));
    const solJson = solve_with_algorithm(scrRes.state, 5000, false, undefined, "kociemba");
    const sol = JSON.parse(solJson);

    evaluated++;
    const moves = sol.moves;
    if (moves.length === 0) continue;

    // phases が返されているか確認
    if (sol.phases && sol.phases.length > 0) {
      console.log(`[Seed ${seed}] phases が存在します:`, sol.phases);
      continue;
    }

    // G1 (Phase 1 ゴール) に達した時点のインデックスを、WASM で 1 手ずつ適用して数学的に正確に判定
    // G1 群の条件: 全エッジの向き(eo)が0、かつ UDスライスエッジ(FR, FL, BL, BR)が中層にある
    let truePhase1End = -1;
    let currState = scrRes.state;
    for (let i = 0; i < moves.length; i++) {
      currState = JSON.parse(apply_moves(currState, moves[i])).state;
      // 以降の手がすべて G1_MOVES か確認
      const remainingMoves = moves.slice(i + 1);
      const allG1 = remainingMoves.every(m => G1_MOVES.has(m));
      if (allG1) {
        // 残り手がすべて G1_MOVES で、かつここから最後まで解けるなら、ここが理論上の最短 Phase 1 終了点
        truePhase1End = i + 1;
        break;
      }
    }

    const guessedIndex = guessPhase2Start(moves);

    // ヒューリスティック判定と真の Phase 1 終了点が食い違うか
    // 特に、Phase 1 の遷移手（moves[truePhase1End - 1]）が G1_MOVES であった場合、
    // ヒューリスティックはさらに手前まで遡ってしまい、Phase 1 の手を Phase 2 と誤判定する
    if (guessedIndex < truePhase1End) {
      falsePhase2Count++;
      if (examples.length < 3) {
        examples.push({
          seed,
          moves: moves.join(" "),
          totalMoves: moves.length,
          truePhase1End,
          guessedIndex,
          misclassifiedMove: moves[guessedIndex],
        });
      }
    }
  }

  console.log(`\n評価局面数: ${evaluated} 件`);
  console.log(`ヒューリスティックによる Phase 2 誤判定発生率: ${falsePhase2Count} / ${evaluated} (${((falsePhase2Count / evaluated) * 100).toFixed(1)}%)`);

  for (const ex of examples) {
    console.log(`\n【誤判定の具体例 (Seed ${ex.seed})】`);
    console.log(`  全手順 (${ex.totalMoves}手): ${ex.moves}`);
    console.log(`  数学的に正しい Phase 2 開始位置: インデックス ${ex.truePhase1End} (${ex.moves.split(" ")[ex.truePhase1End]})`);
    console.log(`  triggers.ts が推定した Phase 2 開始位置: インデックス ${ex.guessedIndex} (${ex.moves.split(" ")[ex.guessedIndex]})`);
    console.log(`  => インデックス ${ex.guessedIndex} の手 '${ex.misclassifiedMove}' は Phase 1 (G0→G1 への縮約) の手ですが、UI 上では「Phase 2: 解決」と虚偽表示されます。`);
  }
}

run().catch(console.error);
