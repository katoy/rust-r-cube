import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  workers: 1,
  retries: process.env.CI ? 2 : 0,
  timeout: 45000,
  expect: { timeout: 10000 },
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:5173",
    viewport: { width: 1440, height: 1080 },
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: {
        browserName: "chromium",
        permissions: ["camera"],
        launchOptions: {
          args: [
            "--use-fake-device-for-media-stream",
            "--use-fake-ui-for-media-stream",
          ],
        },
      },
    },
  ],
  webServer: [
    {
      command: "npm run dev -- --port 5173 --strictPort",
      url: "http://127.0.0.1:5173",
      reuseExistingServer: true,
      timeout: 30000,
    },
    {
      command:
        "npm run preview -- --port 4173 --strictPort --base /nested/cube/",
      url: "http://127.0.0.1:4173/nested/cube/",
      reuseExistingServer: true,
      timeout: 30000,
    },
  ],
});
