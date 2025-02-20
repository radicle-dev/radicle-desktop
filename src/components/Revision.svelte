<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Verdict } from "@bindings/cob/patch/Verdict";

  import partial from "lodash/partial";

  import * as roles from "@app/lib/roles";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import { didFromPublicKey, publicKeyFromDid } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Changes from "@app/components/Changes.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import Discussion from "./Discussion.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import ReviewTeaser from "@app/components/ReviewTeaser.svelte";

  interface Props {
    rid: string;
    repoDelegates: Author[];
    patchId: string;
    revision: Revision;
    config: Config;
    status: PatchStatus | undefined;
    reload: () => Promise<void>;
  }

  /* eslint-disable prefer-const */
  let { rid, repoDelegates, patchId, revision, config, status, reload }: Props =
    $props();
  /* eslint-enable prefer-const */

  const hasOwnReview = $derived(
    Boolean(
      revision.reviews &&
        revision.reviews.some(
          value => value.author.did === didFromPublicKey(config.publicKey),
        ),
    ),
  );

  let hideReviews = $state(
    revision.reviews === undefined || revision.reviews.length === 0,
  );

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    hideReviews =
      revision.reviews === undefined || revision.reviews.length === 0;
  });

  const commentThreads = $derived(
    ((revision.discussion &&
      revision.discussion
        .filter(
          comment =>
            (comment.id !== revision.id && !comment.replyTo) ||
            comment.replyTo === revision.id,
        )
        .map(thread => {
          return {
            root: thread,
            replies:
              revision.discussion &&
              revision.discussion
                .filter(comment => comment.replyTo === thread.id)
                .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp),
          };
        }, [])) as Thread[]) || [],
  );

  async function editRevision(
    revisionId: string,
    description: string,
    embeds: Embed[],
  ) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.edit",
          revision: revisionId,
          description,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing revision failed: ", error);
    } finally {
      await reload();
    }
  }

  async function reactOnRevision(
    publicKey: string,
    revisionId: string,
    authors: Author[],
    reaction: string,
  ) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.react",
          revision: revisionId,
          reaction,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing reactions failed", error);
    } finally {
      await reload();
    }
  }

  async function createReview(verdict?: Verdict) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "review",
          revision: revision.id,
          verdict,
          // We need to pass an empty string to create a review without a verdict.
          summary: "",
          labels: [],
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Creating a review failed: ", error);
    } finally {
      await reload();
    }
  }

  async function createComment(
    body: string,
    embeds: Embed[],
    replyTo?: string,
  ) {
    try {
      await invoke("create_patch_comment", {
        rid: rid,
        new: { id: patchId, body, embeds, replyTo, revision: revision.id },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Creating comment failed", error);
    } finally {
      await reload();
    }
  }

  async function editComment(commentId: string, body: string, embeds: Embed[]) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.comment.edit",
          comment: commentId,
          body,
          revision: revision.id,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing comment failed: ", error);
    } finally {
      await reload();
    }
  }

  async function reactOnComment(
    publicKey: string,
    commentId: string,
    authors: Author[],
    reaction: string,
  ) {
    try {
      await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "revision.comment.react",
          comment: commentId,
          reaction,
          revision: revision.id,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing comment reactions failed", error);
    } finally {
      await reload();
    }
  }
</script>

<style>
  .patch-body {
    margin-bottom: 1rem;
    position: relative;
    z-index: 1;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .patch-body::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--2px-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
  }
  .hide {
    display: none;
  }
</style>

<div class="txt-small patch-body">
  <CommentComponent
    caption={revision.id === patchId ? "opened patch" : "created revision"}
    {rid}
    id={revision.id}
    lastEdit={revision.description.length > 1
      ? revision.description.at(-1)
      : undefined}
    author={revision.author}
    reactions={revision.reactions}
    timestamp={revision.timestamp}
    body={revision.description.slice(-1)[0].body}
    reactOnComment={partial(reactOnRevision, config.publicKey, revision.id)}
    editComment={roles.isDelegateOrAuthor(
      config.publicKey,
      repoDelegates.map(delegate => delegate.did),
      revision.author.did,
    ) && partial(editRevision, revision.id)}>
  </CommentComponent>
</div>

<div style:margin={hideReviews ? "1.5rem 0" : "1.5rem 0 2.5rem 0"}>
  <div class="global-flex">
    <NakedButton
      disabled={revision.reviews === undefined || revision.reviews.length === 0}
      variant="ghost"
      onclick={() => (hideReviews = !hideReviews)}>
      <Icon name={hideReviews ? "chevron-right" : "chevron-down"} />
      <div class="txt-semibold global-flex txt-regular">
        Reviews <span style:font-weight="var(--font-weight-regular)">
          {revision.reviews?.length ?? 0}
        </span>
      </div>
    </NakedButton>

    <div class="global-flex" style:margin-left="auto">
      <NakedButton
        variant="secondary"
        disabled={hasOwnReview}
        title={hasOwnReview ? "You already published a review" : undefined}
        onclick={() => createReview()}>
        <Icon name="plus" />
        <span class="txt-small">Write Review</span>
      </NakedButton>
      <Button
        variant="danger"
        disabled={hasOwnReview}
        title={hasOwnReview ? "You already published a review" : undefined}
        onclick={() => createReview("reject")}>
        <Icon name="comment-cross" />
        <span class="txt-small">Reject</span>
      </Button>
      <Button
        variant="success"
        disabled={hasOwnReview}
        title={hasOwnReview ? "You already published a review" : undefined}
        onclick={() => createReview("accept")}>
        <Icon name="comment-checkmark" />
        <span class="txt-small">Accept</span>
      </Button>
    </div>
  </div>

  {#if revision.reviews && revision.reviews.length}
    <div class:hide={hideReviews} style:margin-top="1rem">
      {#each revision.reviews as review}
        <ReviewTeaser {rid} {review} {patchId} {status} />
      {/each}
    </div>
  {/if}
</div>

<Discussion
  cobId={patchId}
  {commentThreads}
  {config}
  {createComment}
  {editComment}
  {reactOnComment}
  {repoDelegates}
  {rid} />

<Changes {rid} {patchId} {revision} />
