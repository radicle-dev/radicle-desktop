<script lang="ts">
  import type { CodeComments } from "./Diff.svelte";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import { cachedGetDiff, cachedListCommits } from "@app/lib/invoke";
  import { pluralize } from "@app/lib/utils";

  import Changeset from "@app/components/Changeset.svelte";
  import CobCommitTeaser from "./CobCommitTeaser.svelte";
  import CommitsContainer from "@app/components/CommitsContainer.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "./Id.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  interface Props {
    patchId: string;
    revision: Revision;
    rid: string;
    codeComments?: CodeComments;
  }

  const { patchId, revision, rid, codeComments }: Props = $props();

  let hideChanges = $state(false);
  let filesExpanded = $state(true);
  let selectedCommit = $state<string>();
  let base = $state(revision.base);
  let head = $state(revision.head);

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    hideChanges = false;
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
    font-size: 0.875rem;
    margin-left: 0.5rem;
    gap: 0.5rem;
    padding: 1rem 0.5rem 0.5rem 1rem;
    border-left: 1px solid var(--color-fill-separator);
  }
  .commit:last-of-type::after {
    content: "";
    position: absolute;
    left: -18.5px;
    top: 14px;
    bottom: -0.5rem;
    border-left: 4px solid var(--color-background-default);
  }
  .commit-dot {
    width: 4px;
    height: 4px;
    position: absolute;
    top: 0.625rem;
    left: -18.5px;
    background-color: var(--color-fill-separator);
  }
  .commit {
    cursor: pointer;
  }
  .commit-dot.active {
    background-color: var(--color-border-focus);
  }
  .commit:hover:not(.single-commit) .commit-dot:not(.active) {
    background-color: var(--color-foreground-contrast);
  }
  .commit:hover:not(.single-commit) {
    background-color: var(--color-background-float);
  }
  .disabled {
    color: var(--color-foreground-disabled) !important;
  }
  .summary {
    cursor: pointer;
    padding: 0.25rem 0;
  }
  .summary:hover:not(.single-commit) {
    background-color: var(--color-background-float);
    color: var(--color-foreground-contrast) !important;
  }
  .single-commit {
    cursor: default !important;
  }
</style>

<div
  class="txt-semibold global-flex"
  style:margin-bottom={hideChanges ? undefined : "1rem"}>
  <NakedButton variant="ghost" onclick={() => (hideChanges = !hideChanges)}>
    <Icon name={hideChanges ? "chevron-right" : "chevron-down"} />
    <div class="txt-semibold global-flex txt-regular">Changes</div>
  </NakedButton>
  {#if !hideChanges}
    <div style:margin-left="auto">
      <NakedButton
        variant="ghost"
        onclick={() => (filesExpanded = !filesExpanded)}>
        {#if filesExpanded === true}
          <Icon name="collapse" />
          Collapse all
        {:else}
          <Icon name="expand" />
          Expand all
        {/if}
      </NakedButton>
    </div>
  {/if}
</div>

<div style:display={hideChanges ? "none" : "revert"}>
  {#await cachedListCommits(rid, revision.base, revision.head) then commits}
    <div style:margin-bottom="1rem">
      <CommitsContainer>
        {#snippet leftHeader()}
          <div class="txt-semibold">Commits</div>
        {/snippet}
        <div style:padding="0 1rem">
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="global-flex txt-small summary"
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
            <Id
              id={revision.base}
              variant={selectedCommit ? "none" : "commit"} />
            <div class="global-counter">base</div>
          </div>
          <div class="commits">
            {#each commits.reverse() as commit}
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
                  {commit} />
              </div>
            {/each}
          </div>
        </div>
      </CommitsContainer>
    </div>
  {/await}

  {#await cachedGetDiff(rid, { base, head, unified: 3, highlight: true })}
    <span class="txt-small">Loading…</span>
  {:then diff}
    <Changeset expanded={filesExpanded} {head} {diff} {codeComments} />
  {/await}
</div>
