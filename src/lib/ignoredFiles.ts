import type { FileDiff } from "@bindings/diff/FileDiff";

// Files that add noise to patch diffs (lockfiles, VCS metadata, generated
// manifests). These are hidden from the revision file preview and
// auto-collapsed in the Changes tab. Add new entries here to apply both
// behaviours.
export const IGNORED_FILENAMES: ReadonlySet<string> = new Set([
  ".gitignore",
  ".gitattributes",
  ".gitmodules",
  "package-lock.json",
  "npm-shrinkwrap.json",
  "yarn.lock",
  "pnpm-lock.yaml",
  "bun.lockb",
  "Cargo.lock",
  "Gemfile.lock",
  "poetry.lock",
  "composer.lock",
  "go.sum",
  "flake.lock",
]);

export function diffFilePath(file: FileDiff): string {
  return file.status === "moved" || file.status === "copied"
    ? file.newPath
    : file.path;
}

export function isIgnoredFile(file: FileDiff): boolean {
  const path = diffFilePath(file);
  const filename = path.slice(path.lastIndexOf("/") + 1);
  return IGNORED_FILENAMES.has(filename);
}
