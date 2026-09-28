import { SOLVED, type ResultData } from "./model";
import { automaticCenters, centerTurns, turnsToCenters } from "./centers";

export interface CubeSnapshot {
  state: string;
  centerTurns: number[];
}

export type StoreEventType =
  | "replace"
  | "undo"
  | "redo"
  | "modifier"
  | "solution"
  | "step"
  | "seek"
  | "algorithm";

export interface StoreEventDetail {
  type: StoreEventType;
}

export type StoreListener = (store: CubeStore, event: StoreEventDetail) => void;

export class CubeStore {
  private state: string = SOLVED;
  private centerRotations: number[] = [0, 0, 0, 0, 0, 0];
  private revision: number = 0;
  private solution?: ResultData = undefined;
  private step: number = 0;
  private modifier: string = "";
  private history: CubeSnapshot[] = [];
  private future: CubeSnapshot[] = [];
  private listeners: Set<StoreListener> = new Set();

  private baseSnapshot?: CubeSnapshot = undefined;

  getState(): string {
    return this.state;
  }

  getCenterRotations(): number[] {
    return [...this.centerRotations];
  }

  getCenterTurns(): number[] {
    return centerTurns(this.centerRotations);
  }

  getRevision(): number {
    return this.revision;
  }

  getSolution(): ResultData | undefined {
    return this.solution;
  }

  getStep(): number {
    return this.step;
  }

  getModifier(): string {
    return this.modifier;
  }

  private isSameSnapshot(a: CubeSnapshot, b: CubeSnapshot): boolean {
    return (
      a.state === b.state &&
      a.centerTurns.every((val, idx) => val === b.centerTurns[idx])
    );
  }

  canUndo(): boolean {
    if (this.solution && this.step > 0 && this.baseSnapshot) {
      return !this.isSameSnapshot(this.getSnapshot(), this.baseSnapshot);
    }
    if (this.history.length === 0) return false;
    const top = this.history[this.history.length - 1];
    return !this.isSameSnapshot(this.getSnapshot(), top);
  }

  canRedo(): boolean {
    return this.future.length > 0;
  }

  getSnapshot(): CubeSnapshot {
    return {
      state: this.state,
      centerTurns: this.getCenterTurns(),
    };
  }

  private pushHistory(snapshot: CubeSnapshot): void {
    this.history.push(snapshot);
    if (this.history.length > 200) this.history.shift();
    this.future = [];
  }

  private commitBaseSnapshotIfPreviewing(): void {
    if (
      this.baseSnapshot &&
      !this.isSameSnapshot(this.getSnapshot(), this.baseSnapshot)
    ) {
      this.pushHistory(this.baseSnapshot);
    }
  }

  replace(next: string, record = true, centers = automaticCenters(next)): void {
    const nextTurns = centerTurns(centers);
    if (record) {
      this.commitBaseSnapshotIfPreviewing();
      if (
        this.state !== next ||
        this.getCenterTurns().some((t, i) => t !== nextTurns[i])
      ) {
        this.pushHistory(this.getSnapshot());
      }
    }
    this.state = next;
    this.centerRotations = [...centers];
    this.revision++;
    this.solution = undefined;
    this.baseSnapshot = undefined;
    this.step = 0;
    this.notify("replace");
  }

  undo(): boolean {
    if (this.solution && this.step > 0 && this.baseSnapshot) {
      if (this.isSameSnapshot(this.getSnapshot(), this.baseSnapshot)) {
        return false;
      }
      this.future.push(this.getSnapshot());
      this.state = this.baseSnapshot.state;
      this.centerRotations = turnsToCenters(this.baseSnapshot.centerTurns);
      this.revision++;
      this.solution = undefined;
      this.baseSnapshot = undefined;
      this.step = 0;
      this.notify("undo");
      return true;
    }
    const prev = this.history.pop();
    if (!prev) return false;
    this.future.push(this.getSnapshot());
    this.state = prev.state;
    this.centerRotations = turnsToCenters(prev.centerTurns);
    this.revision++;
    this.solution = undefined;
    this.baseSnapshot = undefined;
    this.step = 0;
    this.notify("undo");
    return true;
  }

  redo(): boolean {
    const next = this.future.pop();
    if (!next) return false;
    this.history.push(this.getSnapshot());
    this.state = next.state;
    this.centerRotations = turnsToCenters(next.centerTurns);
    this.revision++;
    this.solution = undefined;
    this.baseSnapshot = undefined;
    this.step = 0;
    this.notify("redo");
    return true;
  }

  setModifier(value: string): void {
    if (this.modifier === value) return;
    this.modifier = value;
    this.notify("modifier");
  }

  toggleModifier(value: "'" | "2"): void {
    this.modifier = this.modifier === value ? "" : value;
    this.notify("modifier");
  }

  getBaseSnapshot(): CubeSnapshot | undefined {
    return this.baseSnapshot ? { ...this.baseSnapshot } : undefined;
  }

  restoreBaseSnapshot(): boolean {
    if (this.baseSnapshot && this.solution) {
      this.state = this.baseSnapshot.state;
      this.centerRotations = turnsToCenters(this.baseSnapshot.centerTurns);
      this.step = 0;
      this.solution = undefined;
      this.baseSnapshot = undefined;
      this.future = [];
      this.revision++;
      this.notify("solution");
      return true;
    }
    return false;
  }

  setSolution(solution: ResultData | undefined): void {
    this.commitBaseSnapshotIfPreviewing();
    this.solution = solution;
    this.baseSnapshot = solution ? this.getSnapshot() : undefined;
    this.step = 0;
    this.notify("solution");
  }

  setStep(target: number): void {
    this.step = target;
    this.notify("step");
  }

  updateAfterSeek(
    nextState: string,
    nextCenterRotations: number[],
    step: number,
  ): void {
    this.state = nextState;
    this.centerRotations = [...nextCenterRotations];
    this.step = step;
    this.revision++;
    this.notify("seek");
  }

  applyAlgorithmResult(
    nextState: string,
    nextCenters: number[],
    record = true,
  ): void {
    if (record) {
      this.commitBaseSnapshotIfPreviewing();
      if (
        this.state !== nextState ||
        nextCenters.some((angle, i) => angle !== this.centerRotations[i])
      ) {
        this.pushHistory(this.getSnapshot());
      }
    }
    this.state = nextState;
    this.centerRotations = [...nextCenters];
    this.revision++;
    this.solution = undefined;
    this.baseSnapshot = undefined;
    this.step = 0;
    this.notify("algorithm");
  }

  subscribe(listener: StoreListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(type: StoreEventType): void {
    const event: StoreEventDetail = { type };
    for (const listener of this.listeners) {
      listener(this, event);
    }
  }
}
