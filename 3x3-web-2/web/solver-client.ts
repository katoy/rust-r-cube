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
  private readyPromise?: Promise<void>;
  private readyResolve?: () => void;
  private readyReject?: (error: Error) => void;
  ready = false;
  constructor(
    private status: (
      status: "loading" | "ready" | "error",
      message: string,
      isInitError?: boolean,
    ) => void,
  ) {
    this.restart();
  }
  restart(): Promise<void> {
    this.disposeRequest();
    this.worker?.terminate();
    this.ready = false;
    const generation = ++this.generation;
    this.readyPromise = new Promise<void>((resolve, reject) => {
      this.readyResolve = resolve;
      this.readyReject = reject;
    });
    this.readyPromise.catch(() => {});
    this.status("loading", "エンジンを準備中");
    this.worker = new Worker(new URL("./solver.worker.ts", import.meta.url), {
      type: "module",
    });
    this.timer = window.setTimeout(
      () =>
        this.fail(
          "エンジンの読み込みが時間切れになりました。再試行してください。",
          true,
        ),
      20000,
    );
    this.worker.onerror = () => {
      if (generation === this.generation)
        this.fail("エンジンを起動できませんでした。再試行してください。", true);
    };
    this.worker.onmessage = ({ data }: MessageEvent<Reply>) => {
      if (generation !== this.generation) return;
      if (data.kind === "ready") {
        clearTimeout(this.timer);
        this.ready = true;
        this.readyResolve?.();
        this.status("ready", `準備完了 · ${Math.round(data.elapsed)} ms`);
      } else if (data.kind === "init-error") {
        this.fail(data.error, true);
      } else if (
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
    return this.readyPromise;
  }
  waitForReady(): Promise<void> {
    if (this.ready) return Promise.resolve();
    if (this.readyPromise) return this.readyPromise;
    return this.restart();
  }
  solve(
    state: string,
    revision: number,
    budget: number,
    includeOrientation = true,
    centerRotations?: number[],
    algorithm?: import("./model").SolverAlgorithm,
  ) {
    if (!this.ready)
      return Promise.reject(new Error("エンジンの準備完了をお待ちください。"));
    if (this.pending) {
      this.cancel();
      return Promise.reject(new Error("cancelled"));
    }
    this.disposeRequest();
    return new Promise<ResultData>((resolve, reject) => {
      const id = ++this.nextId;
      this.pending = { id, revision, resolve, reject };
      this.timer = window.setTimeout(
        () =>
          this.fail(
            "探索時間の上限に達しました。エンジンを再起動してください。",
          ),
        Math.max(budget * 1.5, budget + 4000),
      );
      this.worker!.postMessage({
        kind: "solve",
        id,
        revision,
        state,
        budget,
        includeOrientation,
        centerRotations,
        algorithm,
      });
    });
  }

  cancel() {
    if (this.pending) {
      this.disposeRequest("cancelled");
      this.restart();
    }
  }
  private disposeRequest(reason = "cancelled") {
    clearTimeout(this.timer);
    this.pending?.reject(new Error(reason));
    this.pending = undefined;
  }
  private fail(message: string, isInit = false) {
    this.disposeRequest(message);
    this.worker?.terminate();
    this.generation++;
    this.ready = false;
    this.readyReject?.(new Error(message));
    this.readyPromise = undefined;
    this.readyResolve = undefined;
    this.readyReject = undefined;
    this.status("error", message, isInit);
  }
}
