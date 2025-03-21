import { chromium } from "playwright";

import { defaultHttpdPort, expect, test } from "@tests/support/fixtures.js";

// We explicitly run all clipboard tests withing the context of a single test
// so that we don't run into race conditions, because there is no way to isolate
// the clipboard in Playwright yet.
test("copy to clipboard", async ({ peer }) => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  const page = await context.newPage();

  await page.addInitScript(
    port => localStorage.setItem("TEST_HTTP_API_PORT", port.toString()),
    peer.httpdBaseUrl?.port || defaultHttpdPort,
  );

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

  // Reset system clipboard to a known state.
  await page.evaluate<string>("navigator.clipboard.writeText('')");

  // Repo ID.
  {
    await page
      .getByText("z6MktULudTtAsAhRegYPiZ6631RV3viv12qd4GQF8z1xB22S")
      .click();
    const clipboardContent = await page.evaluate<string>(
      "navigator.clipboard.readText()",
    );
    expect(clipboardContent).toBe(
      "z6MktULudTtAsAhRegYPiZ6631RV3viv12qd4GQF8z1xB22S",
    );
  }

  // Clear the system clipboard contents so developers don't wonder why there's
  // random stuff in their clipboard after running tests.
  await page.evaluate<string>("navigator.clipboard.writeText('')");
});
