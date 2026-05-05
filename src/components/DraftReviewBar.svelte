<script lang="ts">
  import type { Review } from "@bindings/cob/patch/Review";

  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import { draftReviewStorage } from "@app/lib/draftReviewStorage";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";
  import { closeFocused } from "@app/components/Popover.svelte";

  interface Props {
    draftReview: DraftReview;
    onChange: () => Promise<void>;
    onPublish: () => Promise<void>;
    onCancel: () => void;
  }

  const { draftReview, onChange, onPublish, onCancel }: Props = $props();

  let summary = $state(draftReview.summary ?? "");
  let verdict: Review["verdict"] = $state(draftReview.verdict ?? "accept");
  let publishing = $state(false);
  let dropdownExpanded = $state(false);
  let summaryEl: HTMLTextAreaElement | undefined = $state();

  const expanded = $derived(summary.includes("\n") || summary.length > 80);

  function autoResizeSummary() {
    if (!summaryEl) return;
    summaryEl.style.height = "auto";
    summaryEl.style.height = `${summaryEl.scrollHeight}px`;
  }

  $effect(() => {
    void summary;
    void expanded;
    autoResizeSummary();
  });

  type VerdictOption = {
    value: Review["verdict"];
    label: string;
    description: string;
  };

  const verdictOptions: VerdictOption[] = [
    {
      value: "accept",
      label: "Accept revision",
      description: "Approve the revision and publish your review.",
    },
    {
      value: "reject",
      label: "Reject revision",
      description: "Request changes before merging.",
    },
    {
      value: undefined,
      label: "Leave a comment",
      description: "Comment without approving or rejecting.",
    },
  ];

  const pendingCommentCount = $derived(
    draftReview.comments.filter(c => !c.replyTo).length,
  );

  const verdictLabel = $derived(
    verdict === "accept"
      ? "Accept revision"
      : verdict === "reject"
        ? "Reject revision"
        : "Leave a comment",
  );

  const verdictColorClass = $derived(
    verdict === "accept"
      ? "accept"
      : verdict === "reject"
        ? "reject"
        : "comment",
  );

  async function persistSummary() {
    draftReviewStorage.update(draftReview.id, {
      summary,
      verdict,
      labels: draftReview.labels,
    });
    await onChange();
  }

  async function publish() {
    publishing = true;
    try {
      draftReviewStorage.update(draftReview.id, {
        summary,
        verdict,
        labels: draftReview.labels,
      });
      await draftReviewStorage.publish(draftReview.id);
      await onPublish();
    } finally {
      publishing = false;
    }
  }

  const publishDisabled = $derived(
    publishing || (verdict === undefined && summary.trim() === ""),
  );
</script>

<style>
  .bar {
    flex-shrink: 0;
    background-color: var(--color-surface-canvas);
    border-top: 1px solid var(--color-border-subtle);
    padding: 1rem;
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 1.5rem;
  }
  .bar.expanded {
    flex-direction: column;
    align-items: stretch;
    gap: 0.75rem;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }
  .summary {
    flex: 1;
    min-width: 0;
    box-sizing: border-box;
    background: transparent;
    border: 0;
    outline: none;
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    padding: 0.25rem 0;
    resize: none;
    overflow-y: auto;
    line-height: 1.4;
    max-height: 12rem;
  }
  .bar.expanded .summary {
    flex: 0 1 auto;
    width: 100%;
  }
  .summary::placeholder {
    color: var(--color-text-quaternary);
  }
  .status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-text-quaternary);
    font: var(--txt-body-m-regular);
  }
  .chip {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    padding: 0.125rem 0.5rem;
    color: var(--color-text-primary);
  }
  .saved-count {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }
  .split-button {
    display: flex;
    align-items: stretch;
    gap: 0.25rem;
  }
  .verdict-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    border: none;
    border-radius: var(--border-radius-sm);
    height: 2rem;
    padding: 0 0.5rem;
    cursor: pointer;
    white-space: nowrap;
    -webkit-user-select: none;
    user-select: none;
    transition: background-color 0.1s ease;
    color: var(--color-text-on-brand);
  }
  .verdict-btn:disabled {
    cursor: default;
    opacity: 0.6;
  }
  .verdict-btn.accept {
    background-color: var(--color-feedback-success-fill);
  }
  .verdict-btn.accept:hover:not(:disabled) {
    background-color: var(--color-feedback-success-fill-hover);
  }
  .verdict-btn.accept:active:not(:disabled),
  .verdict-btn.accept.active:not(:disabled) {
    background-color: var(--color-feedback-success-fill-active);
  }
  .verdict-btn.reject {
    background-color: var(--color-feedback-error-fill);
  }
  .verdict-btn.reject:hover:not(:disabled) {
    background-color: var(--color-feedback-error-fill-hover);
  }
  .verdict-btn.reject:active:not(:disabled),
  .verdict-btn.reject.active:not(:disabled) {
    background-color: var(--color-feedback-error-fill-active);
  }
  .verdict-btn.comment {
    background-color: var(--color-surface-brand-secondary);
  }
  .verdict-btn.comment:active:not(:disabled),
  .verdict-btn.comment.active:not(:disabled) {
    background-color: var(--color-surface-brand-primary);
  }
  .verdict-btn.caret {
    width: 2rem;
    padding: 0;
  }
  .verdict-menu {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    width: 22rem;
    overflow: hidden;
  }
  .verdict-option {
    display: grid;
    grid-template-columns: 1.5rem 1fr;
    column-gap: 0.5rem;
    align-items: start;
    padding: 0.625rem 0.75rem;
    cursor: pointer;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .verdict-option:last-child {
    border-bottom: none;
  }
  .verdict-option:hover {
    background-color: var(--color-surface-subtle);
  }
  .verdict-check {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 1.25rem;
    color: var(--color-text-primary);
  }
  .verdict-text {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }
  .verdict-label {
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
  }
  .verdict-description {
    font: var(--txt-body-s-regular);
    color: var(--color-text-tertiary);
    white-space: normal;
  }
</style>

<div class="bar" class:expanded>
  <textarea
    bind:this={summaryEl}
    class="summary"
    rows="1"
    placeholder="Add optional review comment"
    bind:value={summary}
    oninput={autoResizeSummary}
    onblur={persistSummary}>
  </textarea>

  <div class="actions">
    <div class="status">
      <span class="chip">Draft saved</span>
      <span class="saved-count">
        <Icon name="comment" />
        {pendingCommentCount}
      </span>
    </div>

    <div style:margin-left="auto" style:display="flex" style:gap="0.5rem">
      <Button variant="outline" disabled={publishing} onclick={onCancel}>
        Discard
      </Button>

      <div class="split-button">
        <button
          class="verdict-btn txt-body-m-medium {verdictColorClass}"
          disabled={publishDisabled}
          onclick={publish}>
          {verdictLabel}
        </button>
        <Popover
          popoverPadding="0"
          placement="top-end"
          bind:expanded={dropdownExpanded}>
          {#snippet toggle(onclick)}
            <button
              class="verdict-btn caret {verdictColorClass}"
              class:active={dropdownExpanded}
              disabled={publishing}
              {onclick}>
              <Icon name={dropdownExpanded ? "chevron-up" : "chevron-down"} />
            </button>
          {/snippet}
          {#snippet popover()}
            <div class="verdict-menu">
              {#each verdictOptions as option (option.label)}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <div
                  role="button"
                  tabindex="0"
                  class="verdict-option"
                  onclick={() => {
                    verdict = option.value;
                    draftReviewStorage.update(draftReview.id, {
                      summary,
                      verdict: option.value,
                      labels: draftReview.labels,
                    });
                    closeFocused();
                    void onChange();
                  }}>
                  <div class="verdict-check">
                    {#if verdict === option.value}
                      <Icon name="checkmark" />
                    {/if}
                  </div>
                  <div class="verdict-text">
                    <span class="verdict-label">{option.label}</span>
                    <span class="verdict-description">
                      {option.description}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/snippet}
        </Popover>
      </div>
    </div>
  </div>
</div>
