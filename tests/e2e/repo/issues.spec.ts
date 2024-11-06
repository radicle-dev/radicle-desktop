import { test, cobRid, expect } from "@tests/support/fixtures.js";

test("navigate issues listing", async ({ page }) => {
  await page.goto(`/repos/${cobRid}/issues?show=all`);
  await page.getByRole("link", { name: "Closed" }).click();
  await expect(page.locator(".issue-teaser")).toHaveCount(2);
  await expect(page).toHaveURL(`/repos/${cobRid}/issues?status=closed`);
});
