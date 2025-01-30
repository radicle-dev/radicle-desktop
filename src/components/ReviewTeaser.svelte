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

  const backgroundColor = {
    review: "var(--color-background-float)",
    accept: "var(--color-fill-diff-green-light)",
    reject: "var(--color-fill-diff-red-light)",
  };
</script>

<style>
  .review {
    clip-path: var(--2px-corner-fill);
    padding: 0.5rem 0.75rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
</style>

<div
  class="review"
  style:background-color={backgroundColor[review.verdict ?? "review"]}>
  <div class="global-flex">
    <NodeId {...authorForNodeId(review.author)} />
    {review.verdict ? `${review.verdict}ed` : "reviewed"} revision
    <div title={absoluteTimestamp(review.timestamp)}>
      {formatTimestamp(review.timestamp)}
    </div>
    {#if review.comments.length > 0}
      <div class="global-flex" style:gap="0.25rem" style:margin-left="auto">
        <Icon name="comment" />{review.comments.length}
      </div>
    {/if}
  </div>
  <div>
    {#if review.summary?.trim()}
      <Markdown {rid} breaks content={review.summary} />
    {:else}
      <span class="txt-missing">No summary.</span>
    {/if}
  </div>
</div>
