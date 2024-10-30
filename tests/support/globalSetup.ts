import * as Fs from "node:fs";
import * as Path from "node:path";
import {
  assertBinariesInstalled,
  heartwoodRelease,
  removeWorkspace,
  tmpDir,
} from "@tests/support/support.js";
import {
  defaultConfig,
  createCobsFixture,
  createMarkdownFixture,
  defaultHttpdPort,
  gitOptions,
} from "@tests/support/fixtures.js";
import { createPeerManager } from "@tests/support/peerManager.js";

const heartwoodBinaryPath = Path.join(
  tmpDir,
  "bin",
  "heartwood",
  heartwoodRelease,
);

process.env.PATH = [heartwoodBinaryPath, process.env.PATH].join(Path.delimiter);

export default async function globalSetup(): Promise<() => void> {
  try {
    await assertBinariesInstalled("rad", heartwoodRelease, heartwoodBinaryPath);
  } catch (error) {
    console.error(error);
    console.log("");
    console.log("To download the required test binaries, run:");
    console.log(" 👉 ./scripts/install-binaries");
    console.log("");
    process.exit(1);
  }

  if (!process.env.SKIP_FIXTURE_CREATION) {
    console.log(
      "Recreating static fixtures. Set SKIP_FIXTURE_CREATION to skip this",
    );
    await removeWorkspace();
  }

  const peerManager = await createPeerManager({
    dataDir: Path.resolve(tmpDir, "peers"),
    outputLog: Fs.createWriteStream(
      Path.resolve(tmpDir, "globalPeerManager.log"),
    )
      // Workaround for fixing MaxListenersExceededWarning.
      // Since every prefixOutput stream adds stream listeners that don't autoClose.
      // TODO: We still seem to have some descriptors left open when running vitest, which we should handle.
      .setMaxListeners(16),
  });

  const palm = await peerManager.createPeer({
    name: "palm",
    gitOptions: gitOptions["alice"],
  });

  if (!process.env.SKIP_FIXTURE_CREATION) {
    await palm.startNode({
      node: {
        ...defaultConfig.node,
        seedingPolicy: { default: "allow", scope: "all" },
        alias: "palm",
      },
    });
    await palm.startHttpd(defaultHttpdPort);

    try {
      console.log("Creating markdown fixture");
      await createMarkdownFixture(palm);
      console.log("Creating cobs fixture");
      await createCobsFixture(peerManager, palm);
      console.log("All fixtures created");
    } catch (error) {
      console.log("");
      console.log("Not able to create the required fixtures.");
      console.log("Make sure you are not using binaries compiled for release.");
      console.log("");
      console.log(error);
      console.log("");
      process.exit(1);
    }
    await palm.stopNode();
  } else {
    await palm.startHttpd(defaultHttpdPort);
  }

  return async () => {
    await peerManager.shutdown();
  };
}
