import { describe, expect, test } from "vitest";

import { pastedLinkUrl, safeHttpUrl } from "@app/lib/utils";

describe("safeHttpUrl", () => {
  test.each([
    ["http://example.com/", "http://example.com/"],
    ["https://example.com/path?a=1&b=2", "https://example.com/path?a=1&b=2"],
    ["HTTP://EXAMPLE.com/Path", "http://example.com/Path"],
    ["http://example.com/<script>", "http://example.com/%3Cscript%3E"],
  ])("accepts %p", (input, expected) => {
    expect(safeHttpUrl(input)).toBe(expected);
  });

  test.each([
    "javascript:alert(1)",
    "\tjavascript:alert(1)",
    "data:text/html,<script>alert(1)</script>",
    "vbscript:msgbox(1)",
    "file:///etc/passwd",
    "//example.com",
    "/relative/path",
    "relative",
    "",
    "not a url",
  ])("rejects %p", input => {
    expect(safeHttpUrl(input)).toBeUndefined();
  });
});

describe("pastedLinkUrl", () => {
  test.each([
    ["https://example.com", "https://example.com"],
    ["http://example.com/a?b=1#c", "http://example.com/a?b=1#c"],
    ["  https://example.com/path  ", "https://example.com/path"],
    [
      "https://app.radicle.xyz/nodes/iris.radicle.xyz/rad:z4D5U/issues/7bdaa5a",
      "https://app.radicle.xyz/nodes/iris.radicle.xyz/rad:z4D5U/issues/7bdaa5a",
    ],
  ])("accepts %p", (input, expected) => {
    expect(pastedLinkUrl(input)).toBe(expected);
  });

  test.each([
    "javascript:alert(1)",
    "data:text/html,<script>alert(1)</script>",
    "file:///etc/passwd",
    "rad:z4D5UCArafTzTQpDZNQRuqswh3ury",
    "https://example.com and more",
    "https://example.com\nhttps://example.org",
    "just some words",
    "  ",
    "",
  ])("rejects %p", input => {
    expect(pastedLinkUrl(input)).toBeUndefined();
  });
});
