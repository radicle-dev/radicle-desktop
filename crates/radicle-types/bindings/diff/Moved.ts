// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DiffContent } from "./DiffContent";
import type { DiffFile } from "./DiffFile";

export type Moved = {
  oldPath: string;
  old: DiffFile;
  newPath: string;
  new: DiffFile;
  diff: DiffContent;
};