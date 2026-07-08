import type { FileDiffMetadata } from "@pierre/diffs";

export type FileStatus = "added" | "deleted" | "modified" | "moved" | "copied";
export type FileNote = "binary" | "empty";

// Reactive state for a single `DiffFileHeader`. One instance is created per
// pooled CodeView item element and mutated in place as the element is recycled
// for different files while scrolling (see `PierreDiff.svelte`), so the header
// updates without remounting.
export class DiffFileHeaderState {
  fileDiff = $state.raw<FileDiffMetadata | undefined>(undefined);
  status = $state<FileStatus | undefined>(undefined);
  note = $state<FileNote | undefined>(undefined);
  collapsed = $state(false);
  // Lazily produces this file's raw diff text for the header's copy/save
  // actions; `undefined` hides the actions menu.
  text = $state<(() => Promise<string>) | undefined>(undefined);
  // Read at click time, so it does not need to be reactive. Overwritten per
  // file in `PierreDiff`'s `syncHeaderSlots`; the default is a no-op.
  onToggleCollapse: () => void = () => {
    // no-op until wired up per file
  };
}
