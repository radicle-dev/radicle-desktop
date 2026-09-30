import type { Page } from "@playwright/test";
import type { RadiclePeer } from "@tests/support/peerManager.js";

import { createProject } from "@tests/support/collaboration.js";
import {
  expect,
  reload,
  test,
  waitForCommand,
} from "@tests/support/fixtures.js";

async function openPatch(page: Page, peer: RadiclePeer) {
  const { rid, patchId } = await createProject(peer);
  await page.goto(`/repos/${rid}/patches/${patchId}`);
  await expect(
    page.getByRole("button", { name: "Shout the third line" }),
  ).toBeVisible();
  return { rid, patchId };
}

test("a patch can be labelled", async ({ page, peer }) => {
  await openPatch(page, peer);

  await page.getByRole("button", { name: "Add labels" }).click();
  await page.getByPlaceholder("Add label").fill("needs-review");
  await waitForCommand(page, "edit_patch", () =>
    page.getByPlaceholder("Add label").press("Enter"),
  );
  const label = page.getByRole("button", { name: "needs-review" });
  await expect(label).toBeVisible();

  await reload(page);
  await expect(label).toBeVisible();
});

test("a patch can be archived, drafted and reopened", async ({
  page,
  peer,
}) => {
  await openPatch(page, peer);

  const state = (name: string) =>
    page.getByRole("button", { name: new RegExp(`${name} icon-chevron`) });

  await state("Open").click();
  await page.getByRole("button", { name: /Archived$/ }).click();
  await expect(state("Archived")).toBeVisible();
  await reload(page);
  await expect(state("Archived")).toBeVisible();

  await state("Archived").click();
  await page.getByRole("button", { name: /Draft$/ }).click();
  await expect(state("Draft")).toBeVisible();

  await state("Draft").click();
  await page.getByRole("button", { name: /Open$/ }).click();
  await expect(state("Open")).toBeVisible();
  await reload(page);
  await expect(state("Open")).toBeVisible();
});

test("a patch can be deleted", async ({ page, peer }) => {
  const { rid } = await openPatch(page, peer);

  await page.getByTitle("Delete patch from your node").click();
  await page
    .getByRole("button", { name: "icon-trash Delete", exact: true })
    .last()
    .click();

  await expect(page).toHaveURL(new RegExp(`/repos/${rid}/patches`));
  // The list renders with its items, so once its filters show it's loaded.
  await expect(page.getByRole("link", { name: /Merged/ })).toBeVisible();
  await expect(page.getByText("Shout the third line")).toBeHidden();
});
