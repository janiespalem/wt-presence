import { defineConfig } from "@playwright/test";
import { tmpdir } from "node:os";
import { join } from "node:path";

export default defineConfig({
  testDir: "./e2e",
  testMatch: "**/*.pw.ts",
  outputDir: process.env.WT_TEST_OUTPUT || join(tmpdir(), "wt-presence-browser-results"),
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:4173",
    viewport: { width: 1280, height: 1000 },
    launchOptions: process.env.WT_CHROME_PATH
      ? { executablePath: process.env.WT_CHROME_PATH }
      : {},
  },
  webServer: {
    command: "npm run preview -- --host 127.0.0.1 --port 4173 --strictPort",
    url: "http://127.0.0.1:4173",
  },
});
