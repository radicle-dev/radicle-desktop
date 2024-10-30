/* eslint-disable @typescript-eslint/naming-convention */
import type { PeerManager, RadiclePeer } from "./peerManager.js";
import type * as Stream from "node:stream";

import * as Fs from "node:fs/promises";
import * as Path from "node:path";
import { test as base, expect } from "@playwright/test";
import { execa } from "execa";

import * as issue from "@tests/support/cobs/issue.js";
import * as logLabel from "@tests/support/logPrefix.js";
import * as patch from "@tests/support/cobs/patch.js";
import { createOptions, supportDir, tmpDir } from "@tests/support/support.js";
import { createPeerManager } from "@tests/support/peerManager.js";
import { createRepo } from "@tests/support/repo.js";
import { formatOid } from "@app/lib/utils.js";

export { expect };

const fixturesDir = Path.resolve(supportDir, "..", "./fixtures");

export const test = base.extend<{
  // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
  forAllTests: void;
  stateDir: string;
  peerManager: PeerManager;
  peer: RadiclePeer;
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

  peerManager: async ({ stateDir, outputLog }, use) => {
    const peerManager = await createPeerManager({
      dataDir: Path.resolve(Path.join(stateDir, "peers")),
      outputLog,
    });
    await use(peerManager);
    await peerManager.shutdown();
  },

  peer: async ({ peerManager }, use) => {
    const peer = await peerManager.createPeer({
      name: "httpd",
      gitOptions: gitOptions["bob"],
    });

    await peer.startNode();
    await peer.startHttpd();

    await use(peer);
  },

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
  const output = text
    .split("\n")
    .map(line => `${label}${line}`)
    .join("\n");

  outputLog.write(`${output}\n`);
  if (!process.env.CI) {
    console.log(output);
  }
}

export async function createCobsFixture(
  peerManager: PeerManager,
  peer: RadiclePeer,
) {
  await peer.rad(["follow", peer.nodeId, "--alias", "palm"]);
  await Fs.mkdir(Path.join(tmpDir, "repos", "cobs"), { recursive: true });
  const { repoFolder, rid, defaultBranch } = await createRepo(peer, {
    name: "cobs",
  });
  const eve = await peerManager.createPeer({
    name: "eve",
    gitOptions: gitOptions["eve"],
  });
  await eve.startNode({
    node: { ...defaultConfig.node, connect: [peer.address], alias: "eve" },
  });
  await eve.rad(["clone", rid], { cwd: eve.checkoutPath });

  const issueOne = await issue.create(
    peer,
    "This `title` has **markdown**",
    "This is a description\nWith some multiline text.",
    ["bug", "feature-request"],
    { cwd: repoFolder },
  );
  await peer.rad(
    ["issue", "react", issueOne, "--emoji", "👍", "--to", issueOne],
    {
      cwd: repoFolder,
    },
  );
  await peer.rad(
    ["issue", "react", issueOne, "--emoji", "🎉", "--to", issueOne],
    {
      cwd: repoFolder,
    },
  );
  await peer.rad(
    ["issue", "assign", issueOne, "--add", `did:key:${peer.nodeId}`],
    createOptions(repoFolder, 1),
  );
  const { stdout: commentIssueOne } = await peer.rad(
    [
      "issue",
      "comment",
      issueOne,
      "--message",
      "This is a multiline comment\n\nWith some more text.",
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 2),
  );
  await peer.rad(
    ["issue", "react", issueOne, "--emoji", "🙏", "--to", commentIssueOne],
    {
      cwd: repoFolder,
    },
  );
  const { stdout: replyIssueOne } = await peer.rad(
    [
      "issue",
      "comment",
      issueOne,
      "--message",
      "This is a reply, to a first comment.",
      "--reply-to",
      commentIssueOne,
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 3),
  );
  await peer.rad(
    ["issue", "react", issueOne, "--emoji", "🚀", "--to", replyIssueOne],
    {
      cwd: repoFolder,
    },
  );
  await peer.rad(
    [
      "issue",
      "comment",
      issueOne,
      "--message",
      "A root level comment after a reply, for margins sake.",
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 4),
  );

  const issueTwo = await issue.create(
    peer,
    "A closed issue",
    "This issue has been closed\n\nsource: [link](https://radicle.xyz)",
    [],
    { cwd: repoFolder },
  );
  await peer.rad(
    ["issue", "state", issueTwo, "--closed"],
    createOptions(repoFolder, 1),
  );

  const issueThree = await issue.create(
    peer,
    "A solved issue",
    "This issue has been solved\n\n```js\nconsole.log('hello world')\nconsole.log(\"\")\n```",
    [],
    { cwd: repoFolder },
  );
  await peer.rad(
    ["issue", "state", issueThree, "--solved"],
    createOptions(repoFolder, 1),
  );

  const patchOne = await patch.create(
    peer,
    ["Add README", "This commit adds more information to the README"],
    "feature/add-readme",
    () => Fs.writeFile(Path.join(repoFolder, "README.md"), "# Cobs Repo"),
    ["Let's add a README", "This repo needed a README"],
    { cwd: repoFolder },
  );
  const { stdout: commentPatchOne } = await peer.rad(
    [
      "patch",
      "comment",
      patchOne,
      "--message",
      "I'll review the patch",
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 1),
  );
  await peer.rad(
    [
      "patch",
      "comment",
      patchOne,
      "--message",
      "Thanks for that!",
      "--reply-to",
      commentPatchOne,
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 2),
  );
  await peer.rad(
    [
      "patch",
      "comment",
      patchOne,
      "--message",
      "Yeah no problem!",
      "--reply-to",
      commentPatchOne,
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 3),
  );
  const { stdout: commentTwo } = await peer.rad(
    [
      "patch",
      "comment",
      patchOne,
      "--message",
      "Looking good so far",
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 4),
  );
  await peer.rad(
    [
      "patch",
      "comment",
      patchOne,
      "--message",
      "Thanks again!",
      "--reply-to",
      commentTwo,
      "--quiet",
      "--no-announce",
    ],
    createOptions(repoFolder, 5),
  );
  await peer.rad(
    ["patch", "review", patchOne, "-m", "LGTM", "--accept"],
    createOptions(repoFolder, 6),
  );
  await patch.merge(
    peer,
    defaultBranch,
    "feature/add-readme",
    createOptions(repoFolder, 7),
  );

  const patchTwo = await patch.create(
    peer,
    ["Add subtitle to README"],
    "feature/add-more-text",
    () => Fs.appendFile(Path.join(repoFolder, "README.md"), "\n\n## Subtitle"),
    [],
    { cwd: repoFolder },
  );
  await peer.rad(
    [
      "patch",
      "review",
      patchTwo,
      "-m",
      "Not the README we are looking for",
      "--reject",
    ],
    createOptions(repoFolder, 1),
  );

  const patchThree = await patch.create(
    peer,
    [
      "Rewrite subtitle to README",
      "This was really necessary",
      "Blazingly fast",
    ],
    "feature/better-subtitle",
    () => Fs.appendFile(Path.join(repoFolder, "README.md"), "\n\n## Better?"),
    [
      "Taking another stab at the README",
      "This is a big improvement over the last one",
      "Hopefully **this** is the last time",
    ],
    { cwd: repoFolder },
  );
  await peer.rad(
    ["patch", "label", patchThree, "--add", "documentation"],
    createOptions(repoFolder, 1),
  );
  await eve.rad(
    ["patch", "review", patchThree, "-m", "This looks better"],
    createOptions(repoFolder, 2),
  );
  await Fs.appendFile(
    Path.join(repoFolder, "README.md"),
    "\n\nHad to push a new revision",
  );
  await peer.git(["add", "."], { cwd: repoFolder });
  await peer.git(["commit", "-m", "Add more text"], { cwd: repoFolder });
  await peer.git(
    [
      "push",
      "-o",
      "patch.message=Most of the missing README text was caused by the git-daemon not having a writers block. It seems like using an RNG was not a good enough solution.",
      "-o",
      "patch.message=After this change, the README seem to be written correctly",
      "rad",
      "feature/better-subtitle",
    ],
    createOptions(repoFolder, 3),
  );
  await peer.rad(
    [
      "patch",
      "review",
      patchThree,
      "-m",
      "No this doesn't look better",
      "--reject",
    ],
    createOptions(repoFolder, 2),
  );

  const patchFour = await patch.create(
    peer,
    ["This patch is going to be archived"],
    "feature/archived",
    () => Fs.writeFile(Path.join(repoFolder, "CONTRIBUTING.md"), "# Archived"),
    [],
    { cwd: repoFolder },
  );
  await peer.rad(
    [
      "patch",
      "review",
      patchFour,
      "-m",
      "No review due to patch being archived.",
    ],
    createOptions(repoFolder, 1),
  );
  await peer.rad(["patch", "archive", patchFour], createOptions(repoFolder, 2));

  const patchFive = await patch.create(
    peer,
    ["This patch is going to be reverted to draft"],
    "feature/draft",
    () => Fs.writeFile(Path.join(repoFolder, "LICENSE"), "Draft"),
    [],
    { cwd: repoFolder },
  );
  await peer.rad(
    ["patch", "ready", patchFive, "--undo"],
    createOptions(repoFolder, 1),
  );
}

export async function createMarkdownFixture(peer: RadiclePeer) {
  await Fs.mkdir(Path.join(tmpDir, "repos", "markdown"), { recursive: true });
  await execa("tar", [
    "-xf",
    Path.join(fixturesDir, "repos", "markdown.tar.bz2"),
    "-C",
    Path.join(tmpDir, "repos", "markdown"),
  ]);
  const { repoFolder } = await createRepo(peer, { name: "markdown" });
  await Fs.cp(Path.join(tmpDir, "repos", "markdown"), repoFolder, {
    recursive: true,
  });

  await peer.git(["add", "."], { cwd: repoFolder });
  const commitMessage = `Add Markdown cheat sheet

  Borrowed from [Adam Pritchard][ap].
  No modifications were made.

  [ap]: https://github.com/adam-p/markdown-here/wiki/Markdown-Cheatsheet`;
  await peer.git(["commit", "-m", commitMessage], {
    cwd: repoFolder,
  });
  await peer.git(["push", "rad"], { cwd: repoFolder });
  await issue.create(
    peer,
    "This `title` has **markdown**",
    'This is a description\n\nWith some multiline text.\n\n```\n23-11-06 10:19 ➜  radicle-jetbrains-plugin git:(main) rad id update --title "Godify jchrist" --description "where jchrist ascends to a god of this project" --delegate did:key:z6MkpaATbhkGbSMysNomYTFVvKG5bnNKYZ2cCamfoHzX9SnL --threshold 1\n\n✓ Identity revision 029837dde8f5c49704e50a19cd709473ac66a456 created\n```',
    ["bug", "feature-request"],
    { cwd: repoFolder },
  );
}

export const aliceMainHead = "7babd25a74eb3752ec24672b5edf0e7ecb4daf24";
export const aliceMainCommitMessage =
  "Verify that crate::DoubleColon::should_work()";
export const aliceMainCommitCount = 8;
export const aliceRemote =
  "did:key:z6MkqGC3nWZhYieEVTVDKW5v588CiGfsDSmRVG9ZwwWTvLSK";
export const shortAliceHead = formatOid(aliceMainHead);
export const bobRemote =
  "did:key:z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5";
export const bobHead = "82f570ec909e77c7e1bb764f1429b1e01b1b4a90";
export const bobMainCommitCount = 9;
export const shortBobHead = formatOid(bobHead);
export const cobRid = "rad:z3fpY7nttPPa6MBnAv2DccHzQJnqe";
export const markdownRid = "rad:z2tchH2Ti4LxRKdssPQYs6VHE5rsg";
export const shortNodeRemote = "z6MktU…1xB22S";
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
