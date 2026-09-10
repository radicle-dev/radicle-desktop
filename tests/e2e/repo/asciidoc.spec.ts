import { expect, test } from "@tests/support/fixtures.js";

test("render an AsciiDoc readme", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("link", { name: "asciidoc" }).click();

  const readme = page.locator(".asciidoc");

  await expect(
    readme.getByRole("heading", { name: "AsciiDoc Fixture", level: 1 }),
  ).toBeVisible();
  await expect(readme.locator("code.language-rust")).toBeVisible();
  await expect(readme.locator("table.tableblock").first()).toBeVisible();
  await expect(readme.getByText("An admonition.")).toBeVisible();
  await expect(readme.getByText("A footnote.")).toBeVisible();
});

test("open a relative link in the source view", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("link", { name: "asciidoc" }).click();

  const readme = page.locator(".asciidoc");
  await expect(readme.getByRole("link", { name: "the notes" })).toBeVisible();

  const url = page.url();
  await readme.getByRole("link", { name: "the notes" }).click();

  // The link opens the file instead of navigating the app away.
  await expect(page.getByText("Notes from a sibling file.")).toBeVisible();
  expect(page.url()).toBe(url);
});

test("keep wide AsciiDoc table cells inside the viewport", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("link", { name: "asciidoc" }).click();

  const readme = page.locator(".asciidoc");
  await expect(readme.locator("table.tableblock").nth(1)).toBeVisible();

  const overflow = await readme.evaluate(
    element => element.scrollWidth - element.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("resolve includes in an AsciiDoc readme", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("link", { name: "asciidoc" }).click();

  const readme = page.locator(".asciidoc");

  await expect(readme.getByText("Alpha section text.")).toBeVisible();
  // Only the requested tag is included.
  await expect(readme.getByText("Beta section text.")).toHaveCount(0);
  // An include may not climb out of the repository.
  await expect(readme.getByText("root:")).toHaveCount(0);
});

test("sanitize untrusted markup in an AsciiDoc readme", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("link", { name: "asciidoc" }).click();

  const readme = page.locator(".asciidoc");
  await expect(readme.getByText("Untrusted markup")).toBeVisible();

  await expect(readme.locator("script")).toHaveCount(0);
  await expect(readme.locator("iframe")).toHaveCount(0);
  await expect(readme.locator("[onerror]")).toHaveCount(0);
  await expect(readme.locator("[onclick]")).toHaveCount(0);
  await expect(readme.locator('a[href^="javascript:"]')).toHaveCount(0);

  const marker = await page.evaluate(
    () => (window as unknown as { xssMarker?: string }).xssMarker,
  );
  expect(marker).toBeUndefined();
});
