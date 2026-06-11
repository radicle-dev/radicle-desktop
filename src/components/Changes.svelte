<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Commit } from "@bindings/repo/Commit";

  import {
    cachedDiffStats,
    cachedGetDiff,
    cachedListCommits,
  } from "@app/lib/invoke";
  import { pluralize } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Changeset from "@app/components/Changeset.svelte";
  import CobCommitTeaser from "@app/components/CobCommitTeaser.svelte";
  import CommitsContainer from "@app/components/CommitsContainer.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import JobCob from "@app/components/JobCob.svelte";
  import { getScrollViewport } from "@app/components/ScrollArea.svelte";

  interface Props {
    patchId: string;
    revision: Revision;
    rid: string;
    codeComments?: CodeComments;
    draftReviewId?: string;
    filesExpanded?: boolean;
  }

  /* eslint-disable prefer-const */
  let {
    patchId,
    revision,
    rid,
    codeComments,
    draftReviewId,
    filesExpanded = $bindable(true),
  }: Props = $props();
  /* eslint-enable prefer-const */

  let selectedCommit = $state<string>();
  let selectedCommitData = $state<Commit>();
  // Parent reuses this component across patch revisions; a sibling $effect
  // resets base and head when patchId changes.
  // svelte-ignore state_referenced_locally
  let base = $state(revision.base);
  // svelte-ignore state_referenced_locally
  let head = $state(revision.head);

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    filesExpanded = true;
    selectedCommit = undefined;
    selectedCommitData = undefined;
    base = revision.base;
    head = revision.head;
  });

  function selectRevision({
    headId,
    baseId,
    commitId = undefined,
    commit = undefined,
    showFiles = true,
  }: {
    headId: string;
    baseId: string;
    commitId?: string;
    commit?: Commit;
    showFiles?: boolean;
  }) {
    head = headId;
    base = baseId;
    selectedCommit = commitId;
    selectedCommitData = commit;
    filesExpanded = showFiles;
  }

  const isActiveCommit = (commitId: string) => selectedCommit === commitId;
  const isTeaserDisabled = (commitId: string) =>
    selectedCommit ? selectedCommit !== commitId : false;

  // Make the Changes tab a fixed-height workspace: the two-column area fills
  // the remaining viewport height and each column scrolls on its own, so the
  // page doesn't scroll here and nothing resizes mid-scroll. Height is measured
  // once (and on layout changes), not on every scroll event.
  let reviewLayout = $state<HTMLElement>();
  const getViewport = getScrollViewport();
  $effect(() => {
    // Recompute when the review bar appears or disappears.
    void draftReviewId;
    const update = () => {
      const el = reviewLayout;
      const vp = getViewport();
      if (!el || !vp) return;
      if (window.matchMedia("(max-width: 60rem)").matches) {
        el.style.height = "";
        return;
      }
      const offset =
        el.getBoundingClientRect().top - vp.getBoundingClientRect().top;
      el.style.height = `${Math.max(vp.clientHeight - offset - 24, 200)}px`;
    };
    update();
    const vp = getViewport();
    const observer = new ResizeObserver(() => update());
    if (vp) observer.observe(vp);
    window.addEventListener("resize", update);
    const settle = setTimeout(update, 50);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", update);
      clearTimeout(settle);
    };
  });
</script>

<style>
  .review-layout {
    display: grid;
    grid-template-columns: minmax(15rem, 22rem) minmax(0, 1fr);
    gap: 1rem;
    align-items: stretch;
    min-height: 0;
  }
  .commits-column {
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-canvas);
  }
  .diff-column {
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
  }
  @media (max-width: 60rem) {
    .review-layout {
      grid-template-columns: 1fr;
    }
    .commits-column,
    .diff-column {
      overflow-y: visible;
    }
  }
  .stats-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .stats {
    min-width: 0;
  }
  .selected-commit-message {
    margin-bottom: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .selected-commit-body {
    margin: 0.5rem 0 0;
    white-space: pre-wrap;
    word-wrap: break-word;
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
  }
  .commit-stats {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    white-space: nowrap;
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
  }
  .commit-stats .insertions {
    color: var(--color-feedback-success-text);
    margin-left: 0.5rem;
  }
  .commit-stats .deletions {
    color: var(--color-feedback-error-text);
  }
  .commits {
    display: flex;
    flex-direction: column;
    font: var(--txt-body-m-regular);
    padding: 0.5rem 0;
  }
  .commit {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    padding: 0.5rem 1rem;
  }
  .commit + .commit {
    border-top: 1px solid var(--color-border-subtle);
  }
  .commit > :global(.teaser) {
    flex: 1;
    min-width: 0;
  }
  .commit:hover:not(.single-commit) {
    background-color: var(--color-surface-subtle);
  }
  .commit.active {
    background-color: var(--color-surface-subtle);
  }
  .summary {
    padding: 0.25rem 0;
  }
  .single-commit {
    cursor: default !important;
  }
</style>

<div>
  {#await cachedDiffStats(rid, revision.base, revision.head) then stats}
    <div class="stats-row txt-body-m-regular">
      <div class="stats" style:color="var(--color-text-secondary)">
        {stats.filesChanged}
        {pluralize("file", stats.filesChanged)} modified with
        <span style:color="var(--color-feedback-success-text)">
          {stats.insertions}
          {pluralize("insertion", stats.insertions)}
        </span>
        and
        <span style:color="var(--color-feedback-error-text)">
          {stats.deletions}
          {pluralize("deletion", stats.deletions)}
        </span>
      </div>
      {#if stats.filesChanged > 0}
        <Button
          variant="naked"
          onclick={() => (filesExpanded = !filesExpanded)}>
          {#if filesExpanded}
            <Icon name="collapse-vertical" />
            Collapse all
          {:else}
            <Icon name="expand-vertical" />
            Expand all
          {/if}
        </Button>
      {/if}
    </div>
  {/await}
  <div class="review-layout" bind:this={reviewLayout}>
    <div class="commits-column">
      {#await cachedListCommits(rid, revision.base, revision.head) then commits}
        <CommitsContainer>
          {#snippet leftHeader()}
            <div class="global-flex txt-body-m-regular summary">
              {commits.length}
              {pluralize("commit", commits.length)} on base
              <Id
                id={revision.base}
                clipboard={revision.base}
                label="base commit" />
              <div class="global-chip">Base</div>
            </div>
          {/snippet}
          <div class="commits">
            {#each commits as commit}
              {@const active = isActiveCommit(commit.id)}
              {@const toggle = () => {
                if (commits.length === 1) return;
                if (active) {
                  selectRevision({
                    headId: revision.head,
                    baseId: revision.base,
                  });
                } else {
                  selectRevision({
                    headId: commit.id,
                    baseId: commit.parents[0],
                    commitId: commit.id,
                    commit,
                  });
                }
              }}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="commit"
                class:single-commit={commits.length === 1}
                class:active
                onclick={toggle}>
                <CobCommitTeaser
                  stacked
                  hoverable={commits.length > 1}
                  disabled={isTeaserDisabled(commit.id)}
                  {commit}>
                  {#if commit.parents.length > 0}
                    {#await cachedDiffStats(rid, commit.parents[0], commit.id) then stats}
                      <span class="commit-stats">
                        <Icon name="document" />
                        {stats.filesChanged}
                        <span class="insertions">+{stats.insertions}</span>
                        <span class="deletions">-{stats.deletions}</span>
                      </span>
                    {/await}
                  {/if}
                  {#if commit.id === revision.head}
                    <JobCob {rid} commit={commit.id} />
                  {/if}
                </CobCommitTeaser>
                {#if active}
                  <span class="global-icon-button" title="Show all changes">
                    <Icon name="close" />
                  </span>
                {/if}
              </div>
            {/each}
          </div>
        </CommitsContainer>
      {/await}
    </div>
    <div class="diff-column">
      {#if selectedCommitData}
        <div class="selected-commit-message">
          <div class="selected-commit-summary txt-body-m-medium">
            {selectedCommitData.summary}
          </div>
          {#if selectedCommitData.message.trim() !== selectedCommitData.summary.trim()}
            <pre class="selected-commit-body">{selectedCommitData.message
                .replace(selectedCommitData.summary, "")
                .trim()}</pre>
          {/if}
        </div>
      {/if}
      {#await cachedGetDiff(rid, { base, head, unified: 3, highlight: true })}
        <span class="txt-body-m-regular">Loading…</span>
      {:then diff}
        <Changeset
          expanded={filesExpanded}
          {head}
          {diff}
          {rid}
          {codeComments}
          {draftReviewId} />
      {/await}
    </div>
  </div>
</div>
