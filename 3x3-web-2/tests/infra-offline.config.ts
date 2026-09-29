import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  testMatch: ["infra-offline-regression.spec.ts", "sw-update.spec.ts"],
  workers: 1,
  timeout: 45000,
  reporter: "list",
  outputDir: "../test-results/infra-offline",
  use: { browserName: "chromium", headless: true },
});
