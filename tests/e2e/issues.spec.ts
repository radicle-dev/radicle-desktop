import { expect, test } from "@tests/support/fixtures.js";

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
    await page.pause();
    await page.getByRole("button", { name: "p playground" }).click();
    await expect(
      page.getByText("rad:z45sg16ehdfh9FqCj1GqE1qB7LLam"),
    ).toBeVisible();
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
      page.getByText("5c1443ea118cbe229ac5eeb0386fef2b960740de"),
    ).toBeVisible();
  });

  await test.step("edit created issue", async () => {
    await page.getByLabel("icon-pen").first().click();
    await page.getByPlaceholder("Issue title").fill("Add another issue");
    await page.getByLabel("icon-checkmark").click();
  });
  await test.step("change issue lifecycle", async () => {
    await page.getByText("Close as solved").click();
    await page
      .getByRole("button", { name: "icon-chevron-down", exact: true })
      .click();
    await expect(page.getByText("Reopen").first()).toBeVisible();
    await page.getByRole("button", { name: "Reopen" }).nth(2).click();
    await page.getByText("Reopen").first().click();
  });
  await test.step("create a top level comment", async () => {
    await page.getByRole("button", { name: "icon-comment Comment" }).click();
    await page.getByPlaceholder("Leave a comment").fill("Lorem ipsum dolor.");
    await page.getByRole("button", { name: "icon-checkmark Comment" }).click();
    await expect(page.getByText("Lorem ipsum dolor.")).toBeVisible();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Comment" }),
    ).toBeHidden();
  });
  await test.step("create a reply comment", async () => {
    await page.getByLabel("icon-reply").click();
    await page
      .getByPlaceholder("Reply to comment")
      .fill("This is a reply comment");
    await page.getByRole("button", { name: "icon-checkmark Reply" }).click();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Reply" }),
    ).toBeHidden();
  });
  await test.step("edit top level comment", async () => {
    await page.getByLabel("icon-pen").nth(2).click();
    await page
      .getByPlaceholder("Leave a comment")
      .fill("Lorem ipsum dolor sit anem.");
    await page.getByRole("button", { name: "icon-checkmark Save" }).click();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Save" }),
    ).toBeHidden();
  });
  await test.step("edit reply comment", async () => {
    await page.getByLabel("icon-pen").nth(3).click();
    await page
      .getByPlaceholder("Leave a comment")
      .fill("This maybe is a reply comment.");
    await page.getByRole("button", { name: "icon-checkmark Save" }).click();
    await expect(
      page.getByRole("button", { name: "icon-checkmark Save" }),
    ).toBeHidden();
  });
  await test.step("react to a top level comment", async () => {
    await page.getByLabel("icon-face").nth(1).click();
    await page.getByRole("button", { name: "👍", exact: true }).click();
  });
  // This is not working yet, due to the reaction selector being hidden in the reply comment.
  // await test.step("react to a reply comment", async () => {
  //   await page.getByLabel("icon-face").nth(3).click();
  //   await page.getByRole("button", { name: "🎉", exact: true }).click();
  // });
});
