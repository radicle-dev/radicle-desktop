import { expect, test } from "@playwright/test";

test("navigate to repo issues", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("button", { name: "cobs" }).click();
  await page
    .getByRole("button", { name: "This title has **markdown**" })
    .click();
  await expect(
    page.getByText("This title has **markdown**").nth(1),
  ).toBeVisible();
});