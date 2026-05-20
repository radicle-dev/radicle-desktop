<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Verdict } from "@bindings/cob/patch/Verdict";
  import type { Config } from "@bindings/config/Config";
  import type { Stats } from "@bindings/diff/Stats";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import { push } from "@app/lib/router";
  import {
    authorForNodeId,
    pluralize,
    publicKeyFromDid,
    verdictIcon,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";

  interface Props {
    config: Config;
    loadPatch: () => Promise<void>;
    patch: Patch;
    repo: RepoInfo;
    revisions: Revision[];
    stats?: Stats;
    onShowChanges?: () => void;
  }

  const {
    config,
    loadPatch,
    patch,
    repo,
    revisions,
    stats,
    onShowChanges,
  }: Props = $props();

  type ReviewerSummary = {
    author: Author;
    verdict?: Verdict;
    reviewId: string;
  };

  const reviewers: ReviewerSummary[] = $derived.by(() => {
    const sorted = [...revisions].sort((a, b) => a.timestamp - b.timestamp);
    const map = new Map<string, ReviewerSummary>();
    for (const rev of sorted) {
      for (const review of rev.reviews ?? []) {
        map.set(review.author.did, {
          author: review.author,
          verdict: review.verdict,
          reviewId: review.id,
        });
      }
    }
    return [...map.values()];
  });

  let reviewersPopoverExpanded = $state(false);

  function openReview(reviewId: string) {
    reviewersPopoverExpanded = false;
    closeFocused();
    void push({
      resource: "repo.patch",
      rid: repo.rid,
      patch: patch.id,
      status: undefined,
      reviewId,
    });
  }

  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);

  async function saveLabels(labels: string[]) {
    try {
      labelSaveInProgress = true;
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "label",
          labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing labels failed", error);
    } finally {
      labelSaveInProgress = false;
      await loadPatch();
    }
  }

  async function saveAssignees(assignees: Author[]) {
    try {
      assigneesSaveInProgress = true;
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "assign",
          assignees,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing assignees failed", error);
    } finally {
      assigneesSaveInProgress = false;
      await loadPatch();
    }
  }
</script>

<style>
  .meta-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    font: var(--txt-body-m-regular);
  }
  .stats {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    color: var(--color-text-tertiary);
    font: var(--txt-body-m-regular);
  }
  .stats.stats-button {
    cursor: pointer;
  }
  .stats.stats-button:hover,
  .stats.stats-button:focus-visible {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .stats .insertions {
    color: var(--color-feedback-success-text);
  }
  .stats .deletions {
    color: var(--color-feedback-error-text);
  }
  .reviews {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    color: var(--color-text-tertiary);
    cursor: pointer;
    font: var(--txt-body-m-regular);
  }
  .reviews:hover,
  .reviews:focus-visible {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .reviewer-stack {
    display: inline-flex;
    align-items: center;
  }
  .reviewer-stack :global(img) {
    outline: 1px solid var(--color-surface-canvas);
    margin-left: -0.375rem;
  }
  .reviewer-stack :global(img:first-child) {
    margin-left: 0;
  }
  .reviewer-overflow {
    margin-left: 0.25rem;
    color: var(--color-text-tertiary);
  }
  .verdict-accept {
    color: var(--color-feedback-success-text);
  }
  .verdict-reject {
    color: var(--color-feedback-error-text);
  }
</style>

<div class="meta-row">
  {#if stats}
    {#if onShowChanges}
      <button
        type="button"
        class="stats stats-button"
        onclick={onShowChanges}
        title="View changed files">
        <Icon name="diff" />
        <span>
          {stats.filesChanged}
          {pluralize("file", stats.filesChanged)}
        </span>
        <span class="insertions">+{stats.insertions}</span>
        <span class="deletions">-{stats.deletions}</span>
      </button>
    {:else}
      <div class="stats">
        <Icon name="diff" />
        <span>
          {stats.filesChanged}
          {pluralize("file", stats.filesChanged)}
        </span>
        <span class="insertions">+{stats.insertions}</span>
        <span class="deletions">-{stats.deletions}</span>
      </div>
    {/if}
  {/if}
  {#if reviewers.length > 0}
    {@const verdicts = reviewers.map(r => r.verdict)}
    {@const hasReject = verdicts.includes("reject")}
    {@const allAccept =
      verdicts.length > 0 && verdicts.every(v => v === "accept")}
    <div class="reviewers-compact">
      <Popover
        popoverPadding="0"
        placement="bottom-start"
        bind:expanded={reviewersPopoverExpanded}>
        {#snippet toggle(onclick)}
          <button
            type="button"
            class="reviews"
            {onclick}
            aria-haspopup="menu"
            aria-expanded={reviewersPopoverExpanded}
            title="{reviewers.length} {pluralize('reviewer', reviewers.length)}">
            <span
              class:verdict-accept={allAccept}
              class:verdict-reject={hasReject}>
              <Icon
                name={hasReject
                  ? "stop"
                  : allAccept
                    ? "thumbs-up"
                    : "comment"} />
            </span>
            <span>
              {reviewers.length}
              {pluralize("review", reviewers.length)}
            </span>
            <span class="reviewer-stack">
              {#each reviewers.slice(0, 3) as reviewer (reviewer.author.did)}
                <UserAvatar
                  nodeId={publicKeyFromDid(reviewer.author.did)}
                  styleWidth="1.125rem" />
              {/each}
              {#if reviewers.length > 3}
                <span class="reviewer-overflow">+{reviewers.length - 3}</span>
              {/if}
            </span>
          </button>
        {/snippet}
        {#snippet popover()}
          <div
            style:border="1px solid var(--color-border-subtle)"
            style:border-radius="var(--border-radius-sm)"
            style:background-color="var(--color-surface-canvas)"
            style:min-width="14rem">
            <DropdownList items={reviewers}>
              {#snippet item(reviewer)}
                <DropdownListItem
                  selected={false}
                  styleGap="0.5rem"
                  onclick={() => openReview(reviewer.reviewId)}>
                  <span
                    class:verdict-accept={reviewer.verdict === "accept"}
                    class:verdict-reject={reviewer.verdict === "reject"}>
                    <Icon name={verdictIcon(reviewer.verdict)} />
                  </span>
                  <NodeId {...authorForNodeId(reviewer.author)} />
                  <span
                    style:margin-left="auto"
                    style:color="var(--color-text-quaternary)">
                    View
                  </span>
                </DropdownListItem>
              {/snippet}
            </DropdownList>
          </div>
        {/snippet}
      </Popover>
    </div>
  {/if}
  <LabelInput
    allowedToEdit={!!roles.isDelegate(
      config.publicKey,
      repo.delegates.map(delegate => delegate.did),
    )}
    labels={patch.labels}
    submitInProgress={labelSaveInProgress}
    save={saveLabels} />
  <AssigneeInput
    allowedToEdit={!!roles.isDelegate(
      config.publicKey,
      repo.delegates.map(delegate => delegate.did),
    )}
    assignees={patch.assignees}
    submitInProgress={assigneesSaveInProgress}
    save={saveAssignees} />
</div>
