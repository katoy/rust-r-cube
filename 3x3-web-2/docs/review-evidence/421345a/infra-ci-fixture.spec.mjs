import { test, expect } from "@playwright/test";
test("intentional reporter failure fixture without browser", () => {
  expect(1).toBe(2);
});
