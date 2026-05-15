import { describe, expect, test } from "vitest";

import { safeHttpUrl } from "@app/lib/utils";

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
