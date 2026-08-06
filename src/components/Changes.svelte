<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Commit } from "@bindings/repo/Commit";

  import { tick, untrack } from "svelte";

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
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
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
    // Reported upward so the review bar knows whether its per-file progress
    // applies to what's on screen.
    showingRevisionDiff?: boolean;
    filesExpanded?: boolean;
    canEditDescription?: boolean;
    onSaveDescription?: (body: string, embeds: Embed[]) => Promise<void>;
  }

  /* eslint-disable prefer-const */
  let {
    patchId,
    revision,
    rid,
    codeComments,
    draftReviewId,
    showingRevisionDiff = $bindable(true),
    filesExpanded = $bindable(true),
    canEditDescription = false,
    onSaveDescription,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let selectedCommit = $state<string>();
  let selectedCommitData = $state<Commit>();
  let editingDescription = $state(false);
  let diffScrollEl = $state<HTMLElement>();
  // Parent reuses this component across patch revisions; a sibling $effect
  // resets base and head when patchId changes.
  // svelte-ignore state_referenced_locally
  let base = $state(revision.base);
  // svelte-ignore state_referenced_locally
  let head = $state(revision.head);

  // Reset the view when the patch or revision being shown actually changes.
  // The effect re-runs far more often than that: reading the `revision` prop
  // depends on its object identity, and reloading the patch after a comment or
  // reaction hands down a fresh one with the same contents. Guarding on the ids
  // keeps those re-runs no-ops, instead of dropping the selected commit and
  // resetting the diff out from under the reader's scroll position.
  // svelte-ignore state_referenced_locally
  let viewedKey = `${patchId}:${revision.id}`;
  $effect(() => {
    const key = `${patchId}:${revision.id}`;
    const nextBase = revision.base;
    const nextHead = revision.head;
    untrack(() => {
      if (key === viewedKey) return;
      viewedKey = key;
      filesExpanded = true;
      selectedCommit = undefined;
      selectedCommitData = undefined;
      editingDescription = false;
      base = nextBase;
      head = nextHead;
    });
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

  // Whether the diff on screen is the revision's own changeset. Selecting a
  // commit normally narrows it, but a single-commit revision's commit diff is
  // the revision diff, so reviewing stays available there. This, rather than
  // "a commit is selected", is what the review affordances depend on.
  const isRevisionDiff = $derived(
    base === revision.base && head === revision.head,
  );
  $effect(() => {
    if (showingRevisionDiff !== isRevisionDiff) {
      showingRevisionDiff = isRevisionDiff;
    }
  });

  // Code comments belong to the revision. A `CodeLocation` records the commit
  // it was written against but not the base it was diffed from, and thread
  // placement matches on path and line number alone — so a thread created or
  // shown against a narrower diff can't be tied back to the revision reliably.
  // A narrowed view therefore carries no comment layer at all.
  const diffCodeComments = $derived(isRevisionDiff ? codeComments : undefined);

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

  // A single-commit revision has nothing to choose between, so select that
  // commit automatically to surface its message.
  $effect(() => {
    if (commitList.length === 1 && !selectedCommit) {
      const only = commitList[0];
      selectRevision({
        headId: only.id,
        baseId: only.parents[0],
        commitId: only.id,
        commit: only,
      });
    }
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
  const DESCRIPTION_MAX_HEIGHT = 150;
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

  // The Changes tab scrolls with the page (see the CSS): the whole patch
  // scrolls so the headers move off-screen and the diff gets the full height,
  // while the commit list is sticky and stays in view alongside it.
  let reviewLayout = $state<HTMLElement>();
  const getViewport = getScrollViewport();

  // Bring the top of the diff into view after the reader changes what the diff
  // column shows; otherwise the old scroll position leaves you mid-diff and the
  // new commit message looks missing.
  //
  // Driven by the click that caused it rather than by watching `selectedCommit`.
  // That selection is also set programmatically — a single-commit revision
  // auto-selects its only commit — so an effect could not tell "the reader
  // picked a commit" from "the revision changed underneath them", and scrolled
  // on both.
  let scrollRequest = 0;
  async function scrollToDiff() {
    const request = ++scrollRequest;
    // Measure only once the column above the diff has settled. The stats row
    // sits in an `{#await}` keyed on base..head, so a cold cache unmounts it and
    // the diff jumps up by its height — measuring first would aim the scroll at
    // a position that no longer exists by the time it lands.
    await cachedDiffStats(rid, base, head).catch(() => undefined);
    await tick();
    // A newer click already started its own scroll; that one wins.
    if (request !== scrollRequest) return;
    const vp = getViewport();
    const el = diffScrollEl;
    if (!vp || !el) return;
    const top =
      el.getBoundingClientRect().top -
      vp.getBoundingClientRect().top +
      vp.scrollTop;
    vp.scrollTo({ top: Math.max(top, 0), behavior: "smooth" });
  }

  let changeset = $state<ReturnType<typeof Changeset> | undefined>();

  /// Scroll to one of the draft review's code comments. Called from the review
  /// bar, which sits outside this component.
  export async function revealComment(threadId: string, path: string) {
    // Code comments are only superimposed on the revision's own diff, so a view
    // narrowed to a single commit has none to scroll to. Widen back first.
    if (!isRevisionDiff) {
      selectRevision({ headId: revision.head, baseId: revision.base });
      // The widened diff is a fresh `{#await}`, so `changeset` is remounted;
      // wait for it before asking it to scroll.
      await tick();
      await cachedGetDiff(rid, {
        base: revision.base,
        head: revision.head,
        unified: 3,
        highlight: true,
      }).catch(() => undefined);
      await tick();
    }
    await changeset?.revealThread(threadId, path);
  }

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
    void scrollToDiff();
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
          void scrollToDiff();
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
  .revision-description:has(.revision-description-actions)
    .revision-description-body {
    padding-right: 2.5rem;
  }
  .revision-description-actions {
    position: absolute;
    top: 0;
    right: 0;
    display: flex;
    align-items: center;
  }
  .revision-description-edit {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    padding: 0.25rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    color: var(--color-text-tertiary);
    opacity: 0;
    transition: opacity 150ms ease;
  }
  .revision-description:hover .revision-description-edit,
  .revision-description-edit:focus-visible {
    opacity: 1;
  }
  .revision-description-edit:hover,
  .revision-description-edit:focus-visible {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .revision-description.collapsed .revision-description-body {
    max-height: 150px;
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
    height: 3rem;
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
    align-items: start;
    min-height: 0;
  }
  /* The whole patch scrolls with the page; the commit list sticks to the top of
     the viewport and scrolls internally only when it's taller than the screen,
     so it stays in view while you scroll through the diff. */
  .commits-column {
    position: sticky;
    top: 0;
    align-self: start;
    max-height: calc(100vh - 1rem);
    min-width: 0;
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
  .diff-scroll {
    border-radius: var(--border-radius-md);
  }
  @media (max-width: 60rem) {
    .review-layout {
      grid-template-columns: 1fr;
    }
    .commits-column {
      position: static;
      max-height: none;
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
    font: var(--txt-body-m-regular);
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
  }
  .commit {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    padding: 0.5rem 1rem;
  }
  .commit + .commit {
    border-top: 1px solid var(--color-border-subtle);
  }
  /* The last row sits flush against the container's bottom edge, so its hover
     and selected fill has to follow the rounded corners itself rather than
     relying on the scroll container to clip it. */
  .commit:last-child {
    border-bottom-left-radius: var(--border-radius-md);
    border-bottom-right-radius: var(--border-radius-md);
  }
  .commit > :global(.teaser) {
    flex: 1;
    min-width: 0;
  }
  .commit:hover {
    background-color: var(--color-surface-subtle);
  }
  /* Overlay the close button so showing it on selection doesn't shift the
     commit content, straddling the commit list's right edge as an outline
     button. */
  .commit-close {
    position: absolute;
    top: 0;
    right: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 1px solid var(--color-border-default);
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-canvas);
    color: var(--color-text-secondary);
    cursor: pointer;
  }
  .commit-close:hover,
  .commit-close:focus-visible {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .commit.active {
    background-color: var(--color-surface-subtle);
  }
  .summary {
    padding: 0.25rem 0;
  }
</style>

<div>
  {#if showRevisionDescription || canEditDescription}
    {#if editingDescription}
      <div class="revision-description">
        <ExtendedTextarea
          {rid}
          body={showRevisionDescription ? revisionDescription : ""}
          focus
          submitCaption="Save"
          submit={async ({ comment, embeds }) => {
            await onSaveDescription?.(comment, Array.from(embeds.values()));
            editingDescription = false;
          }}
          close={() => (editingDescription = false)} />
      </div>
    {:else}
      <div class="revision-description" class:collapsed={descriptionCollapsed}>
        <div
          class="revision-description-body txt-body-m-regular"
          bind:this={descriptionEl}>
          {#if showRevisionDescription}
            <Markdown {rid} breaks content={revisionDescription} />
          {:else}
            <span style:color="var(--color-text-tertiary)">No description</span>
          {/if}
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
        {#if canEditDescription}
          <div class="revision-description-actions">
            <button
              type="button"
              class="revision-description-edit"
              title="Edit description"
              onclick={() => (editingDescription = true)}>
              <Icon name="edit" />
            </button>
          </div>
        {/if}
      </div>
    {/if}
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
                if (active) {
                  // Keep the sole commit selected; there's nothing else to show.
                  if (commitList.length === 1) return;
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
                void scrollToDiff();
              }}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="commit" class:active onclick={toggle}>
                <CobCommitTeaser
                  stacked
                  hoverable
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
                {#if active && commitList.length > 1}
                  <span class="commit-close" title="Show all changes">
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
      <div class="diff-scroll" bind:this={diffScrollEl}>
        {#if selectedCommitData}
          <div class="selected-commit-message txt-selectable">
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
            bind:this={changeset}
            expanded={filesExpanded}
            {head}
            {diff}
            {rid}
            codeComments={diffCodeComments}
            draftReviewId={isRevisionDiff ? draftReviewId : undefined} />
        {/await}
      </div>
    </div>
  </div>
</div>
