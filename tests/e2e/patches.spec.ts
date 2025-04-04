import { expect, playgroundRid, test } from "@tests/support/fixtures.js";
import * as Fs from "node:fs/promises";
import * as Path from "node:path";

test("create and interact with patches", async ({
  authenticatedContext: page,
  peer,
}) => {
  const repoCheckout = Path.resolve(peer.checkoutPath, "playground");
  await test.step("create a new repo", async () => {
    await page
      .getByRole("button", { name: "Create a new repo", exact: true })
      .click();
    await page.getByPlaceholder("Name of your repo").fill("playground");
    await page.getByPlaceholder("Add description").fill("Lorem ipsum dolor");
    await page
      .getByRole("button", { name: "Create new repo", exact: true })
      .click();
    await page.getByRole("button", { name: "p playground" }).click();
    await expect(page.getByText(playgroundRid)).toBeVisible();
  });

  await test.step("create a patch", async () => {
    try {
      await peer.startNode();
      await peer.rad(["clone", playgroundRid], {
        cwd: Path.resolve(peer.checkoutPath),
      });
      await Fs.writeFile(Path.resolve(repoCheckout, "README.md"), "# README");
      await peer.git(["switch", "-c", "new-readme"], { cwd: repoCheckout });
      await peer.git(["add", "."], { cwd: repoCheckout });
      await peer.git(
        [
          "commit",
          "-m",
          "Add a README",
          "-m",
          "There was no README in this repo",
        ],
        { cwd: repoCheckout },
      );
      await peer.git(["push", "rad", "HEAD:refs/patches"], {
        cwd: repoCheckout,
      });
    } catch (err) {
      console.error("Unable to create a patch");
      console.error(err);
      process.exit(1);
    }
  });

  await test.step("navigate to patch", async () => {
    await page.getByRole("link", { name: "icon-patch Patches" }).click();
    await page
      .getByText("Add a README alice opened 572eebc more than a year ago +1 -0")
      .click();
    await expect(page.getByText("Add a README").nth(1)).toBeVisible();
  });

  await test.step("edit patch title", async () => {
    await page.getByLabel("edit-patch-title").click();
    await page.getByLabel("patch-title").fill("Add the first README");
    await page.getByLabel("save-new-title").click();
    await expect(page.getByText("Add the first README").nth(1)).toBeVisible();
  });

  await test.step("edit first revision", async () => {
    await expect(
      page.getByText("There was no README in this repo"),
    ).toBeVisible();
    await page.getByLabel("edit-revision-description").click();
    await page
      .getByPlaceholder("Leave a comment")
      .fill("Now there will be a README");
    await page.getByRole("button", { name: "icon-checkmark Save" }).click();
  });

  await test.step("edit lifecycle patch", async () => {
    await page.getByLabel("toggle-patch-state").click();
    await page
      .getByRole("button", {
        name: "icon-patch-archived Archived",
        exact: true,
      })
      .click();
    await expect(
      page.getByLabel("toggle-patch-state").filter({ hasText: "Archived" }),
    ).toBeVisible();

    await page.getByLabel("toggle-patch-state").click();
    await page
      .getByRole("button", { name: "icon-patch Open", exact: true })
      .click();
    await expect(
      page.getByLabel("toggle-patch-state").filter({ hasText: "Open" }),
    ).toBeVisible();
  });

  await peer.stopNode().catch(() => console.error("Unable to stop the node"));
});
