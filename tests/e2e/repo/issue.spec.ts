import { test, expect, cobRid } from "@tests/support/fixtures.js";

test("navigate single issue", async ({ page }) => {
  await page.goto(`/repos/${cobRid}/issues?status=all`);
  await page.getByText("This title has **markdown**").click();

  await expect(page).toHaveURL(/\/issues\/[0-9a-f]{40}/);
});

test("correct order of threads", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("button", { name: "cobs" }).click();
  await page.getByText("This title has **markdown**").click();
  const body = page.locator(".issue-body");
  await expect(body.getByText("This is a description")).toBeVisible();

  const topLevelComments = await page.locator(".comments").all();
  expect(topLevelComments).toHaveLength(2);

  const [first, second] = topLevelComments;
  await expect(first.getByText("This is a multiline comment")).toBeVisible();
  await expect(
    first.getByText("This is a reply, to a first comment"),
  ).toBeVisible();
  await expect(
    second.getByText("A root level comment after a reply, for margins sake."),
  ).toBeVisible();
});

test("creation of top level comments", async ({ page }) => {
  await page.goto("/repos");
  await page.getByRole("button", { name: "cobs" }).click();
  await page.getByRole("button", { name: "New" }).click();
  await page
    .getByPlaceholder("Title")
    .fill("Make sure that comment creation is working");
  await page
    .getByPlaceholder("Description")
    .fill(
      "It's important for us that the comment creation flow works as expected.",
    );
  await page.getByRole("button", { name: "icon-checkmark" }).click();
  await expect(
    page.getByRole("button", { name: "icon-issue Make sure that" }),
  ).toBeVisible();
  await expect(
    page
      .getByText(
        "It's important for us that the comment creation flow works as expected.",
      )
      .last(),
  ).toBeVisible();

  await page.getByRole("button", { name: "icon-comment Comment" }).click();
  await page
    .getByPlaceholder("Leave a comment")
    .fill("A top level comment by playwright");
  await page.getByRole("button", { name: "icon-checkmark" }).click();
  await expect(
    page.getByText("A top level comment by playwright"),
  ).toBeVisible();

  await page.getByLabel("icon-reply").first().click();
  await page
    .getByPlaceholder("Reply to comment")
    .fill(
      "A top level comment by playwright created by replying to the issue body",
    );
  await page.getByRole("button", { name: "icon-checkmark" }).click();
  await expect(
    page.getByText(
      "A top level comment by playwright created by replying to the issue body",
    ),
  ).toBeVisible();

  await page.getByLabel("icon-reply").click();
  await page
    .getByPlaceholder("Reply to comment")
    .fill("A reply comment by playwright replying to the first comment");
  await page.getByRole("button", { name: "icon-checkmark" }).click();
  await expect(
    page.getByText(
      "A reply comment by playwright replying to the first comment",
    ),
  ).toBeVisible();
});
