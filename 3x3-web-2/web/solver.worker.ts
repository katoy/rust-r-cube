import init, {
  initialize,
  solve,
  solve_with_orientation,
} from "../pkg/cube_studio";
import wasmUrl from "../pkg/cube_studio_bg.wasm?url";
import type { Request, Reply } from "./model";
const send = (reply: Reply) => self.postMessage(reply);
async function start() {
  const before = performance.now();
  try {
    await init({ module_or_path: wasmUrl });
    initialize();
    self.onmessage = ({ data }: MessageEvent<Request>) => {
      try {
        const includeOrientation = data.includeOrientation !== false;
        const centersStr =
          includeOrientation && data.centerRotations
            ? data.centerRotations
                .map((r) => Math.round(r / (Math.PI / 2)).toString())
                .join(",")
            : undefined;
        const result = solve_with_orientation(
          data.state,
          data.budget,
          includeOrientation,
          centersStr,
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
