import { test, expect } from "@playwright/test";

test.describe("R05: 3D arrow and center label rotation directions", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
  });

  test("U center arrow points to +X (right, [1, 0, 0]) after clockwise U turn", async ({
    page,
  }) => {
    // 動きを減らすをチェックしてアニメーション遅延なしで実行
    await page.locator("#reduced-motion").check();

    // 初期状態の U センター矢印方向を検証 (faceUp[0] = [0, 0, -1])
    const initialDir = await page.evaluate(() => {
      const scene = (window as any).cube_scene;
      const arrows = scene.getArrows();
      const uCenterArrow = arrows[4]; // U 面センター (index 4)
      // THREE.Vector3(0, 1, 0) に arrow.quaternion を適用
      const dir = { x: 0, y: 1, z: 0 };
      const q = uCenterArrow.quaternion;
      // quaternion rotation of (0, 1, 0)
      const x = dir.x,
        y = dir.y,
        z = dir.z;
      const qx = q._x,
        qy = q._y,
        qz = q._z,
        qw = q._w;
      const ix = qw * x + qy * z - qz * y;
      const iy = qw * y + qz * x - qx * z;
      const iz = qw * z + qx * y - qy * x;
      const iw = -qx * x - qy * y - qz * z;
      return {
        x: ix * qw + iw * -qx + iy * -qz - iz * -qy,
        y: iy * qw + iw * -qy + iz * -qx - ix * -qz,
        z: iz * qw + iw * -qz + ix * -qy - iy * -qx,
      };
    });

    expect(initialDir.x).toBeCloseTo(0, 2);
    expect(initialDir.y).toBeCloseTo(0, 2);
    expect(initialDir.z).toBeCloseTo(-1, 2);

    // U 操作（時計回り 90度）を実行
    await page.locator('[data-move="U"]').click();

    // 時計回りに 90度回転した場合、奥 [0, 0, -1] から 右 [+1, 0, 0] を向くべき
    const afterDir = await page.evaluate(() => {
      const scene = (window as any).cube_scene;
      const arrows = scene.getArrows();
      const uCenterArrow = arrows[4];
      const dir = { x: 0, y: 1, z: 0 };
      const q = uCenterArrow.quaternion;
      const x = dir.x,
        y = dir.y,
        z = dir.z;
      const qx = q._x,
        qy = q._y,
        qz = q._z,
        qw = q._w;
      const ix = qw * x + qy * z - qz * y;
      const iy = qw * y + qz * x - qx * z;
      const iz = qw * z + qx * y - qy * x;
      const iw = -qx * x - qy * y - qz * z;
      return {
        x: ix * qw + iw * -qx + iy * -qz - iz * -qy,
        y: iy * qw + iw * -qy + iz * -qx - ix * -qz,
        z: iz * qw + iw * -qz + ix * -qy - iy * -qx,
      };
    });

    // 期待値: x は +1（右）。バグのある実装では x が -1（左）になる。
    expect(afterDir.x).toBeCloseTo(1, 2);
    expect(afterDir.y).toBeCloseTo(0, 2);
    expect(afterDir.z).toBeCloseTo(0, 2);
  });
});
