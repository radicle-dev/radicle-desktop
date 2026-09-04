import { describe, expect, test } from "vitest";

import { coAuthors, safeHttpUrl } from "@app/lib/utils";

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

describe("coAuthors", () => {
  test("reads trailers from the final paragraph", () => {
    expect(
      coAuthors(
        "Fix the thing\n\nSome body text.\n\nSigned-off-by: A <a@x>\nCo-authored-by: Alice Lidell <alice@lidell.com>\nCo-authored-by: Bob <bob@example.com>",
      ),
    ).toEqual([
      { name: "Alice Lidell", email: "alice@lidell.com" },
      { name: "Bob", email: "bob@example.com" },
    ]);
  });

  test("is case-insensitive", () => {
    expect(
      coAuthors("Subject\n\nCO-AUTHORED-BY: Bob <bob@example.com>"),
    ).toEqual([{ name: "Bob", email: "bob@example.com" }]);
  });

  test("ignores a trailer that is not in the last paragraph", () => {
    expect(
      coAuthors(
        "Subject\n\nCo-authored-by: Bob <bob@example.com>\n\nThat line was quoted, not a trailer.",
      ),
    ).toEqual([]);
  });

  test("ignores malformed lines", () => {
    expect(
      coAuthors(
        "Subject\n\nCo-authored-by: Bob\nCo-authored-by:\nNot-a-trailer: x <y@z>",
      ),
    ).toEqual([]);
  });

  test("returns nothing when there are no trailers", () => {
    expect(coAuthors("Subject\n\nJust a body.")).toEqual([]);
  });
});
