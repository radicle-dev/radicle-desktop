import type { State } from "@bindings/identity/State";

import { expect, test } from "vitest";

import { stateCaption, stateIcon, stateLabel } from "@app/lib/identityState";

const sibling = "4b2d0a1e9f5c3b7a6d8e0f1a2b3c4d5e6f7a8b9c";

test.each<[State, string, string, string | undefined]>([
  [{ status: "active" }, "hourglass", "Active", undefined],
  [{ status: "accepted" }, "checkmark", "Accepted", undefined],
  [
    { status: "rejected", reason: { type: "vote" } },
    "close",
    "Rejected",
    "Rejected by delegate votes",
  ],
  [
    { status: "rejected", reason: { type: "parent" } },
    "close",
    "Rejected",
    "Rejected because its parent revision was rejected",
  ],
  [
    { status: "rejected", reason: { type: "sibling", revision: sibling } },
    "close",
    "Rejected",
    "Rejected because sibling 4b2d0a1 was accepted",
  ],
])("%j", (state, icon, label, caption) => {
  expect(stateIcon(state)).toBe(icon);
  expect(stateLabel(state)).toBe(label);
  expect(stateCaption(state)).toBe(caption);
});
