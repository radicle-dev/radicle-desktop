import { test, expect, cobRid } from "@tests/support/fixtures.js";

test("navigate single issue", async ({ page }) => {
  await page.goto(`/repos/${cobRid}/issues?status=all`);
  await page.getByText("This title has **markdown**").click();

  await expect(page).toHaveURL(/\/issues\/[0-9a-f]{40}/);
});
