<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Commit } from "@bindings/repo/Commit";

  import { tick } from "svelte";

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
  import Markdown from "@app/components/Markdown.svelte";
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

  // Load the commit list into state so keyboard navigation can step through it.
  let commitList = $state<Commit[]>([]);
  $effect(() => {
    const ridLocal = rid;
    const baseRev = revision.base;
    const headRev = revision.head;
    let cancelled = false;
    void cachedListCommits(ridLocal, baseRev, headRev).then(c => {
      if (!cancelled) commitList = c;
    });
    return () => {
      cancelled = true;
    };
  });

  // The revision's own description, shown above the lists — unless it's empty
  // or just the auto-generated list of commit summaries.
  function isCommitListDescription(description: string, commits: Commit[]) {
    if (commits.length === 0) return false;
    const chunks = description
      .split("\n")
      .map(l => l.trim())
      .filter(l => l.length > 0);
    if (chunks.length !== commits.length) return false;
    const summaries = new Set(commits.map(c => c.summary.trim()));
    return chunks.every(line => summaries.has(line));
  }
  const revisionDescription = $derived(
    revision.description.slice(-1)[0]?.body?.trim() ?? "",
  );
  const showRevisionDescription = $derived(
    revisionDescription !== "" &&
      !isCommitListDescription(revisionDescription, commitList),
  );

  // Collapse a long revision description behind a Show more/less toggle.
  const DESCRIPTION_MAX_HEIGHT = 300;
  let descriptionExpanded = $state(false);
  let descriptionEl = $state<HTMLElement>();
  let descriptionOverflows = $state(false);
  const descriptionCollapsed = $derived(
    descriptionOverflows && !descriptionExpanded,
  );
  $effect(() => {
    const el = descriptionEl;
    if (!el) return;
    const measure = () => {
      descriptionOverflows = el.scrollHeight > DESCRIPTION_MAX_HEIGHT;
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  });

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

  function selectCommitAt(index: number) {
    if (commitList.length <= 1) return;
    const clamped = Math.max(0, Math.min(index, commitList.length - 1));
    const commit = commitList[clamped];
    selectRevision({
      headId: commit.id,
      baseId: commit.parents[0],
      commitId: commit.id,
      commit,
    });
    void tick().then(() => {
      reviewLayout
        ?.querySelector(".commit.active")
        ?.scrollIntoView({ block: "nearest" });
    });
  }

  // Up/Down step through commits; Escape deselects. Ignored while typing.
  $effect(() => {
    const onKeydown = (e: KeyboardEvent) => {
      const el = document.activeElement;
      if (
        el instanceof HTMLElement &&
        (el.tagName === "INPUT" ||
          el.tagName === "TEXTAREA" ||
          el.isContentEditable)
      ) {
        return;
      }
      if (e.key === "Escape") {
        if (selectedCommit) {
          e.preventDefault();
          selectRevision({ headId: revision.head, baseId: revision.base });
        }
        return;
      }
      if (commitList.length <= 1) return;
      const current = commitList.findIndex(c => c.id === selectedCommit);
      if (e.key === "ArrowDown") {
        e.preventDefault();
        selectCommitAt(current === -1 ? 0 : current + 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        selectCommitAt(current === -1 ? commitList.length - 1 : current - 1);
      }
    };
    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });
</script>

<style>
  .revision-description {
    position: relative;
    margin-bottom: 1rem;
    color: var(--color-text-primary);
  }
  .revision-description.collapsed .revision-description-body {
    max-height: 300px;
    overflow: hidden;
  }
  .revision-description-toggle {
    display: flex;
    justify-content: center;
    margin-top: 0.5rem;
  }
  .revision-description.collapsed .revision-description-toggle {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    align-items: flex-end;
    height: 6rem;
    margin: 0;
    padding-bottom: 0.25rem;
    background: linear-gradient(
      to bottom,
      transparent,
      var(--color-surface-canvas)
    );
    pointer-events: none;
  }
  /* Identical to the "View all revision changes" button (.diff-tease-button). */
  .revision-description-button {
    pointer-events: auto;
    position: relative;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    color: var(--color-text-primary);
    cursor: pointer;
    box-shadow: var(--elevation-low);
  }
  .revision-description-button:hover,
  .revision-description-button:focus-visible {
    background-color: var(--color-surface-subtle);
  }
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
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  /* The diffs scroll inside their own box, so they stay below the fixed
     header instead of sliding up under it. File headers stick to the top of
     this box (right beneath the header). */
  .diff-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    border-radius: var(--border-radius-md);
  }
  @media (max-width: 60rem) {
    .review-layout {
      grid-template-columns: 1fr;
    }
    .commits-column,
    .diff-scroll {
      overflow-y: visible;
    }
    .diff-column {
      display: block;
    }
  }
  .stats-row {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 1rem;
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-canvas);
  }
  .stats {
    min-width: 0;
  }
  .selected-commit-message {
    margin-bottom: 1rem;
    padding: 0.75rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-canvas);
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
  {#if showRevisionDescription}
    <div class="revision-description" class:collapsed={descriptionCollapsed}>
      <div
        class="revision-description-body txt-body-m-regular"
        bind:this={descriptionEl}>
        <Markdown {rid} breaks content={revisionDescription} />
      </div>
      {#if descriptionOverflows}
        <div class="revision-description-toggle">
          <button
            type="button"
            class="revision-description-button txt-body-m-medium"
            onclick={() => (descriptionExpanded = !descriptionExpanded)}>
            {descriptionExpanded ? "Show less" : "Show more"}
            <Icon
              name={descriptionExpanded
                ? "collapse-vertical"
                : "expand-vertical"} />
          </button>
        </div>
      {/if}
    </div>
  {/if}
  <div class="review-layout" bind:this={reviewLayout}>
    <div class="commits-column">
      {#if commitList.length > 0}
        <CommitsContainer>
          {#snippet leftHeader()}
            <div class="global-flex txt-body-m-regular summary">
              {commitList.length}
              {pluralize("commit", commitList.length)} on base
              <Id
                id={revision.base}
                clipboard={revision.base}
                label="base commit" />
              <div class="global-chip">Base</div>
            </div>
          {/snippet}
          <div class="commits">
            {#each commitList as commit}
              {@const active = isActiveCommit(commit.id)}
              {@const toggle = () => {
                if (commitList.length === 1) return;
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
                class:single-commit={commitList.length === 1}
                class:active
                onclick={toggle}>
                <CobCommitTeaser
                  stacked
                  hoverable={commitList.length > 1}
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
      {/if}
    </div>
    <div class="diff-column">
      {#await cachedDiffStats(rid, base, head) then stats}
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
      <div class="diff-scroll">
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
</div>
