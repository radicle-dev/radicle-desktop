<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import { cachedGetDiff, cachedListCommits } from "@app/lib/invoke";
  import { pluralize } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Changeset from "@app/components/Changeset.svelte";
  import CobCommitTeaser from "@app/components/CobCommitTeaser.svelte";
  import CommitsContainer from "@app/components/CommitsContainer.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import JobCob from "@app/components/JobCob.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";

  interface Props {
    patchId: string;
    revision: Revision;
    rid: string;
    codeComments?: CodeComments;
    revisions?: Revision[];
    onSelectRevision?: (revision: Revision) => void;
  }

  const {
    patchId,
    revision,
    rid,
    codeComments,
    revisions = [],
    onSelectRevision,
  }: Props = $props();

  let revisionPickerExpanded = $state(false);
  const sortedRevisions = $derived(
    [...revisions].sort((a, b) => a.timestamp - b.timestamp),
  );
  const revisionIndex = $derived(
    sortedRevisions.findIndex(r => r.id === revision.id),
  );

  let filesExpanded = $state(true);
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
  .commits {
    position: relative;
    display: flex;
    flex-direction: column;
    font: var(--txt-body-m-regular);
    margin-left: 0.5rem;
    gap: 0.5rem;
    padding: 1rem 0.5rem 0.5rem 1rem;
    border-left: 1px solid var(--color-border-subtle);
  }
  .commit:last-of-type::after {
    content: "";
    position: absolute;
    left: -18.5px;
    top: 14px;
    bottom: -0.5rem;
    border-left: 4px solid var(--color-surface-canvas);
  }
  .commit-dot {
    width: 0.25rem;
    height: 0.25rem;
    position: absolute;
    top: 0.625rem;
    left: -18.5px;
    background-color: var(--color-border-subtle);
  }
  .commit {
    cursor: pointer;
  }
  .commit-dot.active {
    background-color: var(--color-border-brand);
  }
  .commit:hover:not(.single-commit) .commit-dot:not(.active) {
    background-color: var(--color-text-primary);
  }
  .commit:hover:not(.single-commit) {
    background-color: var(--color-surface-canvas);
  }
  .disabled {
    color: var(--color-text-disabled) !important;
  }
  .summary {
    cursor: pointer;
    padding: 0.25rem 0;
  }
  .summary:hover:not(.single-commit) {
    background-color: var(--color-surface-canvas);
    color: var(--color-text-primary) !important;
  }
  .single-commit {
    cursor: default !important;
  }
</style>

<div
  class="txt-body-m-regular global-flex"
  style:margin-bottom="1rem"
  style:gap="0.5rem">
  {#if sortedRevisions.length > 1 && onSelectRevision}
    <Popover
      popoverPadding="0"
      placement="bottom-start"
      bind:expanded={revisionPickerExpanded}>
      {#snippet toggle(onclick)}
        <Button variant="outline" {onclick} active={revisionPickerExpanded}>
          <Icon name="revision" />
          <span style:color="var(--color-text-secondary)">
            Revision {revisionIndex >= 0 ? revisionIndex + 1 : "?"} of
            {sortedRevisions.length}
          </span>
          <span class="txt-id">{revision.id.substring(0, 7)}</span>
          <Icon name={revisionPickerExpanded ? "chevron-up" : "chevron-down"} />
        </Button>
      {/snippet}
      {#snippet popover()}
        <div
          style:border="1px solid var(--color-border-subtle)"
          style:border-radius="var(--border-radius-sm)"
          style:background-color="var(--color-surface-canvas)">
          <DropdownList items={sortedRevisions}>
            {#snippet item(rev)}
              <DropdownListItem
                selected={rev.id === revision.id}
                styleGap="0.5rem"
                onclick={() => {
                  onSelectRevision?.(rev);
                  closeFocused();
                }}>
                <Icon name="revision" />
                <span class="txt-id">{rev.id.substring(0, 7)}</span>
              </DropdownListItem>
            {/snippet}
          </DropdownList>
        </div>
      {/snippet}
    </Popover>
  {/if}
  <div style:margin-left="auto">
    <Button variant="naked" onclick={() => (filesExpanded = !filesExpanded)}>
      {#if filesExpanded === true}
        <Icon name="collapse-vertical" />
        Collapse all
      {:else}
        <Icon name="expand-vertical" />
        Expand all
      {/if}
    </Button>
  </div>
</div>

<div>
  {#await cachedListCommits(rid, revision.base, revision.head) then commits}
    <div style:margin-bottom="1rem">
      <CommitsContainer>
        {#snippet leftHeader()}
          <div class="txt-body-m-regular">Commits</div>
        {/snippet}
        <div style:padding="0 1rem">
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="global-flex txt-body-m-regular summary"
            class:single-commit={commits.length === 1}
            class:disabled={selectedCommit}
            onclick={() => {
              if (commits.length === 1) return;
              selectRevision({
                headId: revision.head,
                baseId: revision.base,
              });
            }}>
            <Icon name="branch" />
            {commits.length}
            {pluralize("commit", commits.length)} on base
            <Id id={revision.base} clipboard={revision.base} />
            <div class="global-chip">Base</div>
          </div>
          <div class="commits">
            {#each [...commits].reverse() as commit, idx}
              <div
                class="commit"
                class:single-commit={commits.length === 1}
                style:position="relative">
                <div class="commit-dot"></div>
                <div
                  class="commit-dot"
                  class:active={isActiveCommit(commit.id)}>
                </div>
                <CobCommitTeaser
                  hoverable={commits.length > 1}
                  disabled={isTeaserDisabled(commit.id)}
                  onclick={() => {
                    if (commits.length === 1) return;
                    selectRevision({
                      headId: commit.id,
                      baseId: commit.parents[0],
                      commitId: commit.id,
                    });
                  }}
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
    <div
      class="txt-body-m-regular"
      style:color="var(--color-text-secondary)"
      style:margin-bottom="0.5rem">
      {diff.stats.filesChanged}
      {pluralize("file", diff.stats.filesChanged)} modified with
      {diff.stats.insertions}
      {pluralize("insertion", diff.stats.insertions)} and
      {diff.stats.deletions}
      {pluralize("deletion", diff.stats.deletions)}
    </div>
    <Changeset expanded={filesExpanded} {head} {diff} {codeComments} />
  {/await}
</div>
