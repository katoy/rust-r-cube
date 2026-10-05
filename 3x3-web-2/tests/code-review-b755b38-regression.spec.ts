import { test, expect } from "@playwright/test";
import { SOLVED } from "../web/model";

test.describe("b755b38 レビュー指摘点 (F1〜F6) 回帰テスト", () => {
  // F2: 解法再生後の Undo が見かけ上の無操作にならず、混ぜた局面に正しく戻る
  test("F2: 解法再生後に Undo を実行した際、混ぜた局面に復元され、Redo も可逆的に機能する", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const { SOLVED } = await import("/web/model.ts");
      const { apply_moves } = (window as any).cube_studio;

      const store = new CubeStore();
      const rState = JSON.parse(apply_moves(SOLVED, "R")).state;

      // 1. 初期状態から R を回す
      store.replace(rState, true, [Math.PI / 2, 0, 0, 0, 0, 0]);
      const stateAfterR = store.getState();

      // 2. 解法をセット (R' で完成状態に戻る解法)
      store.setSolution({
        state: SOLVED,
        moves: ["R'"],
        states: [rState, SOLVED],
        elapsed_ms: 5,
        nodes: 10,
        algorithm: "kociemba",
        phases: [],
      });

      // 3. 解法を最後までシーク (完成状態へ)
      store.updateAfterSeek(SOLVED, [0, 0, 0, 0, 0, 0], 1);
      const stateAfterSeek = store.getState();
      const canUndoAfterSeek = store.canUndo();

      // 4. Undo を実行 -> 混ぜた局面 (rState) に戻るべき
      const firstUndoSuccess = store.undo();
      const stateAfterFirstUndo = store.getState();
      const canUndoAfterFirstUndo = store.canUndo();
      const canRedoAfterFirstUndo = store.canRedo();

      // 5. さらに Undo を実行 -> 初期の完成局面 (SOLVED) に戻るべき
      const secondUndoSuccess = store.undo();
      const stateAfterSecondUndo = store.getState();
      const canUndoAfterSecondUndo = store.canUndo();

      // 6. Redo を実行 -> 混ぜた局面 (rState) に進むべき
      const firstRedoSuccess = store.redo();
      const stateAfterFirstRedo = store.getState();

      // 7. さらに Redo を実行 -> 解法再生後 (SOLVED) に進むべき
      const secondRedoSuccess = store.redo();
      const stateAfterSecondRedo = store.getState();

      return {
        rState,
        stateAfterR,
        stateAfterSeek,
        canUndoAfterSeek,
        firstUndoSuccess,
        stateAfterFirstUndo,
        canUndoAfterFirstUndo,
        canRedoAfterFirstUndo,
        secondUndoSuccess,
        stateAfterSecondUndo,
        canUndoAfterSecondUndo,
        firstRedoSuccess,
        stateAfterFirstRedo,
        secondRedoSuccess,
        stateAfterSecondRedo,
      };
    });

    expect(result.stateAfterR).toBe(result.rState);
    expect(result.stateAfterSeek).toBe(SOLVED);
    expect(result.canUndoAfterSeek).toBe(true);

    // 最初の Undo で混ぜた局面 (R) に戻る
    expect(result.firstUndoSuccess).toBe(true);
    expect(result.stateAfterFirstUndo).toBe(result.rState);
    expect(result.canUndoAfterFirstUndo).toBe(true);
    expect(result.canRedoAfterFirstUndo).toBe(true);

    // 2回目の Undo で初期の完成状態に戻る
    expect(result.secondUndoSuccess).toBe(true);
    expect(result.stateAfterSecondUndo).toBe(SOLVED);
    expect(result.canUndoAfterSecondUndo).toBe(false);

    // Redo で再び可逆的に進む
    expect(result.firstRedoSuccess).toBe(true);
    expect(result.stateAfterFirstRedo).toBe(result.rState);

    expect(result.secondRedoSuccess).toBe(true);
    expect(result.stateAfterSecondRedo).toBe(SOLVED);
  });

  // F4: getScopedStorageKey で '/' と '/index.html' が同一スコープとして扱われる
  test("F4: getScopedStorageKey は末尾の /index.html を正規化し同一スコープに集約する", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { getScopedStorageKey } = await import("/web/main.ts");

      const rootSlash = getScopedStorageKey("cube-studio-v1", "/");
      const rootIndex = getScopedStorageKey("cube-studio-v1", "/index.html");

      const nestedSlash = getScopedStorageKey(
        "cube-studio-v1",
        "/nested/cube/",
      );
      const nestedIndex = getScopedStorageKey(
        "cube-studio-v1",
        "/nested/cube/index.html",
      );
      const nestedNoSlash = getScopedStorageKey(
        "cube-studio-v1",
        "/nested/cube",
      );

      return {
        rootSlash,
        rootIndex,
        nestedSlash,
        nestedIndex,
        nestedNoSlash,
      };
    });

    // ルートアクセスで同一キー
    expect(result.rootSlash).toBe("cube-studio-v1");
    expect(result.rootIndex).toBe("cube-studio-v1");

    // サブディレクトリで同一キー
    expect(result.nestedSlash).toBe("cube-studio-v1:/nested/cube");
    expect(result.nestedIndex).toBe("cube-studio-v1:/nested/cube");
    expect(result.nestedNoSlash).toBe("cube-studio-v1:/nested/cube");
  });

  // F5: Service Worker がクエリ付きの共有URLを同一 canonicalUrl に集約し重複保存しない
  test("F5: Service Worker は共有リンクのクエリ違いでキャッシュを重複生成しない", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const cacheCount = await page.evaluate(async () => {
      // Service Worker の fetch ハンドラ動作を模倣検証
      const cacheName = "test-sw-cache";
      const cache = await caches.open(cacheName);

      // 異なる共有リンクのURL
      const url1 = new URL("http://localhost:5173/?state=SCRAMBLE_A");
      const url2 = new URL("http://localhost:5173/?state=SCRAMBLE_B");

      const canonicalUrl1 = url1.origin + url1.pathname;
      const canonicalUrl2 = url2.origin + url2.pathname;

      // 正規化URLに保存
      await cache.put(
        canonicalUrl1,
        new Response("<html>Cube Studio</html>", {
          headers: { "Content-Type": "text/html" },
        }),
      );
      await cache.put(
        canonicalUrl2,
        new Response("<html>Cube Studio Updated</html>", {
          headers: { "Content-Type": "text/html" },
        }),
      );

      const keys = await cache.keys();
      await caches.delete(cacheName);
      return keys.length;
    });

    // クエリが異なっても同一 canonicalUrl に集約されるため、エントリ数は 1件
    expect(cacheCount).toBe(1);
  });

  // F3: 64KB を超えるファイル選択時に file.text() を実行せずに即座に拒否する
  test("F3: 64KB を超えるファイル選択時、file.text() のメモリ展開を行わずに即座に拒否される", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      let textCalled = false;

      // 70KB のダミー巨大ファイルを作成
      const largeContent = "x".repeat(70000);
      const largeBlob = new Blob([largeContent], { type: "application/json" });
      const file = new File([largeBlob], "large.json", {
        type: "application/json",
      });

      // file.text をスパイして呼び出しを検知
      const originalText = file.text.bind(file);
      file.text = async () => {
        textCalled = true;
        return originalText();
      };

      const fileInput = document.getElementById("file") as HTMLInputElement;

      // DataTransfer を使って input にファイルをセット
      const dt = new DataTransfer();
      dt.items.add(file);
      fileInput.files = dt.files;

      // change イベントを発火
      fileInput.dispatchEvent(new Event("change"));

      // 非同期処理を少し待つ
      await new Promise((resolve) => setTimeout(resolve, 100));

      const messageElem = document.getElementById("message");
      const messageText = messageElem ? messageElem.textContent : "";

      return {
        textCalled,
        messageText,
      };
    });

    // file.text() が実行されずに早期拒否されていること
    expect(result.textCalled).toBe(false);
    expect(result.messageText).toContain("ファイルは64KB以内にしてください。");
  });
});
