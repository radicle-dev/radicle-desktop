import type { Diff } from "@bindings/diff/Diff";
import type { EofNewLine } from "@bindings/diff/EofNewLine";
import type { FileDiff } from "@bindings/diff/FileDiff";
import type { FileMode } from "@bindings/diff/FileMode";
import type { Hunk } from "@bindings/diff/Hunk";

// Selection across off-screen lines isn't possible once the diff is
// virtualized, so these build copyable patch text straight from the model for
// the "copy" / "save to disk" actions. The output is `git diff` format that
// `git apply` accepts (binary files excepted — their bytes aren't available).

const NULL_OID = "0000000000000000000000000000000000000000";
const NO_NEWLINE = "\\ No newline at end of file";

function gitMode(mode: FileMode): string {
  switch (mode) {
    case "blob":
      return "100644";
    case "blobExecutable":
      return "100755";
    case "link":
      return "120000";
    case "tree":
      return "040000";
    case "commit":
      return "160000";
  }
}

// Diff line/header text from the backend keeps its trailing newline; strip it
// so join("\n") doesn't double every line.
function stripEol(s: string): string {
  return s.replace(/\r?\n$/, "");
}

// The hunk headers and content lines, with "\ No newline at end of file"
// markers placed after the last line of whichever side lacks a trailing
// newline (mirroring what git itself emits).
function hunkBody(hunks: Hunk[], eof: EofNewLine): string[] {
  const out: string[] = [];
  let lastOld = -1;
  let lastNew = -1;
  for (const hunk of hunks) {
    out.push(stripEol(hunk.header));
    for (const m of hunk.lines) {
      if (m.type === "addition") {
        out.push(`+${stripEol(m.line)}`);
        lastNew = out.length - 1;
      } else if (m.type === "deletion") {
        out.push(`-${stripEol(m.line)}`);
        lastOld = out.length - 1;
      } else {
        out.push(` ${stripEol(m.line)}`);
        lastOld = out.length - 1;
        lastNew = out.length - 1;
      }
    }
  }

  const marks = new Set<number>();
  if ((eof === "oldMissing" || eof === "bothMissing") && lastOld >= 0) {
    marks.add(lastOld);
  }
  if ((eof === "newMissing" || eof === "bothMissing") && lastNew >= 0) {
    marks.add(lastNew);
  }
  // Insert from the back so earlier indices stay valid.
  for (const i of [...marks].sort((a, b) => b - a)) {
    out.splice(i + 1, 0, NO_NEWLINE);
  }
  return out;
}

export function fileDiffToText(file: FileDiff): string {
  const lines: string[] = [];

  if (file.status === "added") {
    lines.push(`diff --git a/${file.path} b/${file.path}`);
    lines.push(`new file mode ${gitMode(file.new.mode)}`);
    lines.push(`index ${NULL_OID}..${file.new.oid}`);
    lines.push("--- /dev/null");
    lines.push(`+++ b/${file.path}`);
  } else if (file.status === "deleted") {
    lines.push(`diff --git a/${file.path} b/${file.path}`);
    lines.push(`deleted file mode ${gitMode(file.old.mode)}`);
    lines.push(`index ${file.old.oid}..${NULL_OID}`);
    lines.push(`--- a/${file.path}`);
    lines.push("+++ /dev/null");
  } else if (file.status === "modified") {
    lines.push(`diff --git a/${file.path} b/${file.path}`);
    if (file.old.mode !== file.new.mode) {
      lines.push(`old mode ${gitMode(file.old.mode)}`);
      lines.push(`new mode ${gitMode(file.new.mode)}`);
      lines.push(`index ${file.old.oid}..${file.new.oid}`);
    } else {
      lines.push(
        `index ${file.old.oid}..${file.new.oid} ${gitMode(file.new.mode)}`,
      );
    }
    lines.push(`--- a/${file.path}`);
    lines.push(`+++ b/${file.path}`);
  } else {
    // moved or copied
    const verb = file.status === "moved" ? "rename" : "copy";
    const hasHunks = file.diff.type === "plain" && file.diff.hunks.length > 0;
    lines.push(`diff --git a/${file.oldPath} b/${file.newPath}`);
    if (!hasHunks) {
      lines.push("similarity index 100%");
    }
    lines.push(`${verb} from ${file.oldPath}`);
    lines.push(`${verb} to ${file.newPath}`);
    if (hasHunks) {
      lines.push(
        `index ${file.old.oid}..${file.new.oid} ${gitMode(file.new.mode)}`,
      );
      lines.push(`--- a/${file.oldPath}`);
      lines.push(`+++ b/${file.newPath}`);
    }
  }

  if (file.diff.type === "plain") {
    lines.push(...hunkBody(file.diff.hunks, file.diff.eof));
  } else if (file.diff.type === "binary") {
    lines.push("Binary files differ");
  }

  return lines.join("\n");
}

export function diffToText(diff: Diff): string {
  // git output concatenates file sections with no separator; end with a
  // trailing newline so `git apply` reads the final hunk cleanly.
  return diff.files.map(fileDiffToText).join("\n") + "\n";
}

// A default filename for saving a single file's diff to disk.
export function fileDiffName(file: FileDiff): string {
  const path =
    file.status === "moved" || file.status === "copied"
      ? file.newPath
      : file.path;
  const base = path.split("/").pop() || "file";
  return `${base}.diff`;
}
