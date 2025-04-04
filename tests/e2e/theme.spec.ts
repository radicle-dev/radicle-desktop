import { test, expect } from "@tests/support/fixtures.js";

test("default theme", async ({ authenticatedContext: page }) => {
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
});

test("theme persistence", async ({ authenticatedContext: page }) => {
  await page.getByRole("button", { name: "Settings" }).click();

  await page
    .getByRole("button", { name: "icon-sun Light", exact: true })
    .click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");

  await page.reload();
  // Making sure the page view has reloaded and we see some content.
  await expect(page.getByText("Repositories").nth(1)).toBeVisible();

  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
});

test("change theme", async ({ authenticatedContext: page }) => {
  await page.getByRole("button", { name: "Settings" }).click();

  await page
    .getByRole("button", { name: "icon-sun Light", exact: true })
    .click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");

  await page
    .getByRole("button", { name: "icon-moon Dark", exact: true })
    .click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
});
