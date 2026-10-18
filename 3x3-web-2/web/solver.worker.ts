import init, {
  apply_moves,
  initialize,
  solve_with_algorithm,
} from "../pkg/cube_studio";
import wasmUrl from "../pkg/cube_studio_bg.wasm?url";
import type { Request, Reply, ResultData } from "./model";
import { isUniformSolved2x2, relabel2x2ForDbl } from "./model-2x2";

const send = (reply: Reply) => self.postMessage(reply);

async function start() {
  const before = performance.now();
  try {
    await init({ module_or_path: wasmUrl });
    initialize();
    self.onmessage = ({ data }: MessageEvent<Request>) => {
      try {
        const includeOrientation = data.includeOrientation !== false;
        const is2x2 = data.state.length === 24;
        const algo = data.algorithm || (is2x2 ? "optimal" : "kociemba");

        if (is2x2 && !includeOrientation && isUniformSolved2x2(data.state)) {
          send({
            kind: "result",
            id: data.id,
            revision: data.revision,
            result: {
              state: data.state,
              moves: [],
              states: [data.state],
              elapsed_ms: 0,
              nodes: 0,
              algorithm: algo,
              phases: [],
            },
          });
          return;
        }

        if (
          is2x2 &&
          !includeOrientation &&
          (algo === "lbl" || algo === "ortega")
        ) {
          const relabeled = relabel2x2ForDbl(data.state);
          const rawJson = solve_with_algorithm(
            relabeled,
            data.budget,
            true,
            undefined,
            algo,
          );
          const parsed: ResultData = JSON.parse(rawJson);
          if (parsed.moves.length > 0) {
            const applied: ResultData = JSON.parse(
              apply_moves(data.state, parsed.moves.join(" ")),
            );
            parsed.state = applied.state;
            parsed.states = applied.states;
          } else {
            parsed.state = data.state;
            parsed.states = [data.state];
          }
          send({
            kind: "result",
            id: data.id,
            revision: data.revision,
            result: parsed,
          });
          return;
        }

        const centersStr =
          includeOrientation && data.centerRotations
            ? data.centerRotations
                .map((r) =>
                  (((Math.round(r / (Math.PI / 2)) % 4) + 4) % 4).toString(),
                )
                .join(",")
            : undefined;
        const result = solve_with_algorithm(
          data.state,
          data.budget,
          includeOrientation,
          centersStr,
          algo,
        );

        send({
          kind: "result",
          id: data.id,
          revision: data.revision,
          result: JSON.parse(result),
        });
      } catch (error) {
        send({
          kind: "result",
          id: data.id,
          revision: data.revision,
          error: String(error),
        });
      }
    };
    send({ kind: "ready", elapsed: performance.now() - before });
  } catch (error) {
    send({
      kind: "init-error",
      error: `エンジンを読み込めませんでした。${String(error)}`,
    });
  }
}
void start();
