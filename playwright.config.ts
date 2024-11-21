import type { PlaywrightTestConfig } from "@playwright/test";
import { devices } from "@playwright/test";

const config: PlaywrightTestConfig = {
  outputDir: "./tests/artifacts",
  testDir: "./tests/e2e",
  globalSetup: "./tests/support/globalSetup.ts",
  timeout: 30_000,
  expect: {
    timeout: 8000,
  },
  fullyParallel: true,
  workers: process.env.CI ? 1 : undefined,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: "list",
  use: {
    colorScheme: "dark",
    actionTimeout: 5000,
    baseURL: "http://localhost:3001",
    trace: "retain-on-failure",
  },

  projects: [
    {
      name: "webkit",
      use: {
        ...devices["Desktop Safari"],
      },
    },
  ],

  webServer: [
    {
      command:
        "VITE_AUTH_LONG_DELAY=1000 npm run start -- --strictPort --port 3001",
      port: 3001,
    },
  ],
};

export default config;
