import { describe, expect, test } from "vitest";

import type { MarkdownFormat } from "@app/lib/markdownFormat";
import {
  applyMarkdownFormat,
  applyTextEdit,
  pasteLinkEdit,
} from "@app/lib/markdownFormat";

// Runs a format over a value where the selection is marked by pipes, and
// returns the result in the same notation, so the assertions read like what
// the textarea actually shows.
function format(input: string, kind: MarkdownFormat): string {
  const start = input.indexOf("|");
  const end = input.indexOf("|", start + 1) - 1;
  const value = input.replaceAll("|", "");
  const edit = applyMarkdownFormat(kind, value, start, end);
  const result = applyTextEdit(value, edit);

  return result
    .substring(0, edit.selectionStart)
    .concat(
      "|",
      result.substring(edit.selectionStart, edit.selectionEnd),
      "|",
      result.substring(edit.selectionEnd),
    );
}

describe("bold", () => {
  test.each([
    ["a |word| here", "a **|word|** here"],
    // Collapsed selection leaves the caret between the markers.
    ["a ||word", "a **||**word"],
    // Toggles back off when the markers are just outside the selection.
    ["a **|word|** here", "a |word| here"],
    // Toggles back off when the selection covers the markers too.
    ["a |**word**| here", "a |word| here"],
    ["a **||** here", "a || here"],
  ])("%p => %p", (input, expected) => {
    expect(format(input, "bold")).toBe(expected);
  });
});

describe("italic", () => {
  test.each([
    ["a |word| here", "a _|word|_ here"],
    ["a _|word|_ here", "a |word| here"],
    ["a |_word_| here", "a |word| here"],
    // Nests inside bold rather than colliding with it.
    ["**|word|**", "**_|word|_**"],
  ])("%p => %p", (input, expected) => {
    expect(format(input, "italic")).toBe(expected);
  });
});

describe("code", () => {
  test.each([
    ["a |word| here", "a `|word|` here"],
    ["a `|word|` here", "a |word| here"],
    // A multi-line selection becomes a fenced block.
    ["|one\ntwo|", "```\n|one\ntwo|\n```"],
    ["```\n|one\ntwo|\n```", "|one\ntwo|"],
  ])("%p => %p", (input, expected) => {
    expect(format(input, "code")).toBe(expected);
  });
});

describe("link", () => {
  test.each([
    // Caret lands in the empty target.
    ["see |docs| now", "see [docs](||) now"],
    ["see || now", "see [](||) now"],
    // A selected URL becomes the target, so the caption is what's left to fill.
    ["see |https://radicle.xyz| now", "see [||](https://radicle.xyz) now"],
  ])("%p => %p", (input, expected) => {
    expect(format(input, "link")).toBe(expected);
  });
});

describe("pasteLinkEdit", () => {
  test("wraps the selection and leaves the caret after the link", () => {
    const value = "Read the docs here";
    const edit = pasteLinkEdit(value, 9, 13, "https://radicle.xyz");

    expect(applyTextEdit(value, edit)).toBe(
      "Read the [docs](https://radicle.xyz) here",
    );
    expect(edit.selectionStart).toBe(36);
    expect(edit.selectionEnd).toBe(36);
  });
});
