import type { FileDiffMetadata } from "@pierre/diffs";

export type FileStatus = "added" | "deleted" | "modified" | "moved" | "copied";
// `unchanged` is a file whose contents are identical on both sides — a pure
// rename or copy, or a mode-only change. It has no lines to render, like
// `empty`, but the file is not empty and saying so is misleading.
export type FileNote = "binary" | "empty" | "unchanged";

// Reactive state for a single `DiffFileHeader`. One instance is created per
// pooled CodeView item element and mutated in place as the element is recycled
// for different files while scrolling (see `PierreDiff.svelte`), so the header
// updates without remounting.
export class DiffFileHeaderState {
  fileDiff = $state.raw<FileDiffMetadata | undefined>(undefined);
  status = $state<FileStatus | undefined>(undefined);
  note = $state<FileNote | undefined>(undefined);
  collapsed = $state(false);
  // `undefined` hides the actions menu.
  text = $state<(() => Promise<string>) | undefined>(undefined);
  // `undefined` hides the reviewed toggle, which only exists while a draft
  // review is open.
  reviewed = $state<boolean | undefined>(undefined);
  // Code-comment threads anchored in this file. Only threads that can be
  // resolved are counted, so a file never shows permanently unresolved work
  // (see `CodeComments.canResolveComment`).
  resolvedComments = $state(0);
  unresolvedComments = $state(0);
  // Not `$state`: read only at click time. Set per file in `syncHeaderSlots`.
  onToggleCollapse: () => void = () => {
    // Replaced per file.
  };
  onToggleReviewed: () => void = () => {
    // Replaced per file.
  };
}
