import { expect, playgroundRid, test } from "@tests/support/fixtures.js";

test("create and interact with issues", async ({
  authenticatedContext: page,
}) => {
  await test.step("create a new repo", async () => {
    await page
      .getByRole("button", { name: "Create a new repo", exact: true })
      .click();
    await page.getByPlaceholder("Name of your repo").fill("playground");
    await page.getByPlaceholder("Add description").fill("Lorem ipsum dolor");
    await page
      .getByRole("button", { name: "Create new repo", exact: true })
      .click();
    await page.getByRole("button", { name: "p playground" }).click();
    await expect(page.getByText(playgroundRid)).toBeVisible();
  });

  await test.step("create a new issue", async () => {
    await page.getByRole("button", { name: "icon-plus New" }).click();
    await page.getByPlaceholder("Title").fill("Add missing issue");
    await page.getByPlaceholder("Description").fill(
      `Lorem ipsum dolor sit amet, consetetur sadipscing elitr,
 sed diam nonumy eirmod tempor invidunt ut labore et dolore magna
 aliquyam erat, sed diam voluptua.`,
    );
    await page.getByRole("button", { name: "icon-checkmark Save" }).click();
    await expect(
      page.getByText("c5f47493484e4b1696bfa0bdad21ce2ae439e4f0"),
    ).toBeVisible();
  });

  await test.step("edit created issue", async () => {
    await page.getByLabel("edit-title").click();
    await page.getByPlaceholder("Issue title").fill("Add another issue");
    await page.getByLabel("save-new-title").click();
  });

  await test.step("change issue lifecycle", async () => {
    await expect(page.getByText("Open", { exact: true })).toBeVisible();
    await page.getByLabel("toggle-issue-state").click();
    await page
      .getByRole("button", {
        name: "icon-issue-closed Closed as solved",
        exact: true,
      })
      .click();
    await expect(page.getByText("Closed as solved")).toBeVisible();
    await page.getByLabel("toggle-issue-state").click();
    await page
      .getByRole("button", { name: "icon-issue Open", exact: true })
      .click();
    await expect(
      page.getByLabel("toggle-issue-state").filter({ hasText: "Open" }),
    ).toBeVisible();
  });

  await test.step("create a top level comment", async () => {
    await page.getByRole("button", { name: "icon-comment Comment" }).click();
    await expect(page.getByPlaceholder("Leave a comment")).toBeVisible();
    await page.getByPlaceholder("Leave a comment").fill("Lorem ipsum dolor.");
    await page.getByRole("button", { name: "icon-checkmark Comment" }).click();
    await expect(page.getByText("Lorem ipsum dolor.")).toBeVisible();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Comment" }),
    ).toBeHidden();
  });

  await test.step("create a reply comment", async () => {
    await page.getByLabel("create-top-level-reply").click();
    await page
      .getByPlaceholder("Reply to comment")
      .fill("This is a reply comment");
    await page.getByRole("button", { name: "icon-checkmark Reply" }).click();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Reply" }),
    ).toBeHidden();
  });

  await test.step("edit top level comment", async () => {
    await page.getByLabel("edit-top-level-comment").click();
    await page
      .getByPlaceholder("Leave a comment")
      .fill("Lorem ipsum dolor sit anem.");
    await page.getByRole("button", { name: "icon-checkmark Save" }).click();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Save" }),
    ).toBeHidden();
  });

  await test.step("edit reply comment", async () => {
    await page.getByLabel("edit-reply-comment").click();
    await page
      .getByPlaceholder("Leave a comment")
      .last()
      .fill("This maybe is a reply comment.");
    await page.getByRole("button", { name: "icon-checkmark Save" }).click();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Save" }),
    ).toBeHidden();
  });

  await test.step("react to a top level comment", async () => {
    await page.getByLabel("toggle-reaction-selector").nth(1).click();
    await page.getByLabel("reaction-selector-👍").click();
  });

  // This is not working yet, due to the reaction selector being hidden in the reply comment.
  // await test.step("react to a reply comment", async () => {
  //   await page.getByLabel("icon-face").nth(3).click();
  //   await page.getByRole("button", { name: "🎉", exact: true }).click();
  // });
});
