import type { RadiclePeer } from "@tests/support/peerManager.js";

import * as Fs from "node:fs/promises";
import * as Path from "node:path";

import { expect, test } from "@tests/support/fixtures.js";
import { createRepo } from "@tests/support/repo.js";

async function createRichRepo(peer: RadiclePeer) {
  const { rid, repoFolder } = await createRepo(peer, { name: "rich" });
  await Fs.writeFile(
    Path.join(repoFolder, "README.md"),
    ["# Rich", "", "> [!NOTE]", "> Read the guide first.", ""].join("\n"),
  );
  await peer.git(["add", "."], { cwd: repoFolder });
  await peer.git(["commit", "-m", "Write a rich readme"], { cwd: repoFolder });
  await peer.git(["push", "rad", "main"], { cwd: repoFolder });
  return { rid };
}

test("render alerts in a readme", async ({ page, peer }) => {
  const { rid } = await createRichRepo(peer);
  await page.goto(`/repos/${rid}/home`);

  const readme = page.locator(".markdown");
  await expect(
    readme.locator(".alert-note").getByText("Read the guide first."),
  ).toBeVisible();
});
