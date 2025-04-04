import * as Path from "node:path";
import {
  assertBinariesInstalled,
  heartwoodRelease,
  tmpDir,
} from "@tests/support/support.js";

const heartwoodBinaryPath = Path.join(
  tmpDir,
  "bin",
  "heartwood",
  heartwoodRelease,
);

process.env.PATH = [heartwoodBinaryPath, process.env.PATH].join(Path.delimiter);

export default async function globalSetup() {
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
}
