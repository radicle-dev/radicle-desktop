<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import { cachedGetDiff, cachedListCommits } from "@app/lib/invoke";
  import { pluralize } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Changeset from "@app/components/Changeset.svelte";
  import CobCommitTeaser from "@app/components/CobCommitTeaser.svelte";
  import CommitsContainer from "@app/components/CommitsContainer.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import JobCob from "@app/components/JobCob.svelte";

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
    base = revision.base;
    head = revision.head;
  });

  function selectRevision({
    headId,
    baseId,
    commitId = undefined,
    showFiles = true,
  }: {
    headId: string;
    baseId: string;
    commitId?: string;
    showFiles?: boolean;
  }) {
    head = headId;
    base = baseId;
    selectedCommit = commitId;
    filesExpanded = showFiles;
  }

  const isActiveCommit = (commitId: string) => selectedCommit === commitId;
  const isTeaserDisabled = (commitId: string) =>
    selectedCommit ? selectedCommit !== commitId : false;
</script>

<style>
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
  .commits {
    position: relative;
    display: flex;
    flex-direction: column;
    font: var(--txt-body-m-regular);
    gap: 0.5rem;
    padding: 1rem 0 0.5rem 0;
  }
  .commits::before {
    content: "";
    position: absolute;
    top: 1rem;
    bottom: 0.5rem;
    left: 7.5px;
    width: 1px;
    background-color: var(--color-border-subtle);
    pointer-events: none;
  }
  .commit {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    padding: 0.125rem 0.5rem;
    margin: 0 -0.5rem;
    border-radius: var(--border-radius-sm);
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
  .commit-marker {
    flex-shrink: 0;
    width: 1rem;
    height: 1rem;
    color: var(--color-text-secondary);
    cursor: pointer;
    position: relative;
  }
  .commit-marker-state {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .commit-marker-hover {
    display: none;
  }
  .commit:hover:not(.single-commit) .commit-marker-default {
    display: none;
  }
  .commit:hover:not(.single-commit) .commit-marker-hover {
    display: flex;
  }
  .commit-marker-dot {
    position: absolute;
    top: 6px;
    left: 6px;
    width: 4px;
    height: 4px;
    background-color: var(--color-border-subtle);
  }
  .commit.active .commit-marker {
    color: var(--color-text-primary);
  }
  .summary {
    padding: 0.25rem 0;
  }
  .single-commit {
    cursor: default !important;
  }
</style>

<div>
  {#await cachedListCommits(rid, revision.base, revision.head) then commits}
    <div style:margin-bottom="1rem">
      <CommitsContainer>
        {#snippet leftHeader()}
          <div class="global-flex txt-body-m-regular summary">
            {commits.length}
            {pluralize("commit", commits.length)} on base
            <Id id={revision.base} clipboard={revision.base} />
            <div class="global-chip">Base</div>
          </div>
        {/snippet}
        <div style:padding="0 1rem">
          <div class="commits">
            {#each [...commits].reverse() as commit, idx}
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
                  });
                }
              }}
              <div
                class="commit"
                class:single-commit={commits.length === 1}
                class:active
                style:position="relative">
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="commit-marker" onclick={toggle}>
                  {#if active}
                    <div class="commit-marker-state">
                      <Icon name="close" />
                    </div>
                  {:else}
                    <div class="commit-marker-state commit-marker-default">
                      <div class="commit-marker-dot"></div>
                    </div>
                    <div class="commit-marker-state commit-marker-hover">
                      <Icon name="eye" />
                    </div>
                  {/if}
                </div>
                <CobCommitTeaser
                  hoverable={commits.length > 1}
                  disabled={isTeaserDisabled(commit.id)}
                  onclick={toggle}
                  {commit}>
                  {#if idx === commits.length - 1}
                    <JobCob {rid} commit={commit.id} />
                  {/if}
                </CobCommitTeaser>
              </div>
            {/each}
          </div>
        </div>
      </CommitsContainer>
    </div>
  {/await}

  {#await cachedGetDiff(rid, { base, head, unified: 3, highlight: true })}
    <span class="txt-body-m-regular">Loading…</span>
  {:then diff}
    <div class="stats-row txt-body-m-regular">
      <div class="stats" style:color="var(--color-text-secondary)">
        {diff.stats.filesChanged}
        {pluralize("file", diff.stats.filesChanged)} modified with
        <span style:color="var(--color-feedback-success-text)">
          {diff.stats.insertions}
          {pluralize("insertion", diff.stats.insertions)}
        </span>
        and
        <span style:color="var(--color-feedback-error-text)">
          {diff.stats.deletions}
          {pluralize("deletion", diff.stats.deletions)}
        </span>
      </div>
      {#if diff.stats.filesChanged > 0}
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
    <Changeset
      expanded={filesExpanded}
      {head}
      {diff}
      {codeComments}
      {draftReviewId} />
  {/await}
</div>
