/* eslint-disable @typescript-eslint/naming-convention */
import type { PeerManager, RadiclePeer } from "./peerManager.js";
import type * as Stream from "node:stream";

import * as Fs from "node:fs/promises";
import * as Path from "node:path";
import getPort from "get-port";
import { test as base, expect, type Page } from "@playwright/test";

import * as logLabel from "@tests/support/logPrefix.js";
import { createPeerManager } from "@tests/support/peerManager.js";
import { randomTag } from "./support.js";

export { expect };

export const test = base.extend<{
  // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
  forAllTests: void;
  stateDir: string;
  peerManager: PeerManager;
  peer: RadiclePeer;
  authenticatedContext: Page;
  outputLog: Stream.Writable;
}>({
  forAllTests: [
    async ({ outputLog, page }, use) => {
      const browserLabel = logLabel.logPrefix("browser");
      page.on("console", msg => {
        // Ignore common console logs that we don't care about.
        if (
          msg.text().startsWith("[vite] connected.") ||
          msg.text().startsWith("[vite] connecting...") ||
          msg.text().startsWith("Not able to parse url") ||
          msg
            .text()
            .includes("Please make sure it wasn't preloaded for nothing.")
        ) {
          return;
        }
        log(msg.text(), browserLabel, outputLog);
      });

      if (!process.env.CONTINUE_ON_ERRORS) {
        page.on("pageerror", msg => {
          expect(
            false,
            `Test failed because there was a console error in the app: ${msg}`,
          ).toBeTruthy();
        });
      }

      const playwrightLabel = logLabel.logPrefix("playwright");

      function isLocalhost(url: URL) {
        return url.hostname === "localhost" || url.hostname === "127.0.0.1";
      }

      await page.route(
        url => !isLocalhost(url),
        route => {
          log(
            `Aborted remote request: ${route.request().url()}`,
            playwrightLabel,
            outputLog,
          );
          return route.abort();
        },
      );

      await page.route(
        url =>
          url.href.startsWith("https://www.gravatar.com/avatar/") ||
          (url.href.endsWith(".png") && !isLocalhost(url)),
        route => {
          return route.fulfill({
            status: 200,
            path: "./public/radicle.svg",
          });
        },
      );

      await use();
    },
    { scope: "test", auto: true },
  ],

  outputLog: async ({ stateDir }, use) => {
    const logFile = await Fs.open(Path.join(stateDir, "test.log"), "a");
    await use(logFile.createWriteStream());
    await logFile.close();
  },

  authenticatedContext: async ({ page }, use) => {
    await page.goto("/");
    await page.getByPlaceholder("Enter desired alias").fill("palm");
    await page.getByPlaceholder("Enter passphrase to protect").fill("asdf");
    await page.getByPlaceholder("Repeat passphrase").fill("asdf");
    await page
      .getByRole("button", { name: "icon-seedling Create new identity" })
      .click();
    await expect(
      page.getByRole("button", {
        name: "z6MktULudTtAsAhRegYPiZ6631RV3viv12qd4GQF8z1xB22S icon-copy",
      }),
    ).toBeVisible();

    await use(page);
  },

  peerManager: async ({ stateDir, outputLog }, use) => {
    const peerManager = await createPeerManager({
      dataDir: Path.resolve(Path.join(stateDir, "peers")),
      outputLog,
    });
    await use(peerManager);
    await peerManager.shutdown();
  },

  peer: [
    async ({ page, peerManager }, use) => {
      const peer = await peerManager.createPeer({
        name: randomTag(),
        gitOptions: gitOptions["bob"],
      });

      const port = await getPort();
      await page.addInitScript(
        port => localStorage.setItem("TEST_HTTP_API_PORT", port.toString()),
        port,
      );
      await peer.startSSHAgent();
      await peer.startHttpd(port);

      await use(peer);

      await peer.stopSSHAgent();
    },
    { scope: "test", auto: true },
  ],

  // eslint-disable-next-line no-empty-pattern
  stateDir: async ({}, use, testInfo) => {
    const stateDir = testInfo.outputDir;
    await Fs.rm(stateDir, { recursive: true, force: true });
    await Fs.mkdir(stateDir, { recursive: true });

    await use(stateDir);
    if (
      process.env.CI &&
      (testInfo.status === "passed" || testInfo.status === "skipped")
    ) {
      await Fs.rm(stateDir, { recursive: true });
    }
  },
});

function log(text: string, label: string, outputLog: Stream.Writable) {
  if (!process.env.QUIET) {
    const output = text
      .split("\n")
      .map(line => `${label}${line}`)
      .join("\n");

    outputLog.write(`${output}\n`);
    if (!process.env.CI) {
      console.log(output);
    }
  }
}

export const defaultHttpdPort = 8081;
export const gitOptions = {
  alice: {
    GIT_AUTHOR_NAME: "Alice Liddell",
    GIT_AUTHOR_EMAIL: "alice@radicle.xyz",
    GIT_AUTHOR_DATE: "1727621093",
    GIT_COMMITTER_NAME: "Alice Liddell",
    GIT_COMMITTER_EMAIL: "alice@radicle.xyz",
    GIT_COMMITTER_DATE: "1727621093",
  },
  bob: {
    GIT_AUTHOR_NAME: "Bob Belcher",
    GIT_AUTHOR_EMAIL: "bob@radicle.xyz",
    GIT_AUTHOR_DATE: "1727621093",
    GIT_COMMITTER_NAME: "Bob Belcher",
    GIT_COMMITTER_EMAIL: "bob@radicle.xyz",
    GIT_COMMITTER_DATE: "1730220293",
  },

  eve: {
    GIT_AUTHOR_NAME: "Eve Johnson",
    GIT_AUTHOR_EMAIL: "eve@radicle.xyz",
    GIT_AUTHOR_DATE: "1727621093",
    GIT_COMMITTER_NAME: "Eve Johnson",
    GIT_COMMITTER_EMAIL: "eve@radicle.xyz",
    GIT_COMMITTER_DATE: "1730220293",
  },
};
export const defaultConfig: Config = {
  publicExplorer: "https://app.radicle.xyz/nodes/$host/$rid$path",
  preferredSeeds: [],
  web: {
    pinned: {
      repositories: [],
    },
  },
  cli: {
    hints: true,
  },
  node: {
    alias: "alice",
    listen: [],
    peers: {
      type: "dynamic",
    },
    connect: [],
    externalAddresses: [],
    network: "main",
    log: "INFO",
    relay: "auto",
    limits: {
      routingMaxSize: 1000,
      routingMaxAge: 604800,
      gossipMaxAge: 1209600,
      fetchConcurrency: 1,
      maxOpenFiles: 4096,
      rate: {
        inbound: {
          fillRate: 5.0,
          capacity: 1024,
        },
        outbound: {
          fillRate: 10.0,
          capacity: 2048,
        },
      },
      connection: {
        inbound: 128,
        outbound: 16,
      },
    },
    workers: 8,
    seedingPolicy: {
      default: "block",
    },
  },
};

export type Config = {
  publicExplorer: string;
  preferredSeeds: string[];
  cli: { hints: boolean };
  web: {
    pinned: {
      repositories: string[];
    };
    imageUrl?: string;
    name?: string;
    description?: string;
  };
  node: NodeConfig;
};

export type NodeConfig = {
  alias: string;
  peers: { type: "static" } | { type: "dynamic" };
  listen: string[];
  connect: string[];
  externalAddresses: string[];
  proxy?: string;
  onion?: { mode: "proxy"; address: string } | { mode: "forward" };
  log: "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE";
  network: "main" | "test";
  relay: "always" | "never" | "auto";
  limits: {
    routingMaxSize: number;
    routingMaxAge: number;
    fetchConcurrency: number;
    gossipMaxAge: number;
    maxOpenFiles: number;
    rate: {
      inbound: {
        fillRate: number;
        capacity: number;
      };
      outbound: {
        fillRate: number;
        capacity: number;
      };
    };
    connection: {
      inbound: number;
      outbound: number;
    };
  };
  workers: number;
  seedingPolicy:
    | {
        default: "block";
      }
    | {
        default: "allow";
        scope: "followed" | "all";
      };
};
