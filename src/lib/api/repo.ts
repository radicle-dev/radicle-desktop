import {
  array,
  literal,
  number,
  object,
  optional,
  string,
  union,
  z,
} from "zod";

import { authorSchema } from "./author";

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const repoSchema = object({
  rid: string(),
  payloads: object({
    "xyz.radicle.project": object({
      data: object({
        name: string(),
        description: string(),
        defaultBranch: string(),
      }),
      meta: object({
        head: string(),
        patches: object({
          open: number(),
          draft: number(),
          archived: number(),
          merged: number(),
        }),
        issues: object({
          open: number(),
          closed: number(),
        }),
      }),
    }),
  }),
  delegates: array(authorSchema),
  threshold: number(),
  visibility: union([
    object({ type: literal("public") }),
    object({ type: literal("private"), allow: optional(array(string())) }),
  ]),
  seeding: number(),
});

export type Repo = z.infer<typeof repoSchema>;
