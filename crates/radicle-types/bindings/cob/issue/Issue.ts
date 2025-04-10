// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Author } from "../Author";
import type { Comment } from "../thread/Comment";
import type { Never } from "../Never";
import type { State } from "./State";

export type Issue = {
  id: string;
  author: Author;
  title: string;
  state: State;
  assignees: Array<Author>;
  body: Comment<Never>;
  commentCount: number;
  labels: Array<string>;
  timestamp: number;
};
