import { describe, expect, test } from "vitest";

import { findMentionTrigger } from "@app/lib/mentionTrigger";

const repo = "z3gqcJUoA1n9HaHKufZs5FCSGazv5";
const rid = `rad:${repo}`;
const nid = "z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi";
const oid = "5e4683023476778201ab25c87e1343fbfbe67c89";

function trigger(text: string, caret = text.length) {
  return findMentionTrigger(text, caret);
}

describe("queries", () => {
  test.each([
    ["@", { scope: "all", start: 0, query: "" }],
    ["#", { scope: "cobs", start: 0, query: "" }],
    ["hi @rud", { scope: "all", start: 3, query: "rud" }],
    ["(#fix", { scope: "cobs", start: 1, query: "fix" }],
    ["line\n@a.b-c", { scope: "all", start: 5, query: "a.b-c" }],
  ])("%j", (text, expected) => {
    expect(trigger(text)).toEqual({
      kind: "query",
      end: text.length,
      ...expected,
    });
  });

  test.each([
    ["an email address", "me@example"],
    ["a URL fragment", "page#section"],
    ["a heading", "# Heading"],
    ["a query past the limit", `@${"a".repeat(65)}`],
  ])("ignores %s", (_, text) => {
    expect(trigger(text)).toBeUndefined();
  });

  test("only considers text up to the caret", () => {
    expect(trigger("@rud and more", 4)).toMatchObject({ query: "rud" });
    expect(trigger("@rud and more")).toBeUndefined();
  });
});

describe("identifiers", () => {
  test.each([
    [`see did:key:${nid}`, 4, { type: "node", nid }],
    [`(${rid}`, 1, { type: "repo", rid }],
    [
      `https://app.radicle.xyz/nodes/seed/${rid}/issues/${oid}`,
      0,
      { type: "cob", kind: "issue", rid, oid },
    ],
    [`https://app.radicle.xyz/nodes/seed/${rid}`, 0, { type: "repo", rid }],
    [nid, 0, { type: "node", nid }],
    [`x ${repo}`, 2, { type: "repo", rid }],
  ])("%s", (text, start, target) => {
    expect(trigger(text)).toEqual({
      kind: "identifier",
      start,
      end: text.length,
      target,
    });
  });

  test("does not reopen on an inserted link", () => {
    expect(trigger(`[heartwood](${rid})`)).toBeUndefined();
  });

  test("offers a full oid, lowercased", () => {
    expect(trigger(`see ${oid.toUpperCase()}`)).toEqual({
      kind: "oid",
      start: 4,
      end: 44,
      oid,
    });
  });

  test("ignores an abbreviated oid", () => {
    expect(trigger(oid.slice(0, 39))).toBeUndefined();
  });
});
