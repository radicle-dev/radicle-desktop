import { test, expect } from "@tests/support/fixtures.js";

test("removing identities from ssh-agent and re-adding them", async ({
  page,
  peer,
}) => {
  await page.goto("/");
  await expect(
    page.getByText("did:key:z6MktULudTtAsAhRegYPiZ6631RV3viv12qd4GQF8z1xB22S"),
  ).toBeVisible();

  await peer.logOut();
  await expect(
    page.getByText("Not able to find your keys in the ssh agent"),
  ).toBeVisible();

  await peer.authenticate();
  await expect(
    page.getByText("did:key:z6MktULudTtAsAhRegYPiZ6631RV3viv12qd4GQF8z1xB22S"),
  ).toBeVisible();
});
