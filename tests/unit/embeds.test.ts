import { describe, expect, test } from "vitest";

import { basename, embedPreviewKind } from "@app/lib/embeds";

describe("basename", () => {
  test.each([
    ["/home/alice/cat.png", "cat.png"],
    ["cat.png", "cat.png"],
  ])("of %j is %j", (path, expected) => {
    expect(basename(path)).toBe(expected);
  });
});

test.each([
  ["image/png", "image"],
  ["application/pdf", "document"],
  ["video/mp4", "video"],
  ["audio/ogg", "audio"],
  ["text/plain", undefined],
  [null, undefined],
])("a %j embed previews as %j", (mimeType, expected) => {
  expect(embedPreviewKind(mimeType)).toBe(expected);
});
