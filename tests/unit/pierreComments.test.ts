import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
import type { CodeRange } from "@bindings/cob/thread/CodeRange";

import { describe, expect, test } from "vitest";

import type { ComposerTarget } from "@app/lib/pierreComments";
import {
  anchorOf,
  commentCountsByPath,
  fileAnnotations,
  formatAnchor,
  formatAnchorLines,
  isCommentableStatus,
  locationOf,
} from "@app/lib/pierreComments";

import { comment, location, thread } from "./support/cobs";

function lines(start: number, end: number): CodeRange {
  return { type: "lines", range: { start, end } };
}

function at(
  path: string,
  props: { old?: CodeRange; new?: CodeRange },
): CodeLocation {
  return {
    commit: "c".repeat(40),
    path,
    old: props.old ?? null,
    new: props.new ?? null,
  };
}

describe("anchorOf", () => {
  test("anchors a new-side range under its last line", () => {
    expect(anchorOf(at("a.ts", { new: lines(3, 8) }))).toEqual({
      path: "a.ts",
      side: "additions",
      line: 7,
    });
  });

  test("anchors an old-side range on the deletions side", () => {
    expect(anchorOf(at("a.ts", { old: lines(5, 6) }))).toEqual({
      path: "a.ts",
      side: "deletions",
      line: 5,
    });
  });

  test("prefers the new side when a location has both", () => {
    expect(
      anchorOf(at("a.ts", { old: lines(1, 2), new: lines(9, 10) }))?.side,
    ).toBe("additions");
  });

  test("places a chars range on its line", () => {
    expect(
      anchorOf(
        at("a.ts", {
          new: { type: "chars", line: 4, range: { start: 0, end: 2 } },
        }),
      )?.line,
    ).toBe(4);
  });

  test.each([null, undefined, at("a.ts", {})])(
    "has no anchor for %j",
    value => {
      expect(anchorOf(value)).toBeUndefined();
    },
  );
});

describe("locationOf", () => {
  const target: ComposerTarget = {
    path: "a.ts",
    side: "additions",
    firstLine: 4,
    lastLine: 6,
  };

  test("writes the range on the composer's side", () => {
    expect(locationOf("abc", target)).toEqual({
      commit: "abc",
      path: "a.ts",
      old: null,
      new: lines(4, 7),
    });
  });

  test("writes a deletions range on the old side", () => {
    expect(locationOf("abc", { ...target, side: "deletions" })).toEqual({
      commit: "abc",
      path: "a.ts",
      old: lines(4, 7),
      new: null,
    });
  });

  test("normalises a range dragged upwards", () => {
    expect(
      locationOf("abc", { ...target, firstLine: 6, lastLine: 4 }).new,
    ).toEqual(lines(4, 7));
  });

  test("round-trips through anchorOf to the composer's last line", () => {
    expect(anchorOf(locationOf("abc", target))).toEqual({
      path: "a.ts",
      side: "additions",
      line: 6,
    });
  });
});

describe("formatAnchorLines", () => {
  test.each<[CodeLocation, string | undefined]>([
    [at("a.ts", { new: lines(12, 13) }), "R12"],
    [at("a.ts", { old: lines(3, 10) }), "L3-L9"],
    [at("a.ts", { new: lines(3, 10) }), "R3-R9"],
    [
      at("a.ts", {
        old: { type: "chars", line: 7, range: { start: 1, end: 4 } },
      }),
      "L7",
    ],
    [at("a.ts", {}), undefined],
  ])("formats %j as %j", (value, expected) => {
    expect(formatAnchorLines(value)).toBe(expected);
  });

  test("formatAnchor prefixes the path", () => {
    expect(formatAnchor(at("src/a.ts", { old: lines(3, 10) }))).toBe(
      "src/a.ts:L3-L9",
    );
    expect(formatAnchor(at("src/a.ts", {}))).toBe("src/a.ts");
  });
});

test.each([
  ["added", true],
  ["deleted", true],
  ["modified", true],
  [undefined, true],
  ["moved", false],
  ["copied", false],
] as const)("isCommentableStatus(%j) is %j", (status, expected) => {
  expect(isCommentableStatus(status)).toBe(expected);
});

describe("fileAnnotations", () => {
  const onLine5 = location("a.ts", "new", 5);

  test("keeps only this file's threads, one annotation per side and line", () => {
    const annotations = fileAnnotations(
      "a.ts",
      [
        thread(comment({ id: "1", location: onLine5 })),
        thread(comment({ id: "2", location: location("a.ts", "old", 5) })),
        thread(comment({ id: "3", location: location("b.ts", "new", 5) })),
        thread(comment({ id: "4", location: null })),
      ],
      undefined,
    );

    expect(
      annotations.map(a => [
        a.side,
        a.lineNumber,
        a.metadata.threads.map(t => t.root.id),
      ]),
    ).toEqual([
      ["additions", 5, ["1"]],
      ["deletions", 5, ["2"]],
    ]);
  });

  test("orders threads on the same line oldest first", () => {
    const [annotation] = fileAnnotations(
      "a.ts",
      [
        thread(comment({ id: "new", location: onLine5, timestamp: 20 })),
        thread(comment({ id: "old", location: onLine5, timestamp: 10 })),
      ],
      undefined,
    );

    expect(annotation.metadata.threads.map(t => t.root.id)).toEqual([
      "old",
      "new",
    ]);
  });

  test("puts the composer on its last line, sharing a slot with threads", () => {
    const composer: ComposerTarget = {
      path: "a.ts",
      side: "additions",
      firstLine: 2,
      lastLine: 5,
    };
    const annotations = fileAnnotations(
      "a.ts",
      [thread(comment({ id: "1", location: onLine5 }))],
      composer,
    );

    expect(annotations).toHaveLength(1);
    expect(annotations[0].metadata.composer).toBe(composer);
    expect(annotations[0].metadata.threads).toHaveLength(1);
  });

  test("ignores a composer in another file", () => {
    const annotations = fileAnnotations("a.ts", [], {
      path: "b.ts",
      side: "additions",
      firstLine: 1,
      lastLine: 1,
    });

    expect(annotations).toEqual([]);
  });
});

describe("commentCountsByPath", () => {
  test("counts resolved and unresolved threads per file", () => {
    const counts = commentCountsByPath(
      [
        thread(comment({ id: "1", location: location("a.ts", "new", 1) })),
        thread(
          comment({
            id: "2",
            location: location("a.ts", "new", 2),
            resolved: true,
          }),
        ),
        thread(comment({ id: "3", location: location("b.ts", "old", 1) })),
        thread(comment({ id: "4", location: null })),
      ],
      () => true,
    );

    expect(Object.fromEntries(counts)).toEqual({
      "a.ts": { resolved: 1, unresolved: 1 },
      "b.ts": { resolved: 0, unresolved: 1 },
    });
  });

  test("skips threads that can't be resolved", () => {
    const counts = commentCountsByPath(
      [
        thread(comment({ id: "draft", location: location("a.ts", "new", 1) })),
        thread(comment({ id: "real", location: location("a.ts", "new", 2) })),
      ],
      id => id !== "draft",
    );

    expect(counts.get("a.ts")).toEqual({ resolved: 0, unresolved: 1 });
  });
});
