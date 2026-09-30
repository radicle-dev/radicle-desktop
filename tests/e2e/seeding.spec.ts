import { createCollaborators } from "@tests/support/collaboration.js";
import {
  expect,
  test,
  useBackend,
  waitForCommand,
} from "@tests/support/fixtures.js";
import { createRepo } from "@tests/support/repo.js";

test("seeds a repo from the network by its bare RID", async ({
  page,
  peerManager,
}) => {
  const { bob, eve } = await createCollaborators(peerManager);
  const { rid } = await createRepo(eve, { name: "garden" });
  await bob.startHttpd();
  await useBackend(page, bob);

  await page.goto("/inbox");
  await page.getByRole("button", { name: "icon-plus", exact: true }).click();
  const input = page.getByPlaceholder(/^RID, e\.g\./);
  await input.fill(rid.replace(/^rad:/, ""));
  await waitForCommand(page, "seed", () =>
    page.getByRole("button", { name: "icon-seed Seed", exact: true }).click(),
  );

  await expect(page.getByRole("link", { name: /garden/ })).toBeVisible({
    timeout: 20_000,
  });

  await page.getByRole("button", { name: "icon-plus", exact: true }).click();
  await input.fill(rid);
  await input.press("Enter");
  await expect(
    page.getByText(/This repo is already (seeded|queued for fetching)/),
  ).toBeVisible();
});
