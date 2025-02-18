<script lang="ts">
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Review } from "@bindings/cob/patch/Review";

  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
    verdictIcon,
  } from "@app/lib/utils";
  import { push } from "@app/lib/router";

  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Label from "@app/components/Label.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    patchId: string;
    review: Review;
    rid: string;
    status: PatchStatus | undefined;
  }

  const { patchId, review, rid, status }: Props = $props();
</script>

<style>
  .review {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    z-index: 1;
    position: relative;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .review::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-fill-float);
    clip-path: var(--2px-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
  }
  .review:hover::after {
    background-color: var(--color-fill-float-hover);
  }
  .review-content {
    padding: 10px 0.75rem 0.5rem 0;
    width: 100%;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .timestamp {
    color: var(--color-foreground-dim);
  }
  .review-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }
  .status {
    padding: 0;
    margin: 0.5rem 0 0 0.5rem;
  }

  .accepted {
    background-color: var(--color-fill-diff-green-light);
    color: var(--color-foreground-success);
  }

  .rejected {
    background-color: var(--color-fill-diff-red-light);
    color: var(--color-foreground-red);
  }

  .no-verdict {
    background-color: var(--color-fill-ghost);
    color: var(--color-foreground-dim);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  tabindex="0"
  role="button"
  class="review"
  style:cursor="pointer"
  onclick={() => {
    void push({
      resource: "repo.patch",
      rid,
      patch: patchId,
      status,
      reviewId: review.id,
    });
  }}>
  <div
    class:accepted={review.verdict === "accept"}
    class:rejected={review.verdict === "reject"}
    class:no-verdict={review.verdict === undefined}
    class="global-counter status">
    <Icon name={verdictIcon(review.verdict)} />
  </div>
  <div class="review-content">
    <div class="review-header">
      <div class="global-flex">
        <NodeId {...authorForNodeId(review.author)} />
        <span>published review</span>
        <Id id={review.id} variant="oid" />
        <div class="timestamp" title={absoluteTimestamp(review.timestamp)}>
          {formatTimestamp(review.timestamp)}
        </div>
      </div>
      {#if review.comments.length > 0}
        <div class="global-flex" style:gap="0.25rem" style:margin-left="auto">
          <Icon name="comment" />{review.comments.length}
        </div>
      {/if}
      {#if review.labels.length > 0}
        <div class="global-flex" style:margin-left="auto">
          {#each review.labels as label}
            <Label {label} />
          {/each}
        </div>
      {/if}
    </div>
    {#if review.summary?.trim()}
      <div>
        <Markdown {rid} breaks content={review.summary} />
      </div>
    {/if}
  </div>
</div>
