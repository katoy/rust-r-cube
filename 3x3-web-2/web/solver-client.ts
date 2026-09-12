import type { Reply, ResultData } from "./model";
export class SolverClient {
  private worker?: Worker;
  private generation = 0;
  private nextId = 0;
  private timer = 0;
  private pending?: {
    id: number;
    revision: number;
    resolve: (data: ResultData) => void;
    reject: (error: Error) => void;
  };
  ready = false;
  constructor(
    private status: (
      status: "loading" | "ready" | "error",
      message: string,
    ) => void,
  ) {
    this.restart();
  }
  restart() {
    this.disposeRequest();
    this.worker?.terminate();
    this.ready = false;
    const generation = ++this.generation;
    this.status("loading", "エンジンを準備中");
    this.worker = new Worker(new URL("./solver.worker.ts", import.meta.url), {
      type: "module",
    });
    this.timer = window.setTimeout(
      () =>
        this.fail(
          "エンジンの読み込みが時間切れになりました。再試行してください。",
        ),
      20000,
    );
    this.worker.onerror = () => {
      if (generation === this.generation)
        this.fail("エンジンを起動できませんでした。再試行してください。");
    };
    this.worker.onmessage = ({ data }: MessageEvent<Reply>) => {
      if (generation !== this.generation) return;
      if (data.kind === "ready") {
        clearTimeout(this.timer);
        this.ready = true;
        this.status("ready", `準備完了 · ${Math.round(data.elapsed)} ms`);
      } else if (data.kind === "init-error") this.fail(data.error);
      else if (
        this.pending?.id === data.id &&
        this.pending.revision === data.revision
      ) {
        clearTimeout(this.timer);
        const pending = this.pending;
        this.pending = undefined;
        if (data.result) pending.resolve(data.result);
        else pending.reject(new Error(data.error || "探索に失敗しました。"));
      }
    };
  }
  solve(state: string, revision: number, budget: number, includeOrientation = true) {
    if (!this.ready)
      return Promise.reject(new Error("エンジンの準備完了をお待ちください。"));
    this.disposeRequest();
    return new Promise<ResultData>((resolve, reject) => {
      const id = ++this.nextId;
      this.pending = { id, revision, resolve, reject };
      this.timer = window.setTimeout(
        () =>
          this.fail(
            "探索時間の上限に達しました。エンジンを再起動してください。",
          ),
        budget + 1500,
      );
      this.worker!.postMessage({ kind: "solve", id, revision, state, budget, includeOrientation });
    });
  }
  cancel() {
    this.restart();
  }
  private disposeRequest() {
    clearTimeout(this.timer);
    this.pending?.reject(new Error("cancelled"));
    this.pending = undefined;
  }
  private fail(message: string) {
    this.disposeRequest();
    this.worker?.terminate();
    this.generation++;
    this.ready = false;
    this.status("error", message);
  }
}
