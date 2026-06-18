import type { Author } from "@bindings/cob/Author";
import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
import type { Embed } from "@bindings/cob/thread/Embed";
import type { Thread } from "@bindings/cob/thread/Thread";
import type { Config } from "@bindings/config/Config";
import type { FileDiff } from "@bindings/diff/FileDiff";
import type { Modification } from "@bindings/diff/Modification";

export interface CodeComments {
  changeCommentStatus?: (commentId: string, resolved: boolean) => Promise<void>;
  config: Config;
  createComment: (
    body: string,
    embeds: Embed[],
    replyTo?: string,
    location?: CodeLocation,
  ) => Promise<void>;
  editComment: (
    commentId: string,
    body: string,
    embeds: Embed[],
  ) => Promise<void>;
  reactOnComment?: (
    commentId: string,
    authors: Author[],
    reaction: string,
  ) => Promise<void>;
  deleteComment?: (commentId: string) => Promise<void>;
  // Defaults to `true`.
  canReply?: boolean;
  // See `ExtendedTextarea`.
  disableAttachments?: boolean | string;
  repoDelegates: Author[];
  rid: string;
  threads: Thread<CodeLocation>[];
}

// Everything the interactive (patch-review) diff path needs, bundled so it can
// be threaded through the single changeset-level virtualizer as one prop.
export interface CommentContext {
  codeComments: CodeComments;
  threadByLine: Map<string, Thread<CodeLocation>>;
  // Per-file resolved/unresolved thread tallies for the file-header badges.
  threadCountsByFile: Map<string, { resolved: number; unresolved: number }>;
  selection: Selection | undefined;
  expandedStates: Record<string, boolean>;
  onSelectLine: (
    fileIdx: number,
    side: Side,
    line: Modification,
    hunkIdx: number,
    lineIdx: number,
    file: FileDiff,
  ) => void;
  onClearSelection: () => void;
  onToggleThread: (threadId: string) => void;
}

export type Side = "left" | "right";
export type SelectionAnchor = { side: Side; lineNumber: number };
export type SelectionRange = { start: SelectionAnchor; end?: SelectionAnchor };

export interface Selection {
  file: string;
  // Index of the file in the changeset; disambiguates lines across files, since
  // hunkIdx/lineIdx are only unique within a single file.
  fileIdx: number;
  start: SelectionAnchor;
  end: SelectionAnchor;
  lineIdx: number;
  hunkIdx: number;
  codeLocation: CodeLocation;
}

export function lineNumber(line: Modification, side: Side): number | undefined {
  if (side === "left") {
    if (line.type === "context") {
      return line.lineNoOld;
    }
    if (line.type === "deletion") {
      return line.lineNo;
    }
  } else {
    if (line.type === "context") {
      return line.lineNoNew;
    }
    if (line.type === "addition") {
      return line.lineNo;
    }
  }
}

export function determineSelectedAnchor(
  side: Side,
  line: Modification,
): SelectionAnchor {
  // When a user tries to select a side with no changes, use opposite side.
  if (side === "left" && line.type === "addition") {
    return { side: "right", lineNumber: line.lineNo };
  } else if (side === "right" && line.type === "deletion") {
    return { side: "left", lineNumber: line.lineNo };
  } else {
    return side === "left"
      ? { side: "left", lineNumber: lineNumber(line, "left") as number }
      : { side: "right", lineNumber: lineNumber(line, "right") as number };
  }
}

export function filePath(file: FileDiff, side: Side): string {
  if (file.status === "moved" || file.status === "copied") {
    if (side === "left") {
      return file.oldPath;
    } else {
      return file.newPath;
    }
  } else {
    return file.path;
  }
}

export function buildSelection(
  head: string,
  file: FileDiff,
  fileIdx: number,
  side: Side,
  line: Modification,
  hunkIdx: number,
  lineIdx: number,
): Selection {
  const commentAnchor = determineSelectedAnchor(side, line);

  return {
    file: filePath(file, side),
    fileIdx,
    start: commentAnchor,
    end: commentAnchor,
    hunkIdx,
    lineIdx,
    codeLocation: {
      commit: head,
      path: filePath(file, side),
      old:
        commentAnchor.side === "left"
          ? {
              type: "lines",
              range: {
                start: commentAnchor.lineNumber,
                end: commentAnchor.lineNumber + 1,
              },
            }
          : null,
      new:
        commentAnchor.side === "right"
          ? {
              type: "lines",
              range: {
                start: commentAnchor.lineNumber,
                end: commentAnchor.lineNumber + 1,
              },
            }
          : null,
    },
  };
}

export function rangeAnchorsFromCodeLocation(
  location: CodeLocation | null,
): SelectionRange | undefined {
  if (location?.old?.type === "lines") {
    return {
      start: { side: "left", lineNumber: location.old.range.start },
    };
  } else if (location?.new?.type === "lines") {
    return {
      start: { side: "right", lineNumber: location.new.range.start },
    };
  }
}

// Index threads by the file and line they anchor to, so each rendered line
// does an O(1) lookup instead of scanning every thread. Rebuilt wholesale by
// callers and only read, so a plain Map is correct here. Keys are qualified
// by the thread's file path — line numbers alone repeat across files.
export function buildThreadByLine(
  threads: Thread<CodeLocation>[],
): Map<string, Thread<CodeLocation>> {
  const map = new Map<string, Thread<CodeLocation>>();
  for (const t of threads) {
    const path = t.root.location?.path;
    if (path === undefined) {
      continue;
    }
    const newEnd = t.root.location?.new?.range.end;
    const oldEnd = t.root.location?.old?.range.end;
    if (newEnd !== undefined && !map.has(`${path}:new:${newEnd}`)) {
      map.set(`${path}:new:${newEnd}`, t);
    }
    if (oldEnd !== undefined && !map.has(`${path}:old:${oldEnd}`)) {
      map.set(`${path}:old:${oldEnd}`, t);
    }
  }
  return map;
}

// Per-file resolved/unresolved thread tallies for the file-header badges.
// Like buildThreadByLine, rebuilt wholesale and only read.
export function buildThreadCountsByFile(
  threads: Thread<CodeLocation>[],
): Map<string, { resolved: number; unresolved: number }> {
  const counts = new Map<string, { resolved: number; unresolved: number }>();
  for (const t of threads) {
    const path = t.root.location?.path;
    if (path === undefined) {
      continue;
    }
    const c = counts.get(path) ?? { resolved: 0, unresolved: 0 };
    if (t.root.resolved) {
      c.resolved += 1;
    } else {
      c.unresolved += 1;
    }
    counts.set(path, c);
  }
  return counts;
}

export function findLineThread(
  map: Map<string, Thread<CodeLocation>>,
  file: FileDiff,
  line: Modification,
): Thread<CodeLocation> | undefined {
  // Comment locations carry a single path, which is ambiguous for renames
  // and copies; like the pre-virtualization per-file diff, those files
  // don't get inline threads.
  if (file.status === "moved" || file.status === "copied") {
    return undefined;
  }
  if (line.type === "addition") {
    return map.get(`${file.path}:new:${line.lineNo + 1}`);
  } else if (line.type === "deletion") {
    return map.get(`${file.path}:old:${line.lineNo + 1}`);
  } else if (line.type === "context") {
    return (
      map.get(`${file.path}:new:${line.lineNoNew + 1}`) ??
      map.get(`${file.path}:old:${line.lineNoOld + 1}`)
    );
  }
}
