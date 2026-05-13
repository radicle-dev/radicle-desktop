<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Comment } from "@bindings/cob/thread/Comment";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";

  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import { push } from "@app/lib/router";
  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
    verdictIcon,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import ReviewCodeThread from "@app/components/ReviewCodeThread.svelte";

  interface Props {
    config: Config;
    patch: Patch;
    repoDelegates: Author[];
    review: Review | DraftReview;
    revisions: Revision[];
    rid: string;
    status: Patch["state"]["status"] | undefined;
  }

  const {
    config,
    patch,
    repoDelegates,
    review,
    revisions,
    rid,
    status,
  }: Props = $props();

  type FileGroup = {
    path: string;
    threads: Thread<CodeLocation>[];
  };

  const reviewedRevision: Revision | undefined = $derived.by(() => {
    if ("draft" in review) {
      return revisions.find(r => r.id === review.revisionId);
    }
    return revisions.find(r => r.reviews?.some(rev => rev.id === review.id));
  });

  const fileGroups: FileGroup[] = $derived.by(() => {
    const groups = new Map<string, Thread<CodeLocation>[]>();
    const comments = review.comments as Comment<CodeLocation>[];
    const roots = comments.filter(
      c => c.location && !c.replyTo && c.id !== review.id,
    );
    for (const root of roots) {
      const replies = comments
        .filter(c => c.replyTo === root.id)
        .sort((a, b) => a.edits[0].timestamp - b.edits[0].timestamp);
      const thread = { root, replies } as Thread<CodeLocation>;
      const path = root.location!.path;
      const list = groups.get(path) ?? [];
      list.push(thread);
      groups.set(path, list);
    }
    return [...groups.entries()].map(([path, threads]) => ({ path, threads }));
  });

  const verdict = $derived(review.verdict);
  const timestamp = $derived(
    "draft" in review ? Date.now() : (review as Review).timestamp,
  );
  const summary = $derived(review.summary?.trim() ?? "");

  function backToPatch() {
    void push({
      resource: "repo.patch",
      rid,
      patch: patch.id,
      status,
      reviewId: undefined,
    });
  }
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 0.75rem;
  }
  .back {
    background: none;
    border: 1px solid transparent;
    padding: 0.25rem 0.5rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    color: var(--color-text-secondary);
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font: var(--txt-body-m-regular);
  }
  .back:hover,
  .back:focus-visible {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
  }
  .meta :global(*) {
    font: inherit;
  }
  .verdict-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.125rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
  }
  .verdict-chip.accept {
    color: var(--color-feedback-success-text);
  }
  .verdict-chip.reject {
    color: var(--color-feedback-error-text);
  }
  .draft-chip {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    color: var(--color-text-secondary);
  }
  .summary {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    padding: 0.75rem 1rem;
    margin-bottom: 1.5rem;
  }
  .threads {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .empty {
    color: var(--color-text-secondary);
    padding: 1.5rem 0;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
</style>

<div class="header">
  <button
    type="button"
    class="back"
    onclick={backToPatch}
    title="Back to patch">
    <Icon name="arrow-left" />
    Back to patch
    <Id id={patch.id} clipboard={patch.id} />
  </button>
</div>

<div class="meta">
  <NodeId {...authorForNodeId(review.author)} />
  <span
    class="verdict-chip"
    class:accept={verdict === "accept"}
    class:reject={verdict === "reject"}>
    <Icon name={verdictIcon(verdict)} />
    {verdict === "accept"
      ? "Accepted"
      : verdict === "reject"
        ? "Rejected"
        : "Reviewed"}
  </span>
  {#if reviewedRevision}
    <span>revision</span>
    <Id id={reviewedRevision.id} clipboard={reviewedRevision.id} />
  {/if}
  {#if "draft" in review}
    <span
      class="draft-chip"
      title="This review is not yet visible to your peers">
      Draft
    </span>
  {/if}
  <span class="timestamp" title={absoluteTimestamp(timestamp)}>
    {formatTimestamp(timestamp)}
  </span>
</div>

{#if summary !== ""}
  <div class="summary">
    <Markdown {rid} breaks content={summary} />
  </div>
{/if}

{#if fileGroups.length === 0}
  <div class="empty">No inline comments in this review.</div>
{:else if reviewedRevision}
  <div class="threads">
    {#each fileGroups as group (group.path)}
      {#each group.threads as thread (thread.root.id)}
        <ReviewCodeThread
          base={reviewedRevision.base}
          head={reviewedRevision.head}
          {thread}
          {config}
          {repoDelegates}
          {rid} />
      {/each}
    {/each}
  </div>
{/if}
