import { expect, test } from "vitest";

import { badgeCount } from "@app/lib/notificationCount.svelte";

test.each([
  [3, true, 3],
  [0, true, undefined],
  [3, false, undefined],
])("%j notifications with the badge on %j show %j", (count, on, expected) => {
  expect(badgeCount(count, on)).toBe(expected);
});
