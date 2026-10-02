import type { Config } from "@bindings/config/Config";

import { describe, expect, test } from "vitest";

import { markdownWithExtensions } from "@app/lib/markdown";
import {
  bareReferenceStart,
  describeLink,
  matchBareReference,
  mentionHref,
  mentionMarkdown,
  mentionUrl,
  parseBareIdentifier,
  parseMentionHref,
  referenceMarkdown,
  referenceRoute,
} from "@app/lib/mentions";
import { parseReference } from "@app/lib/radUri";

const repo = "z3gqcJUoA1n9HaHKufZs5FCSGazv5";
const rid = `rad:${repo}`;
const nid = "z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi";
const oid = "5e4683023476778201ab25c87e1343fbfbe67c89";

describe("parseMentionHref", () => {
  test.each([
    [`did:key:${nid}`, { type: "node", nid }],
    [rid, { type: "repo", rid }],
    [`${rid}/commit/${oid}`, { type: "commit", rid, oid }],
    [
      `${rid}/cob/xyz.radicle.issue/${oid}`,
      { type: "cob", kind: "issue", rid, oid },
    ],
    [
      `${rid}/cob/xyz.radicle.patch/${oid}`,
      { type: "cob", kind: "patch", rid, oid },
    ],
    [
      `https://app.radicle.xyz/nodes/seed/${rid}/patches/${oid}`,
      { type: "cob", kind: "patch", rid, oid },
    ],
    [`  ${rid}  `, { type: "repo", rid }],
  ])("%s", (href, expected) => {
    expect(parseMentionHref(href)).toEqual(expected);
  });

  test.each([
    ["a tag", `${rid}/tag/v1.0`],
    ["a tree", `${rid}/tree/${oid}`],
    ["a commit by branch", `${rid}/commit/master`],
    ["an unknown cob type", `${rid}/cob/org.example/${oid}`],
    ["a cob list", `${rid}/cob/xyz.radicle.issue`],
    ["a namespace", `${rid}/${nid}`],
    ["a query", `${rid}/commit/${oid}?path=src`],
    ["a fragment", `${rid}/commit/${oid}#L10`],
    ["an unrelated link", "https://example.com"],
  ])("has no chip for %s", (_, href) => {
    expect(parseMentionHref(href)).toBeUndefined();
  });
});

describe("parseBareIdentifier", () => {
  test.each([
    [nid, { type: "node", nid }],
    [repo, { type: "repo", rid }],
    ["zlib", undefined],
    [oid, undefined],
  ])("%s", (token, expected) => {
    expect(parseBareIdentifier(token)).toEqual(expected);
  });
});

describe("bareReferenceStart", () => {
  test.each([
    [`${rid} at the start`, 0],
    [`see ${rid}`, 4],
    [`(did:key:${nid})`, 1],
    [`Conrad:${repo}`, undefined],
    [`Conrad:${repo} and ${rid}`, 41],
  ])("%s", (src, expected) => {
    expect(bareReferenceStart(src)).toBe(expected);
  });
});

describe("matchBareReference", () => {
  test.each([
    [`${rid}.`, rid],
    [`${rid}).`, rid],
    [`${rid}'s`, rid],
    [`${rid}, then`, rid],
    [`did:key:${nid}!`, `did:key:${nid}`],
    [`${rid}/commit/master?path=src.`, `${rid}/commit/master?path=src`],
  ])("%s", (src, raw) => {
    expect(matchBareReference(src)?.raw).toBe(raw);
  });

  test("leaves an invalid reference as prose", () => {
    expect(matchBareReference(`${rid}/bogus here`)).toBeUndefined();
  });

  test("skips the tail of a word", () => {
    expect(matchBareReference(rid, "n")).toBeUndefined();
    expect(matchBareReference(rid, "(")?.raw).toBe(rid);
  });
});

describe("markdown", () => {
  const render = (src: string) => markdownWithExtensions.parse(src) as string;

  test("links a bare reference", () => {
    expect(render(`See ${rid}.`)).toBe(
      `<p>See <a href="${rid}">${rid}</a>.</p>\n`,
    );
  });

  test("leaves the tail of a word alone", () => {
    expect(render(`Conrad:${repo}`)).not.toContain("<a");
  });

  test("does not nest a link inside a link label", () => {
    expect(render(`[see ${rid}](https://example.com)`)).toBe(
      `<p><a href="https://example.com">see ${rid}</a></p>\n`,
    );
  });

  test("leaves code alone", () => {
    expect(render(`\`${rid}\``)).toBe(`<p><code>${rid}</code></p>\n`);
  });
});

describe("serialising", () => {
  const issue = { type: "cob", kind: "issue", rid, oid } as const;

  test("mentionHref writes the RIP 4 form", () => {
    expect(mentionHref(issue)).toBe(`${rid}/cob/xyz.radicle.issue/${oid}`);
    expect(mentionHref({ type: "node", nid })).toBe(`did:key:${nid}`);
    expect(mentionHref({ ...issue, kind: "patch" })).toBe(
      `${rid}/cob/xyz.radicle.patch/${oid}`,
    );
    expect(mentionHref({ type: "commit", rid, oid })).toBe(
      `${rid}/commit/${oid}`,
    );
  });

  test("referenceMarkdown links any reference", () => {
    expect(
      referenceMarkdown(
        {
          type: "uri",
          uri: { repo, resource: { type: "commit", ref: "main" } },
        },
        "heartwood: README",
      ),
    ).toBe(`[heartwood: README](${rid}/commit/main)`);
  });

  test("mentionMarkdown escapes the label", () => {
    expect(mentionMarkdown(issue, "Fix [x] \\ y")).toBe(
      `[Fix \\[x\\] \\\\ y](${rid}/cob/xyz.radicle.issue/${oid})`,
    );
  });

  test("mentionUrl uses the configured explorer and preferred seed", () => {
    const config = {
      publicExplorer: "https://explorer.example/nodes/$host/$rid$path",
      preferredSeeds: [`${nid}@seed.example.com:8776`],
    } as Config;

    expect(mentionUrl(issue, config)).toBe(
      `https://explorer.example/nodes/seed.example.com/${rid}/issues/${oid}`,
    );
    expect(mentionUrl({ type: "node", nid }, config)).toBe(
      `https://explorer.example/nodes/seed.example.com/users/did:key:${nid}`,
    );
  });

  test("mentionUrl keeps a path prefix in the explorer URL", () => {
    const config = {
      publicExplorer: "https://example.com/explorer/nodes/$host/$rid$path",
      preferredSeeds: [],
    } as unknown as Config;

    expect(mentionUrl({ type: "repo", rid }, config)).toBe(
      `https://example.com/explorer/nodes/rosa.radicle.network/${rid}`,
    );
  });
});

describe("referenceRoute", () => {
  test.each([
    [
      `${rid}/commit/master?path=.github/README.md`,
      {
        resource: "repo.home",
        rid,
        revision: "master",
        path: ".github/README.md",
      },
    ],
    [
      `${rid}/commit/main?path=My%20File.md`,
      { resource: "repo.home", rid, revision: "main", path: "My File.md" },
    ],
    [
      `${rid}/commit/refs/heads/main?path=`,
      { resource: "repo.home", rid, revision: "main" },
    ],
    [
      `${rid}/${nid}/commit/main?path=src`,
      { resource: "repo.home", rid, peer: nid, revision: "main", path: "src" },
    ],
    [`${rid}/tag/v1.0`, { resource: "repo.home", rid, revision: "v1.0" }],
    [`${rid}/${nid}`, { resource: "repo.home", rid, peer: nid }],
    [
      `${rid}/cob/xyz.radicle.issue`,
      { resource: "repo.issues", rid, status: "all" },
    ],
    [
      `${rid}/cob/xyz.radicle.patch`,
      { resource: "repo.patches", rid, status: undefined },
    ],
  ])("%s", (href, expected) => {
    expect(referenceRoute(parseReference(href)!)).toEqual(expected);
  });

  test.each([
    `${rid}/cob/dev.radicle.artifact`,
    `${rid}/tree/${oid}`,
    `did:key:${nid}`,
  ])("has no page for %s", href => {
    expect(referenceRoute(parseReference(href)!)).toBeUndefined();
  });
});

describe("describeLink", () => {
  test.each([
    [`${rid}/commit/main?path=src/lib.rs`, "heartwood: src/lib.rs", "document"],
    [
      `${rid}/commit/main?path=src/lib.rs#L10`,
      "heartwood: src/lib.rs#L10",
      "document",
    ],
    [`${rid}/commit/main?path=`, "heartwood: main", "folder"],
    [`${rid}/commit/${oid}?path=`, "heartwood: 5e46830", "folder"],
    [`${rid}/commit/refs/heads/main`, "heartwood: main", "branch"],
    [`${rid}/tag/v1.0`, "heartwood: v1.0", "label"],
    [`${rid}/cob/xyz.radicle.issue`, "heartwood: issues", "issue"],
    [`${rid}/cob/xyz.radicle.patch`, "heartwood: patches", "patch"],
    [`${rid}/cob/dev.radicle.artifact`, "heartwood: releases", "archive"],
    [
      `${rid}/cob/dev.radicle.artifact/${oid}`,
      "heartwood: release 5e46830",
      "archive",
    ],
    [`${rid}/${nid}`, "heartwood: z6MknS…tBU8Vi", "repository"],
  ])("%s", (href, label, icon) => {
    expect(describeLink(parseReference(href)!, "heartwood")).toMatchObject({
      label,
      icon,
    });
  });

  test("names a remote by its alias", () => {
    expect(
      describeLink(parseReference(`${rid}/${nid}`)!, "heartwood", "cloudhead"),
    ).toMatchObject({ label: "heartwood: cloudhead" });
  });

  test.each([
    rid,
    `${rid}/commit/${oid}`,
    `${rid}/cob/xyz.radicle.issue/${oid}`,
    `${rid}/tree/${oid}`,
    `did:key:${nid}`,
  ])("is not a link for %s", href => {
    expect(describeLink(parseReference(href)!, "heartwood")).toBeUndefined();
  });
});
