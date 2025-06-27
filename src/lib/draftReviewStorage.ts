import { z } from "zod";

import type { Comment } from "@bindings/cob/thread/Comment";
import type { Author } from "@bindings/cob/Author";
import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
import type { CodeRange } from "@bindings/cob/thread/CodeRange";
import type { CreateReviewArgs } from "@bindings/cob/patch/CreateReviewArgs";
import type { Embed } from "@bindings/cob/thread/Embed";
import type { Patch } from "@bindings/cob/patch/Patch";
import type { Verdict } from "@bindings/cob/patch/Verdict";

import useLocalStorage from "@app/lib/useLocalStorage.svelte";
import { invoke } from "./invoke";

// This is different from the stored draft review to align it with a
// published `Review`.
export interface DraftReview {
  id: string;
  draft: true;
  rid: string;
  author: Author;
  revisionId: string;
  verdict?: Verdict;
  summary?: string;
  labels: string[];
  comments: Array<Comment<CodeLocation>>;
}

const codeRangeSchema: z.Schema<CodeRange> = z.union([
  z.object({
    type: z.literal("lines"),
    range: z.object({ start: z.number(), end: z.number() }),
  }),
  z.object({
    type: z.literal("chars"),
    line: z.number(),
    range: z.object({ start: z.number(), end: z.number() }),
  }),
]);

const draftReviewStoredSchema = z.object({
  id: z.string(),
  rid: z.string(),
  revision: z.string(),
  verdict: z.union([z.literal("accept"), z.literal("reject")]).optional(),
  summary: z.string().default(""),
  labels: z.array(z.string()),
  comments: z.array(
    z.object({
      id: z.string(),
      body: z.string(),
      location: z
        .object({
          commit: z.string(),
          path: z.string(),
          old: codeRangeSchema.nullable(),
          new: codeRangeSchema.nullable(),
        })
        .optional(),
    }),
  ),
});

const storage = useLocalStorage(
  "repo.patches.draftReviews",
  z.record(z.string(), draftReviewStoredSchema),
  {},
);

export const draftReviewStorage = {
  get(id: string, author: Author): DraftReview | undefined {
    const draftReviewStored = storage.value[id];
    if (!draftReviewStored) {
      return undefined;
    }

    return draftReviewFromStored(draftReviewStored, author);
  },

  getForRevision(revisionId: string, author: Author): DraftReview | undefined {
    const draftReviewStored = Object.values(storage.value).find(
      draftReview => draftReview.revision === revisionId,
    );

    if (draftReviewStored) {
      return draftReviewFromStored(draftReviewStored, author);
    }
  },

  create(rid: string, revisionId: string): string {
    const draftReviews = storage.value;

    const id = crypto.randomUUID();
    draftReviews[id] = {
      id,
      rid,
      revision: revisionId,
      summary: "",
      labels: [],
      comments: [],
    };

    storage.value = draftReviews;
    return id;
  },

  update(
    id: string,
    props: { summary: string; verdict: Verdict | undefined; labels: string[] },
  ) {
    const draftPatches = storage.value;
    draftPatches[id].summary = props.summary;
    draftPatches[id].verdict = props.verdict;
    draftPatches[id].labels = props.labels;
    storage.value = draftPatches;
  },

  addComment(
    id: string,
    comment: {
      body: string;
      embeds: Embed[];
      location: CodeLocation;
    },
  ): string {
    const draftPatches = storage.value;
    const commentId = crypto.randomUUID();
    draftPatches[id].comments.push({
      id: crypto.randomUUID(),
      body: comment.body,
      location: comment.location,
    });
    storage.value = draftPatches;
    return commentId;
  },

  updateComment(
    id: string,
    commentId: string,
    comment: {
      body: string;
      embeds: Embed[];
    },
  ) {
    const draftPatches = storage.value;
    const storedComment = draftPatches[id].comments.find(
      comment => comment.id === commentId,
    );
    storedComment!.body = comment.body;
    storage.value = draftPatches;
  },

  async publish(id: string) {
    const draftReviewStored = storage.value[id];
    delete storage.value[id];
    // We need to explicitly persist the storage
    // eslint-disable-next-line no-self-assign
    storage.value = storage.value;
    await invoke<Patch>("create_patch_review", {
      args: {
        rid: draftReviewStored.rid,
        revision: draftReviewStored.revision,
        verdict: draftReviewStored.verdict ?? null,
        summary: draftReviewStored.summary,
        labels: draftReviewStored.labels,
        comments: draftReviewStored.comments.map(storedComment => ({
          body: storedComment.body,
          location: storedComment.location ?? null,
        })),
      } satisfies CreateReviewArgs,
    });
  },
};

function draftReviewFromStored(
  draftReviewStored: z.infer<typeof draftReviewStoredSchema>,
  author: Author,
): DraftReview {
  return {
    id: draftReviewStored.id,
    draft: true,
    rid: draftReviewStored.rid,
    summary: draftReviewStored.summary,
    author,
    revisionId: draftReviewStored.revision,
    verdict: draftReviewStored.verdict,
    labels: draftReviewStored.labels,
    comments: draftReviewStored.comments.map(rawComment => ({
      id: rawComment.id,
      author,
      edits: [
        {
          author,
          timestamp: 0,
          body: rawComment.body,
        },
      ],
      reactions: [],
      replyTo: null,
      location: rawComment.location ?? null,
      resolved: false,
    })),
  };
}
