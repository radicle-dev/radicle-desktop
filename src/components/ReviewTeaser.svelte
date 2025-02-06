<script lang="ts">
  import type { Review } from "@bindings/cob/patch/Review";

  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
  } from "@app/lib/utils";

  import Icon from "./Icon.svelte";
  import Markdown from "./Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    rid: string;
    review: Review;
  }

  const { rid, review }: Props = $props();

  const backgroundColor = $derived.by(() => {
    if (review.verdict === "accept") {
      return "var(--color-fill-diff-green-light)";
    } else if (review.verdict === "reject") {
      return "var(--color-fill-diff-red-light)";
    } else {
      return "var(--color-fill-float-hover)";
    }
  });

  const header = $derived.by(() => {
    if (!review.verdict) {
      return "published a review";
    }

    return `${review.verdict ? `${review.verdict}ed` : "reviewed"} revision with a review`;
  });

  const icon = $derived.by(() => {
    if (review.verdict === "accept") {
      return "comment-checkmark";
    } else if (review.verdict === "reject") {
      return "comment-cross";
    } else {
      return "comment";
    }
  });
</script>

<style>
  .review {
    clip-path: var(--2px-corner-fill);
    display: flex;
    align-items: flex-start;
    padding: 0.5rem 0.75rem;
    gap: 1rem;
  }
  .review-content {
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
  .icon {
    padding-top: 0.25rem;
  }
</style>

<div class="review" style:background-color={backgroundColor}>
  <div class="icon">
    <Icon name={icon} />
  </div>
  <div class="review-content">
    <div class="review-header">
      <div class="global-flex">
        <NodeId {...authorForNodeId(review.author)} />
        <span>{header}</span>
        <div class="timestamp" title={absoluteTimestamp(review.timestamp)}>
          {formatTimestamp(review.timestamp)}
        </div>
        {#if review.comments.length > 0}
          <div class="global-flex" style:gap="0.25rem" style:margin-left="auto">
            <Icon name="comment" />{review.comments.length}
          </div>
        {/if}
      </div>
    </div>
    <div>
      {#if review.summary?.trim()}
        <Markdown {rid} breaks content={review.summary} />
      {:else}
        <span class="txt-missing">No summary.</span>
      {/if}
    </div>
  </div>
</div>
