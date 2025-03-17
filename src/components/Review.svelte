<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import partial from "lodash/partial";
  import uniqBy from "lodash/uniqBy";

  import * as roles from "@app/lib/roles";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { authorForNodeId, publicKeyFromDid } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";

  import Border from "@app/components/Border.svelte";
  import Changes from "@app/components/Changes.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import Discussion from "@app/components/Discussion.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import VerdictBadge from "@app/components/VerdictBadge.svelte";
  import VerdictButton from "@app/components/VerdictButton.svelte";

  interface Props {
    config: Config;
    onNavigateBack: () => void;
    patchId: string;
    loadReview: () => Promise<void>;
    repo: RepoInfo;
    review: Review;
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

  const contributors = $derived(
    uniqBy(
      [
        review.author,
        ...review.comments.map(c => {
          return c.author;
        }),
      ],
      "did",
    ),
  );

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
  let labelSaveInProgress: boolean = $state(false);

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
      labelSaveInProgress = true;
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
    } catch (error) {
      console.error("Editing review failed: ", error);
    } finally {
      labelSaveInProgress = false;
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
    } catch (error) {
      console.error("Creating comment failed", error);
    } finally {
      await loadReview();
    }
  }

  async function editComment(commentId: string, body: string, embeds: Embed[]) {
    try {
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
    } catch (error) {
      console.error("Editing comment failed: ", error);
    } finally {
      await loadReview();
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
        rid: repo.rid,
        cobId: patchId,
        action: {
          type: "review.comment.react",
          comment: commentId,
          reaction,
          review: review.id,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === publicKey,
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
</script>

<style>
  .content {
    padding: 1rem 1rem 1rem 0;
  }
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    display: flex;
    align-items: center;
    white-space: nowrap;
    min-height: 40px;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .metadata-divider {
    width: 2px;
    background-color: var(--color-fill-ghost);
    height: calc(100% + 4px);
    top: 0;
    position: relative;
  }
  .metadata-section {
    padding: 0.5rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    height: 100%;
  }
  .metadata-section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
  .review-body {
    margin-top: 1rem;
    margin-bottom: 1rem;
    position: relative;
    z-index: 1;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .review-body::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--2px-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
  }
</style>

<div class="content">
  <div style:margin-bottom="0.5rem">
    <div class="title">
      <NakedButton
        variant="ghost"
        onclick={onNavigateBack}
        stylePadding="0 4px">
        <Icon name="arrow-left" />
      </NakedButton>
      <span class="global-flex" style:gap="0">
        <NodeId
          {...authorForNodeId(review.author)}
          styleFontSize="var(--font-size-medium)"
          styleFontWeight="var(--font-weight-medium)" />'s review
      </span>
    </div>

    <Border variant="ghost" styleGap="0">
      <div class="metadata-section" style:min-width="8rem">
        <div class="metadata-section-title">Verdict</div>
        {#if !!roles.isDelegateOrAuthor( config.publicKey, repo.delegates.map(delegate => delegate.did), review.author.did, )}
          <VerdictButton
            selectedVerdict={verdict}
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
      </div>

      <div class="metadata-divider"></div>

      <div class="metadata-section" style:flex="1">
        <LabelInput
          allowedToEdit={!!roles.isDelegateOrAuthor(
            config.publicKey,
            repo.delegates.map(delegate => delegate.did),
            review.author.did,
          )}
          labels={review.labels}
          submitInProgress={labelSaveInProgress}
          save={async labels => {
            await editReview(review.id, verdict, labels, review.summary);
          }} />
      </div>

      <div class="metadata-divider"></div>

      <div class="metadata-section" style:flex="1">
        <div class="metadata-section-title">Participants</div>
        <div class="global-flex">
          {#each contributors as contributor}
            <NodeId {...authorForNodeId(contributor)} />
          {/each}
        </div>
      </div>
    </Border>

    <div class="review-body">
      <CommentComponent
        rid={repo.rid}
        disallowEmptyBody={review.verdict === undefined}
        emptyBodyTooltip="Summary is mandatory when verdict is None"
        styleWidth="100%"
        caption="published review"
        id={review.id}
        author={review.author}
        timestamp={review.timestamp}
        editComment={(publicKeyFromDid(review.author.did) ===
          config.publicKey ||
          undefined) &&
          partial(async (id: string, summary: string) => {
            await editReview(id, verdict, review.labels, summary);
          }, review.id)}
        body={review.summary}>
        {#snippet beforeTimestamp()}
          on revision <Id id={revision.id} variant="oid" />
        {/snippet}
      </CommentComponent>
    </div>
  </div>

  <Discussion
    cobId={patchId}
    repoDelegates={repo.delegates}
    rid={repo.rid}
    {commentThreads}
    {config}
    {createComment}
    {editComment}
    {reactOnComment} />

  <Changes
    codeComments={{
      config,
      createComment,
      editComment,
      reactOnComment,
      repoDelegates: repo.delegates,
      rid: repo.rid,
      threads: codeCommentThreads,
    }}
    rid={repo.rid}
    {patchId}
    {revision} />
</div>
