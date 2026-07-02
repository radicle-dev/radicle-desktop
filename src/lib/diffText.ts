import type { FileDiff } from "@bindings/diff/FileDiff";

// The repo-relative path identifying a file diff (the new side for renames
// and copies).
export function fileDiffPath(file: FileDiff): string {
  return file.status === "moved" || file.status === "copied"
    ? file.newPath
    : file.path;
}

// A default filename for saving a single file's diff to disk.
export function fileDiffName(file: FileDiff): string {
  const base = fileDiffPath(file).split("/").pop() || "file";
  return `${base}.diff`;
}
