<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Verdict } from "@bindings/cob/patch/Verdict";
  import type { CodeLocation } from "@bindings/cob/thread/CodeLocation";
  import type { Comment } from "@bindings/cob/thread/Comment";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";

  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import { push } from "@app/lib/router";
  import {
    absoluteTimestamp,
    authorForNodeId,
    didFromPublicKey,
    formatTimestamp,
    verdictIcon,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
  import ReviewCodeThread from "@app/components/ReviewCodeThread.svelte";

  interface Props {
    config: Config;
    loadPatch: () => Promise<void>;
    patch: Patch;
    repoDelegates: Author[];
    review: Review | DraftReview;
    revisions: Revision[];
    rid: string;
    status: Patch["state"]["status"] | undefined;
  }

  const {
    config,
    loadPatch,
    patch,
    repoDelegates,
    review,
    revisions,
    rid,
    status,
  }: Props = $props();

  const isOwnPublishedReview = $derived(
    !("draft" in review) &&
      review.author.did === didFromPublicKey(config.publicKey),
  );

  let verdictPickerExpanded = $state(false);
  let savingVerdict = $state(false);
  let editingSummary = $state(false);
  let savingSummary = $state(false);
  let deleteConfirmExpanded = $state(false);
  let deleting = $state(false);

  const verdictOptions: { value: Verdict | undefined; label: string }[] = [
    { value: undefined, label: "Comment" },
    { value: "accept", label: "Accept" },
    { value: "reject", label: "Reject" },
  ];

  async function editCodeComment(
    commentId: string,
    body: string,
    embeds: Embed[],
  ) {
    if ("draft" in review) return;
    try {
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: {
          type: "review.comment.edit",
          review: review.id,
          comment: commentId,
          body,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing review comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function deleteCodeComment(commentId: string) {
    if ("draft" in review) return;
    try {
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: {
          type: "review.comment.redact",
          review: review.id,
          comment: commentId,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Deleting review comment failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function saveSummary(newSummary: string) {
    if ("draft" in review) return;
    try {
      savingSummary = true;
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: {
          type: "review.edit",
          review: review.id,
          summary: newSummary,
          verdict: review.verdict,
          labels: review.labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing review summary failed", error);
    } finally {
      savingSummary = false;
      editingSummary = false;
      await loadPatch();
    }
  }

  async function deleteReview() {
    if ("draft" in review) return;
    try {
      deleting = true;
      closeFocused();
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: { type: "review.redact", review: review.id },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Deleting review failed", error);
      deleting = false;
      return;
    }
    deleting = false;
    backToPatch();
    await loadPatch();
  }

  async function setVerdict(verdict: Verdict | undefined) {
    if (verdict === review.verdict) {
      closeFocused();
      return;
    }
    try {
      savingVerdict = true;
      closeFocused();
      await invoke("edit_patch", {
        rid,
        cobId: patch.id,
        action: {
          type: "review.edit",
          review: review.id,
          summary: review.summary ?? "",
          verdict,
          labels: review.labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing review verdict failed", error);
    } finally {
      savingVerdict = false;
      await loadPatch();
    }
  }

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
    background-color: var(--color-feedback-success-bg);
    border-color: transparent;
    color: var(--color-feedback-success-text);
  }
  .verdict-chip.reject {
    background-color: var(--color-feedback-error-bg);
    border-color: transparent;
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
    position: relative;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    padding: 0.75rem 1rem;
    margin-bottom: 1.5rem;
  }
  .summary-empty {
    color: var(--color-text-tertiary);
  }
  .summary-edit {
    position: absolute;
    top: 0.375rem;
    right: 0.375rem;
    opacity: 0;
    transition: opacity 150ms ease;
  }
  .summary:hover .summary-edit,
  .summary:focus-within .summary-edit,
  .summary-edit:focus-visible {
    opacity: 1;
  }
  .summary-editor {
    margin-bottom: 1.5rem;
  }
  .action-icon {
    background: none;
    border: 1px solid transparent;
    cursor: pointer;
    color: var(--color-text-tertiary);
    padding: 0.25rem;
    border-radius: var(--border-radius-sm);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .action-icon:hover:not(:disabled),
  .action-icon:focus-visible {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .action-icon:disabled {
    cursor: progress;
    opacity: 0.6;
  }
  .delete-confirm {
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background-color: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    max-width: 20rem;
  }
  .delete-confirm-message {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
  }
  .delete-confirm-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
  .delete-confirm-button {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    background-color: var(--color-feedback-error-bg);
    color: var(--color-feedback-error-text);
    border: 1px solid var(--color-feedback-error-text);
    border-radius: var(--border-radius-sm);
    padding: 0.25rem 0.75rem;
    cursor: pointer;
    font: var(--txt-body-m-medium);
  }
  .delete-confirm-button:hover:not(:disabled),
  .delete-confirm-button:focus-visible {
    background-color: var(--color-feedback-error-text);
    color: var(--color-surface-canvas);
  }
  .delete-confirm-button:disabled {
    cursor: progress;
    opacity: 0.6;
  }
  .threads {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
  .verdict-toggle {
    cursor: pointer;
    font: inherit;
  }
  .verdict-toggle:not(.accept):not(.reject):hover,
  .verdict-toggle:not(.accept):not(.reject):focus-visible {
    background-color: var(--color-surface-subtle);
  }
  .verdict-toggle.accept:hover,
  .verdict-toggle.accept:focus-visible,
  .verdict-toggle.reject:hover,
  .verdict-toggle.reject:focus-visible {
    filter: brightness(0.97);
  }
  .verdict-toggle:disabled {
    cursor: progress;
    opacity: 0.6;
  }
  .verdict-accept {
    color: var(--color-feedback-success-text);
  }
  .verdict-reject {
    color: var(--color-feedback-error-text);
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
  {#if isOwnPublishedReview}
    {@const verdictLabel =
      verdict === "accept"
        ? "Accepted"
        : verdict === "reject"
          ? "Rejected"
          : "Reviewed"}
    <Popover
      popoverPadding="0"
      placement="bottom-start"
      bind:expanded={verdictPickerExpanded}>
      {#snippet toggle(onclick)}
        <button
          type="button"
          class="verdict-chip verdict-toggle"
          class:accept={verdict === "accept"}
          class:reject={verdict === "reject"}
          aria-haspopup="menu"
          aria-expanded={verdictPickerExpanded}
          disabled={savingVerdict}
          {onclick}>
          <Icon name={verdictIcon(verdict)} />
          {verdictLabel}
          <Icon name={verdictPickerExpanded ? "chevron-up" : "chevron-down"} />
        </button>
      {/snippet}
      {#snippet popover()}
        <div
          style:border="1px solid var(--color-border-subtle)"
          style:border-radius="var(--border-radius-sm)"
          style:background-color="var(--color-surface-canvas)"
          style:min-width="10rem">
          <DropdownList items={verdictOptions}>
            {#snippet item(option)}
              {@const disabled = option.value === undefined && summary === ""}
              <DropdownListItem
                selected={verdict === option.value}
                {disabled}
                title={disabled
                  ? "Add a summary before switching to Comment"
                  : undefined}
                styleGap="0.5rem"
                onclick={() => setVerdict(option.value)}>
                <span
                  class:verdict-accept={option.value === "accept"}
                  class:verdict-reject={option.value === "reject"}>
                  <Icon name={verdictIcon(option.value)} />
                </span>
                {option.label}
              </DropdownListItem>
            {/snippet}
          </DropdownList>
        </div>
      {/snippet}
    </Popover>
  {:else}
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
  {/if}
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
  {#if isOwnPublishedReview}
    <span style:margin-left="auto"></span>
    <Popover
      popoverPadding="0"
      placement="bottom-end"
      bind:expanded={deleteConfirmExpanded}>
      {#snippet toggle(onclick)}
        <button
          type="button"
          class="action-icon"
          title="Delete review"
          aria-haspopup="dialog"
          aria-expanded={deleteConfirmExpanded}
          disabled={deleting}
          {onclick}>
          <Icon name="trash" />
        </button>
      {/snippet}
      {#snippet popover()}
        <div class="delete-confirm">
          <div class="delete-confirm-message">
            Delete this review? This cannot be undone.
          </div>
          <div class="delete-confirm-actions">
            <Button variant="naked" onclick={() => closeFocused()}>
              Cancel
            </Button>
            <button
              type="button"
              class="delete-confirm-button"
              onclick={deleteReview}
              disabled={deleting}>
              <Icon name="trash" />
              Delete review
            </button>
          </div>
        </div>
      {/snippet}
    </Popover>
  {/if}
</div>

{#if editingSummary}
  <div class="summary-editor">
    <ExtendedTextarea
      {rid}
      body={review.summary ?? ""}
      focus
      submitCaption={savingSummary ? "Saving…" : "Save"}
      disableSubmit={savingSummary}
      submit={async ({ comment }) => {
        await saveSummary(comment);
      }}
      close={() => (editingSummary = false)} />
  </div>
{:else if summary !== "" || isOwnPublishedReview}
  <div class="summary">
    {#if summary !== ""}
      <Markdown {rid} breaks content={summary} />
    {:else}
      <span class="summary-empty">No summary.</span>
    {/if}
    {#if isOwnPublishedReview}
      <button
        type="button"
        class="action-icon summary-edit"
        title="Edit summary"
        onclick={() => (editingSummary = true)}>
        <Icon name="edit" />
      </button>
    {/if}
  </div>
{/if}

{#if fileGroups.length > 0 && reviewedRevision}
  <div class="threads">
    {#each fileGroups as group (group.path)}
      {#each group.threads as thread (thread.root.id)}
        <ReviewCodeThread
          base={reviewedRevision.base}
          head={reviewedRevision.head}
          {thread}
          {config}
          {repoDelegates}
          {rid}
          editComment={editCodeComment}
          deleteComment={deleteCodeComment} />
      {/each}
    {/each}
  </div>
{/if}
