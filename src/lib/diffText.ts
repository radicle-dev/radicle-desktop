import type { Diff } from "@bindings/diff/Diff";
import type { FileDiff } from "@bindings/diff/FileDiff";

// Selection across off-screen lines isn't possible once the diff is
// virtualized, so these build copyable unified-diff text straight from the
// model for the "copy file" / "copy diff" buttons.

function filePath(file: FileDiff): string {
  if (file.status === "moved" || file.status === "copied") {
    return `${file.oldPath} → ${file.newPath}`;
  }
  return file.path;
}

// Diff line/header text from the backend keeps its trailing newline; strip it
// so join("\n") doesn't double every line.
function stripEol(s: string): string {
  return s.replace(/\r?\n$/, "");
}

export function fileDiffToText(file: FileDiff): string {
  const lines: string[] = [filePath(file)];
  if (file.diff.type === "plain") {
    for (const hunk of file.diff.hunks) {
      lines.push(stripEol(hunk.header));
      for (const m of hunk.lines) {
        const prefix =
          m.type === "addition" ? "+" : m.type === "deletion" ? "-" : " ";
        lines.push(`${prefix}${stripEol(m.line)}`);
      }
    }
  } else if (file.diff.type === "binary") {
    lines.push("Binary file");
  }
  return lines.join("\n");
}

export function diffToText(diff: Diff): string {
  return diff.files.map(fileDiffToText).join("\n\n");
}
