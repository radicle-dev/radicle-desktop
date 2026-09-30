import { expect, test } from "@tests/support/fixtures.js";

test("explains why an alias is invalid", async ({ page }) => {
  await page.route(
    url => url.pathname === "/startup",
    route =>
      route.fulfill({
        status: 500,
        json: {
          code: "IdentityError.MissingProfile",
          message: "No profile found",
        },
      }),
  );
  await page.goto("/");

  const alias = page.getByPlaceholder("Enter desired alias");
  await alias.fill("has space");
  await expect(
    page.getByText("Alias cannot contain whitespace."),
  ).toBeVisible();

  await alias.fill("a".repeat(33));
  await expect(
    page.getByText("Alias is too long, max 32 characters."),
  ).toBeVisible();
  await expect(page.getByText("Alias cannot contain whitespace.")).toBeHidden();

  await alias.fill("alice");
  await expect(
    page.getByText("Max 32 characters, no whitespace."),
  ).toBeVisible();
});
