import type { Diff } from "@bindings/diff/Diff";
import type { FileDiff } from "@bindings/diff/FileDiff";
import type { Modification } from "@bindings/diff/Modification";

// A changeset flattened into a single stream of rows so it can be rendered
// through one virtualizer. Collapsed files contribute only their header row.
export type DiffRow =
  // Spacing between file cards; its own row so file-header cards start flush at
  // the row top and line up exactly with the sticky header bar.
  | { type: "file-gap"; fileIndex: number }
  | {
      type: "file-header";
      fileIndex: number;
      file: FileDiff;
      // No body rows follow (collapsed), so the header is a self-contained
      // card and needs a full border.
      standalone: boolean;
    }
  | {
      type: "hunk-header";
      fileIndex: number;
      hunkIndex: number;
      header: string;
    }
  // The file's only body row when there are no hunks to render, telling the
  // user why (binary content vs. an empty file) instead of a bare header.
  | {
      type: "file-note";
      fileIndex: number;
      file: FileDiff;
      note: "binary" | "empty";
    }
  | {
      type: "line";
      fileIndex: number;
      hunkIndex: number;
      lineIndex: number;
      file: FileDiff;
      modification: Modification;
      // Last line of the file, so it closes the card (bottom border + radius).
      last: boolean;
    };

export function flattenDiff(
  diff: Diff,
  isExpanded: (fileIndex: number) => boolean,
): DiffRow[] {
  const rows: DiffRow[] = [];
  diff.files.forEach((file, fileIndex) => {
    if (fileIndex > 0) {
      rows.push({ type: "file-gap", fileIndex });
    }
    const expanded = isExpanded(fileIndex);
    rows.push({ type: "file-header", fileIndex, file, standalone: !expanded });
    if (!expanded) {
      return;
    }
    if (file.diff.type !== "plain" || file.diff.hunks.length === 0) {
      rows.push({
        type: "file-note",
        fileIndex,
        file,
        note: file.diff.type === "binary" ? "binary" : "empty",
      });
      return;
    }

    const lineRows: DiffRow[] = [];
    file.diff.hunks.forEach((hunk, hunkIndex) => {
      lineRows.push({
        type: "hunk-header",
        fileIndex,
        hunkIndex,
        header: hunk.header,
      });
      hunk.lines.forEach((modification, lineIndex) => {
        lineRows.push({
          type: "line",
          fileIndex,
          hunkIndex,
          lineIndex,
          file,
          modification,
          last: false,
        });
      });
    });
    const lastLine = lineRows.findLast(r => r.type === "line");
    if (lastLine && lastLine.type === "line") {
      lastLine.last = true;
    }
    rows.push(...lineRows);
  });
  return rows;
}

export function diffRowKey(row: DiffRow): string {
  switch (row.type) {
    case "file-gap":
      return `g:${row.fileIndex}`;
    case "file-header":
      return `f:${row.fileIndex}`;
    case "hunk-header":
      return `h:${row.fileIndex}:${row.hunkIndex}`;
    case "file-note":
      return `n:${row.fileIndex}`;
    case "line":
      return `l:${row.fileIndex}:${row.hunkIndex}:${row.lineIndex}`;
  }
}
