/**
 * アプリケーション状態管理 (State パターン & 有限状態機械)
 *
 * 状態の種類:
 * - Idle: 通常状態。キューブの編集・回転・探索開始・ファイル/プリセット読込などが可能。
 * - Solving: 探索実行中。キューブの変更や新規探索はすべて遮断され、キャンセルのみ受付。
 * - Previewing: 解法プレビュー中。解法ステップの再生・シーク・コピーが可能。
 *              盤面変更操作が行われた場合は自動的にプレビューを確定・終了して Idle へ復帰する。
 */

export type AppStateKind = "idle" | "solving" | "previewing";

export interface AppState {
  readonly kind: AppStateKind;

  /** キューブの盤面変更（回転、スクランブル、リセット、手順適用、外部読込など）が可能か */
  canModifyCube(): boolean;

  /** 探索（solve）の開始が可能か */
  canStartSolve(): boolean;

  /** 探索のキャンセルが可能か */
  canCancelSolve(): boolean;

  /** Undo / Redo が可能か */
  canUndoRedo(): boolean;

  /** 解法プレビューのシーク・再生が可能か */
  canSeekSolution(): boolean;

  /** 探索開始時の遷移先状態（不正遷移時は null） */
  startSolve(): AppState | null;

  /** 探索完了時の遷移先状態（不正遷移時は null） */
  finishSolve(hasSolution: boolean): AppState | null;

  /** 探索キャンセル時の遷移先状態（不正遷移時は null） */
  cancelSolve(): AppState | null;

  /** 解法パネルを閉じる時の遷移先状態（不正遷移時は null） */
  closeSolution(): AppState | null;

  /** 盤面変更操作の実行前フック（操作許可時は遷移先状態または this、不許可・遮断時は null） */
  beforeModifyCube(): AppState | null;
}

export class IdleState implements AppState {
  readonly kind = "idle" as const;

  canModifyCube(): boolean {
    return true;
  }
  canStartSolve(): boolean {
    return true;
  }
  canCancelSolve(): boolean {
    return false;
  }
  canUndoRedo(): boolean {
    return true;
  }
  canSeekSolution(): boolean {
    return false;
  }

  startSolve(): AppState | null {
    return new SolvingState();
  }

  finishSolve(_hasSolution: boolean): AppState | null {
    return null; // Idle 中に finishSolve は不正遷移
  }

  cancelSolve(): AppState | null {
    return null; // Idle 中に cancelSolve は不正遷移
  }

  closeSolution(): AppState | null {
    return null; // Idle 中に closeSolution は不正遷移
  }

  beforeModifyCube(): AppState | null {
    return this; // そのまま許可
  }
}

export class SolvingState implements AppState {
  readonly kind = "solving" as const;

  canModifyCube(): boolean {
    return false;
  }
  canStartSolve(): boolean {
    return false;
  }
  canCancelSolve(): boolean {
    return true;
  }
  canUndoRedo(): boolean {
    return false;
  }
  canSeekSolution(): boolean {
    return false;
  }

  startSolve(): AppState | null {
    return null; // 二重探索開始は遮断
  }

  finishSolve(hasSolution: boolean): AppState | null {
    return hasSolution ? new PreviewingState() : new IdleState();
  }

  cancelSolve(): AppState | null {
    return new IdleState();
  }

  closeSolution(): AppState | null {
    return null; // 探索中に解法パネルクローズは不正
  }

  beforeModifyCube(): AppState | null {
    return null; // 探索中は盤面変更を厳格に遮断
  }
}

export class PreviewingState implements AppState {
  readonly kind = "previewing" as const;

  canModifyCube(): boolean {
    return true; // プレビュー終了を伴って許可
  }
  canStartSolve(): boolean {
    return true; // プレビュー終了を伴って許可
  }
  canCancelSolve(): boolean {
    return false;
  }
  canUndoRedo(): boolean {
    return true;
  }
  canSeekSolution(): boolean {
    return true;
  }

  startSolve(): AppState | null {
    return new SolvingState();
  }

  finishSolve(_hasSolution: boolean): AppState | null {
    return null; // Previewing 中に finishSolve は不正遷移
  }

  cancelSolve(): AppState | null {
    return null; // Previewing 中に cancelSolve は不正遷移
  }

  closeSolution(): AppState | null {
    return new IdleState();
  }

  beforeModifyCube(): AppState | null {
    // プレビュー中に盤面変更を行う場合、プレビューを自動終了して Idle へ復帰
    return new IdleState();
  }
}

export type StateChangeListener = (
  newState: AppState,
  oldState: AppState,
) => void;

/**
 * アプリケーション状態マシン (有限状態機械)
 */
export class AppStateMachine {
  private currentState: AppState;
  private listeners: Set<StateChangeListener> = new Set();

  constructor(initialState: AppState = new IdleState()) {
    this.currentState = initialState;
  }

  getState(): AppState {
    return this.currentState;
  }

  get kind(): AppStateKind {
    return this.currentState.kind;
  }

  isIdle(): boolean {
    return this.currentState.kind === "idle";
  }

  isSolving(): boolean {
    return this.currentState.kind === "solving";
  }

  isPreviewing(): boolean {
    return this.currentState.kind === "previewing";
  }

  subscribe(listener: StateChangeListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private transitionTo(nextState: AppState): boolean {
    const prevState = this.currentState;
    if (prevState === nextState || prevState.kind === nextState.kind) {
      return false;
    }
    this.currentState = nextState;
    for (const listener of this.listeners) {
      listener(nextState, prevState);
    }
    return true;
  }

  /**
   * キューブ盤面変更操作の型安全な一括遮断・実行ラッパー
   *
   * @param action 同期操作。false を返す未変更操作や例外では状態を遷移しない
   * @param onExitPreview プレビュー状態から抜ける際の追加処理（例: stop() や 解法のクリア等）
   * @returns 操作結果。Solving 中など不許可時は undefined
   */
  runCubeMutation<T>(
    action: () => T,
    onExitPreview?: () => void,
  ): T | undefined {
    const previousState = this.currentState;
    const nextState = previousState.beforeModifyCube();
    if (!nextState) {
      // 遮断された
      return undefined;
    }
    const result = action();
    if (result === false) return result;
    if (nextState !== previousState && this.currentState === previousState) {
      if (previousState.kind === "previewing" && onExitPreview) {
        onExitPreview();
      }
      this.transitionTo(nextState);
    }
    return result;
  }

  /**
   * 探索開始
   * @returns 遷移成功なら true、遮断時は false
   */
  startSolving(): boolean {
    const nextState = this.currentState.startSolve();
    if (!nextState) return false;
    return this.transitionTo(nextState);
  }

  /**
   * 探索完了
   * @param hasSolution 解法が得られたかどうか
   */
  finishSolving(hasSolution: boolean): boolean {
    const nextState = this.currentState.finishSolve(hasSolution);
    if (!nextState) return false;
    return this.transitionTo(nextState);
  }

  /**
   * 探索キャンセル
   */
  cancelSolving(): boolean {
    const nextState = this.currentState.cancelSolve();
    if (!nextState) return false;
    return this.transitionTo(nextState);
  }

  /**
   * 解法パネルクローズ
   */
  closeSolution(): boolean {
    const nextState = this.currentState.closeSolution();
    if (!nextState) return false;
    return this.transitionTo(nextState);
  }

  /**
   * 外部から解法プレビューを設定（ストレージ復元や直接設定など）
   */
  setPreviewing(): boolean {
    return this.transitionTo(new PreviewingState());
  }

  /**
   * 外部から Idle 状態にリセット
   */
  resetToIdle(): boolean {
    return this.transitionTo(new IdleState());
  }
}
