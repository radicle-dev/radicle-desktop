import { describe, expect, test } from "vitest";

import { resolveRepoPath } from "@app/lib/asciidoc";

describe("resolveRepoPath", () => {
  test.each([
    ["notes.txt", "README.adoc", "notes.txt"],
    ["./notes.txt", "README.adoc", "notes.txt"],
    ["guide.adoc", "docs/index.adoc", "docs/guide.adoc"],
    ["../README.adoc", "docs/index.adoc", "README.adoc"],
    ["../../README.adoc", "docs/index.adoc", "README.adoc"],
    ["../notes.txt", "README.adoc", "notes.txt"],
    ["/src/main.rs", "docs/index.adoc", "src/main.rs"],
    ["my%20notes.txt", "README.adoc", "my notes.txt"],
    ["notes.txt?plain=1", "README.adoc", "notes.txt"],
    ["notes.txt#usage", "README.adoc", "notes.txt"],
    ["docs/", "README.adoc", "docs/"],
  ])("resolves %j from %j to %j", (href, from, expected) => {
    expect(resolveRepoPath(href, from)).toBe(expected);
  });

  test.each([
    ["https://radicle.xyz", "README.adoc"],
    ["//radicle.xyz/notes.txt", "README.adoc"],
    ["mailto:alice@example.com", "README.adoc"],
    ["javascript:alert(1)", "README.adoc"],
    ["/", "README.adoc"],
    ["..", "README.adoc"],
  ])("leaves %j from %j outside the repository", (href, from) => {
    expect(resolveRepoPath(href, from)).toBeUndefined();
  });
});
