// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Author } from "../Author";
import type { CodeLocation } from "../thread/CodeLocation";
import type { Comment } from "../thread/Comment";
import type { Edit } from "./Edit";
import type { Reaction } from "../Reaction";
import type { Review } from "./Review";

export type Revision = {
  id: string;
  author: Author;
  description: Array<Edit>;
  base: string;
  head: string;
  reviews?: Array<Review>;
  timestamp: number;
  discussion?: Array<Comment<CodeLocation>>;
  reactions?: Array<Reaction>;
};
