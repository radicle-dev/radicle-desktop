import { expect, test } from "@tests/support/fixtures.js";

test("create a new identity", async ({ page }) => {
  await page.goto("/");
  await page.getByPlaceholder("Enter desired alias").fill("palm");
  await page.getByPlaceholder("Enter passphrase to protect").fill("asdf");
  await page.getByPlaceholder("Repeat passphrase").fill("asdf");
  await page
    .getByRole("button", { name: "icon-seedling Create new identity" })
    .click();
  await expect(
    page.getByRole("button", {
      name: "z6MktULudTtAsAhRegYPiZ6631RV3viv12qd4GQF8z1xB22S icon-copy",
    }),
  ).toBeVisible();
});

test("validate new identity inputs", async ({ page }) => {
  await page.goto("/");
  await page.getByPlaceholder("Enter desired alias").fill("hello world");
  await expect(
    page.getByText("Alias cannot contain whitespace."),
  ).toBeVisible();

  await page.getByPlaceholder("Enter desired alias").fill("a".repeat(33));
  await expect(
    page.getByText("Alias is too long, make it less than 32 characters."),
  ).toBeVisible();

  await page.getByPlaceholder("Enter passphrase to protect").fill("asdf");
  await page.getByPlaceholder("Repeat passphrase").fill("asdfe");
  await expect(page.getByText("Passphrases don't match")).toBeVisible();

  await expect(
    page.getByRole("button", { name: "icon-seedling Create new identity" }),
  ).toHaveClass(/disabled/);
});
