import { cobRid, expect, test } from "@tests/support/fixtures.js";

test("create and view a release", async ({ page }) => {
  await page.goto(`/repos/${cobRid}/releases`);

  // Open the inline create form.
  await page.getByRole("button", { name: "New release" }).click();

  // Pick the most recent commit from the picker suggestions.
  await page.getByPlaceholder("Search commits or paste an OID").click();
  await page.locator(".menu .item").first().click();

  // Stage an artifact via the stubbed file picker and wait for its CID to be
  // computed (blob CIDs are base32 CIDv1, prefixed "ba").
  await page.getByRole("button", { name: "Add files…" }).click();
  await expect(page.locator(".file-row .cid")).toHaveText(/^ba/, {
    timeout: 10000,
  });

  // Publish and land on the release detail view.
  await page.getByRole("button", { name: "Publish release" }).click();
  await expect(page).toHaveURL(
    new RegExp(`/repos/${cobRid}/releases/[0-9a-f]+`),
  );

  // The published artifact is shown on the detail page.
  await expect(
    page.getByText("radicle-desktop-e2e-artifact.txt"),
  ).toBeVisible();
});
