import type { PlaywrightTestConfig } from "@playwright/test";

import * as net from "node:net";

import { devices } from "@playwright/test";

function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(0, "127.0.0.1", () => {
      const port = (server.address() as net.AddressInfo).port;
      server.close(err => (err ? reject(err) : resolve(port)));
    });
    server.on("error", reject);
  });
}

const testHttpApiPort = await getFreePort();
process.env.VITE_TEST_HTTP_API_PORT = String(testHttpApiPort);

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
      command: `VITE_AUTH_LONG_DELAY=1000 VITE_TEST_HTTP_API_PORT=${testHttpApiPort} npm run start -- --strictPort --port 3001`,
      port: 3001,
    },
  ],
};

export default config;
