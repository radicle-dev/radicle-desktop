<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import partial from "lodash/partial";

  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import * as router from "@app/lib/router";
  import { formatOid, publicKeyFromDid, truncateId } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import Changes from "@app/components/Changes.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import Discussion from "@app/components/Discussion.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import VerdictBadge from "@app/components/VerdictBadge.svelte";
  import VerdictButton from "@app/components/VerdictButton.svelte";

  interface Props {
    config: Config;
    onNavigateBack: () => void;
    patchId: string;
    loadReview: () => Promise<void>;
    repo: RepoInfo;
    review: Review | DraftReview;
    revision: Revision;
  }

  const {
    config,
    onNavigateBack,
    patchId,
    loadReview,
    repo,
    review,
    revision,
  }: Props = $props();

  let publishingInProgress = $state(false);
  const canPublish = $derived(review.verdict || review.summary);

  const commentThreads = $derived(
    ((review.comments &&
      review.comments
        .filter(
          comment =>
            (!comment.location &&
              comment.id !== review.id &&
              !comment.replyTo) ||
            comment.replyTo === review.id,
        )
        .map(thread => {
          return {
            root: thread,
            replies:
              review.comments &&
              review.comments
                .filter(comment => comment.replyTo === thread.id)
                .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp),
          };
        }, [])) as Thread[]) || [],
  );

  const codeCommentThreads = $derived(
    ((review.comments &&
      review.comments
        .filter(
          comment =>
            (comment.location &&
              comment.id !== review.id &&
              !comment.replyTo) ||
            comment.replyTo === review.id,
        )
        .map(thread => {
          return {
            root: thread,
            replies:
              review.comments &&
              review.comments
                .filter(comment => comment.replyTo === thread.id)
                .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp),
          };
        }, [])) as Thread<CodeLocation>[]) || [],
  );

  let verdict: Review["verdict"] = $state(review.verdict);

  async function editReview(
    reviewId: string,
    verdict: Review["verdict"],
    labels: string[],
    summary?: string,
  ) {
    if (summary?.trim() === "") {
      summary = undefined;
    } else {
      summary = summary?.trim();
    }

    try {
      if ("draft" in review) {
        draftReviewStorage.update(review.id, {
          verdict,
          summary: summary ?? "",
          labels,
        });
      } else {
        await invoke("edit_patch", {
          rid: repo.rid,
          cobId: patchId,
          action: {
            type: "review.edit",
            review: reviewId,
            summary,
            verdict,
            labels,
          },
          opts: { announce: $nodeRunning && $announce },
        });
      }
    } catch (error) {
      console.error("Editing review failed: ", error);
    } finally {
      await loadReview();
    }
  }

  async function createComment(
    body: string,
    embeds: Embed[],
    replyTo?: string,
    location?: CodeLocation,
  ) {
    try {
      if ("draft" in review) {
        draftReviewStorage.addComment(review.id, {
          body,
          location: location!,
        });
      } else {
        await invoke("edit_patch", {
          rid: repo.rid,
          cobId: patchId,
          action: {
            type: "review.comment",
            review: review.id,
            body,
            embeds,
            replyTo,
            location,
          },
          opts: { announce: $nodeRunning && $announce },
        });
      }
    } catch (error) {
      console.error("Creating comment failed", error);
    } finally {
      await loadReview();
    }
  }

  async function editComment(commentId: string, body: string, embeds: Embed[]) {
    try {
      if ("draft" in review) {
        draftReviewStorage.updateComment(review.id, commentId, {
          body,
        });
      } else {
        await invoke("edit_patch", {
          rid: repo.rid,
          cobId: patchId,
          action: {
            type: "review.comment.edit",
            comment: commentId,
            body,
            review: review.id,
            embeds,
          },
          opts: { announce: $nodeRunning && $announce },
        });
      }
    } catch (error) {
      console.error("Editing comment failed: ", error);
    } finally {
      await loadReview();
    }
  }

  async function reactOnComment(
    commentId: string,
    authors: Author[],
    reaction: string,
  ) {
    if ("draft" in review) {
      throw new Error("Cannot react on comment for draft review");
    }
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patchId,
        action: {
          type: "review.comment.react",
          comment: commentId,
          reaction,
          review: review.id,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === config.publicKey,
          ),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing comment reactions failed", error);
    } finally {
      await loadReview();
    }
  }

  async function changeCommentStatus(commentId: string, resolved: boolean) {
    if ("draft" in review) {
      throw new Error("Cannot change comment status for draft review");
    }
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patchId,
        action: {
          type: resolved
            ? "review.comment.resolve"
            : "review.comment.unresolve",
          comment: commentId,
          review: review.id,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Updating comment status failed: ", error);
    } finally {
      await loadReview();
    }
  }

  async function deleteDraftComment(commentId: string) {
    if (!("draft" in review)) {
      throw new Error("Cannot change comment status for draft review");
    }

    draftReviewStorage.deleteComment(review.id, commentId);
    await loadReview();
  }
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .topbar {
    display: flex;
    align-items: center;
    padding: 0 1rem;
    height: 2.5rem;
    flex-shrink: 0;
    gap: 0.375rem;
    border-bottom: 1px solid var(--color-border-subtle);
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .topbar-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .topbar-link:hover {
    color: var(--color-text-primary);
  }
  .title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
    font: var(--txt-heading-s);
  }
  .main {
    padding: 1.5rem 2rem;
    min-width: 0;
  }
  .review-body {
    margin: 1rem 0;
    background-color: var(--color-surface-canvas);
    border-radius: var(--border-radius-md);
  }
</style>

<div class="page">
  <div class="topbar">
    <Icon name="patch" />
    <button
      class="topbar-link"
      onclick={() =>
        router.push({
          resource: "repo.patches",
          rid: repo.rid,
          status: undefined,
        })}>
      All Patches
    </button>
    <Icon name="chevron-right" />
    <button class="topbar-link" onclick={onNavigateBack}>
      {formatOid(patchId)}
    </button>
    <Icon name="chevron-right" />
    <span>Review</span>
  </div>

  <ScrollArea style="flex: 1; min-height: 0;">
    <div class="main">
      <div class="title">
        {#if "draft" in review}
          <span
            class="global-chip"
            title="This review is not yet visible to your peers">
            Draft
          </span>
        {/if}
        <span>
          {review.author.alias ??
            truncateId(publicKeyFromDid(review.author.did))}'s verdict
        </span>
        {#if !!roles.isDelegateOrAuthor( config.publicKey, repo.delegates.map(delegate => delegate.did), review.author.did, )}
          <VerdictButton
            selectedVerdict={verdict}
            draft={"draft" in review}
            summaryMissing={review.summary === undefined ||
              review.summary.trim() === ""}
            onSelect={async newVerdict => {
              verdict = newVerdict;
              await editReview(
                review.id,
                verdict,
                review.labels,
                review.summary,
              );
            }} />
        {:else}
          <VerdictBadge {verdict} />
        {/if}
        {#if "draft" in review}
          <div class="global-flex" style:margin-left="auto" style:gap="0.5rem">
            <Button
              variant="naked"
              styleHeight="2rem"
              onclick={() => {
                draftReviewStorage.delete(review.id);
                void router.push({
                  resource: "repo.patch",
                  rid: repo.rid,
                  patch: patchId,
                  reviewId: undefined,
                  status: undefined,
                });
              }}>
              <Icon name="trash" />Delete
            </Button>
            <Button
              styleHeight="2rem"
              variant="secondary"
              title={canPublish
                ? undefined
                : "Add a summary or select a verdict to publish the review"}
              disabled={!canPublish || publishingInProgress}
              onclick={async () => {
                publishingInProgress = true;
                try {
                  await draftReviewStorage.publish(review.id);
                  await router.push({
                    resource: "repo.patch",
                    rid: repo.rid,
                    patch: patchId,
                    reviewId: undefined,
                    status: undefined,
                  });
                } finally {
                  publishingInProgress = false;
                }
              }}>
              <Icon name="checkout" />Publish
            </Button>
          </div>
        {/if}
      </div>
      <div class="review-body">
        <CommentComponent
          disableAttachments
          rid={repo.rid}
          currentUserNid={config.publicKey}
          disallowEmptyBody={!("draft" in review) &&
            review.verdict === undefined}
          emptyBodyTooltip="Summary is mandatory when verdict is None"
          styleWidth="100%"
          caption={"draft" in review ? "draft review" : "published review"}
          id={"draft" in review ? undefined : review.id}
          author={review.author}
          timestamp={"draft" in review ? undefined : review.timestamp}
          editComment={(publicKeyFromDid(review.author.did) ===
            config.publicKey ||
            undefined) &&
            partial(async (id: string, summary: string) => {
              await editReview(id, verdict, review.labels, summary);
            }, review.id)}
          body={review.summary}>
          {#snippet beforeTimestamp()}
            on revision <Id id={revision.id} clipboard={revision.id} />
          {/snippet}
        </CommentComponent>
      </div>

      {#if !("draft" in review)}
        <Discussion
          cobId={patchId}
          repoDelegates={repo.delegates}
          rid={repo.rid}
          {commentThreads}
          {config}
          {createComment}
          {editComment}
          {reactOnComment} />
      {/if}

      <Changes
        codeComments={{
          changeCommentStatus:
            "draft" in review ? undefined : changeCommentStatus,
          config,
          createComment,
          editComment,
          reactOnComment: "draft" in review ? undefined : reactOnComment,
          deleteComment: "draft" in review ? deleteDraftComment : undefined,
          repoDelegates: repo.delegates,
          canReply: !("draft" in review),
          disableAttachments:
            "draft" in review ? "Publish your review to attach files" : false,
          rid: repo.rid,
          threads: codeCommentThreads,
        }}
        rid={repo.rid}
        {patchId}
        {revision} />
    </div>
  </ScrollArea>
</div>
