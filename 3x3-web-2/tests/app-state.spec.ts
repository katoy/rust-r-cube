import { test, expect } from "@playwright/test";
import {
  AppStateMachine,
  IdleState,
  SolvingState,
  PreviewingState,
} from "../web/app-state";

test.describe("AppStateMachine / AppState (State パターン検証)", () => {
  test("初期状態は IdleState であり、適切な権限を持つ", () => {
    const fsm = new AppStateMachine();
    expect(fsm.kind).toBe("idle");
    expect(fsm.isIdle()).toBe(true);
    expect(fsm.isSolving()).toBe(false);
    expect(fsm.isPreviewing()).toBe(false);

    const state = fsm.getState();
    expect(state.canModifyCube()).toBe(true);
    expect(state.canStartSolve()).toBe(true);
    expect(state.canCancelSolve()).toBe(false);
    expect(state.canUndoRedo()).toBe(true);
    expect(state.canSeekSolution()).toBe(false);
  });

  test("Idle から Solving への遷移と、Solving 中の不正操作一括遮断", () => {
    const fsm = new AppStateMachine();
    let listenerCalled = false;
    fsm.subscribe((next, prev) => {
      expect(prev.kind).toBe("idle");
      expect(next.kind).toBe("solving");
      listenerCalled = true;
    });

    // 探索開始
    const started = fsm.startSolving();
    expect(started).toBe(true);
    expect(listenerCalled).toBe(true);
    expect(fsm.kind).toBe("solving");
    expect(fsm.isSolving()).toBe(true);

    const state = fsm.getState();
    expect(state.canModifyCube()).toBe(false);
    expect(state.canStartSolve()).toBe(false);
    expect(state.canCancelSolve()).toBe(true);
    expect(state.canUndoRedo()).toBe(false);
    expect(state.canSeekSolution()).toBe(false);

    // Solving 中に二重に startSolving を呼んでも遮断される
    expect(fsm.startSolving()).toBe(false);

    // Solving 中の runCubeMutation は一切実行されず undefined を返す
    let mutationExecuted = false;
    const result = fsm.runCubeMutation(() => {
      mutationExecuted = true;
      return 42;
    });
    expect(result).toBeUndefined();
    expect(mutationExecuted).toBe(false);
    expect(fsm.kind).toBe("solving");
  });

  test("Solving 中のキャンセルによる Idle 復帰", () => {
    const fsm = new AppStateMachine();
    fsm.startSolving();

    const cancelled = fsm.cancelSolving();
    expect(cancelled).toBe(true);
    expect(fsm.kind).toBe("idle");
    expect(fsm.isIdle()).toBe(true);

    // Idle 状態での cancelSolving は不正遷移として拒絶される
    expect(fsm.cancelSolving()).toBe(false);
  });

  test("Solving 完了による Previewing 遷移（解法あり）", () => {
    const fsm = new AppStateMachine();
    fsm.startSolving();

    const finished = fsm.finishSolving(true);
    expect(finished).toBe(true);
    expect(fsm.kind).toBe("previewing");
    expect(fsm.isPreviewing()).toBe(true);

    const state = fsm.getState();
    expect(state.canModifyCube()).toBe(true);
    expect(state.canStartSolve()).toBe(true);
    expect(state.canCancelSolve()).toBe(false);
    expect(state.canUndoRedo()).toBe(true);
    expect(state.canSeekSolution()).toBe(true);
  });

  test("Solving 完了による Idle 復帰（解法なし・失敗）", () => {
    const fsm = new AppStateMachine();
    fsm.startSolving();

    const finished = fsm.finishSolving(false);
    expect(finished).toBe(true);
    expect(fsm.kind).toBe("idle");
  });

  test("Previewing 中にキューブ変更操作が入った場合、プレビューを自動終了して Idle へ安全に遷移する", () => {
    const fsm = new AppStateMachine();
    fsm.startSolving();
    fsm.finishSolving(true);
    expect(fsm.kind).toBe("previewing");

    let exitPreviewHookCalled = false;
    let mutationExecuted = false;

    const result = fsm.runCubeMutation(
      () => {
        mutationExecuted = true;
        return "mutated";
      },
      () => {
        exitPreviewHookCalled = true;
      },
    );

    expect(result).toBe("mutated");
    expect(mutationExecuted).toBe(true);
    expect(exitPreviewHookCalled).toBe(true);
    expect(fsm.kind).toBe("idle");
  });

  test("Previewing 中に closeSolution を呼ぶと Idle へ遷移する", () => {
    const fsm = new AppStateMachine();
    fsm.startSolving();
    fsm.finishSolving(true);
    expect(fsm.kind).toBe("previewing");

    const closed = fsm.closeSolution();
    expect(closed).toBe(true);
    expect(fsm.kind).toBe("idle");

    // Idle 中に再度 closeSolution を呼んでも遮断される
    expect(fsm.closeSolution()).toBe(false);
  });

  test("Previewing 中に再探索（startSolving）を開始すると Solving へ直接遷移可能", () => {
    const fsm = new AppStateMachine();
    fsm.startSolving();
    fsm.finishSolving(true);
    expect(fsm.kind).toBe("previewing");

    const started = fsm.startSolving();
    expect(started).toBe(true);
    expect(fsm.kind).toBe("solving");
  });
});
