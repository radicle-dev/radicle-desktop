import type { Author } from "@bindings/cob/Author";
import type { CreateReviewArgs } from "@bindings/cob/patch/CreateReviewArgs";
import type { Patch } from "@bindings/cob/patch/Patch";
import type { Verdict } from "@bindings/cob/patch/Verdict";
import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
import type { CodeRange } from "@bindings/cob/thread/CodeRange";
import type { Comment } from "@bindings/cob/thread/Comment";

import { z } from "zod";

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

type DraftReviewStored = z.infer<typeof draftReviewStoredSchema>;

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
    const id = crypto.randomUUID();
    storage.update(reviews => {
      reviews[id] = {
        id,
        rid,
        revision: revisionId,
        summary: "",
        labels: [],
        comments: [],
      };
      return reviews;
    });
    return id;
  },

  hasForRevision(revisionId: string): boolean {
    return Object.values(storage.value).some(
      review => review.revision === revisionId,
    );
  },

  update(
    id: string,
    props: { summary: string; verdict: Verdict | undefined; labels: string[] },
  ) {
    updateStoredDraftReview(id, review => {
      return Object.assign(review, props);
    });
  },

  delete(id: string): DraftReviewStored | undefined {
    const review = storage.value[id];
    storage.update(reviews => {
      delete reviews[id];
      return reviews;
    });
    return review;
  },

  deleteComment(id: string, commentId: string) {
    updateStoredDraftReview(id, review => {
      const index = review.comments.findIndex(
        comment => comment.id === commentId,
      );
      review.comments.splice(index, 1);
      return review;
    });
  },

  addComment(
    id: string,
    comment: {
      body: string;
      location: CodeLocation;
    },
  ): string {
    const commentId = crypto.randomUUID();
    updateStoredDraftReview(id, review => {
      review.comments.push({
        id: commentId,
        body: comment.body,
        location: comment.location,
      });
      return review;
    });
    return commentId;
  },

  updateComment(
    id: string,
    commentId: string,
    comment: {
      body: string;
    },
  ) {
    updateStoredDraftReview(id, review => {
      const storedComment = review.comments.find(
        comment => comment.id === commentId,
      );

      if (!storedComment) {
        throw new Error(
          `Comment ${commentId} does not exist for draft review ${id}`,
        );
      }

      storedComment!.body = comment.body;
      return review;
    });
  },

  async publish(id: string) {
    const draftReviewStored = draftReviewStorage.delete(id);
    if (!draftReviewStored) {
      throw new Error(
        `Failed to publish draft review: Review ${id} does not exist`,
      );
    }

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

function updateStoredDraftReview(
  id: string,
  update: (draftReviewStored: DraftReviewStored) => DraftReviewStored,
): void {
  storage.update(reviews => {
    const review = reviews[id];
    if (!review) {
      throw new Error(`Draft review ${id} does not exist`);
    }

    reviews[id] = update(review);
    return reviews;
  });
}

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
