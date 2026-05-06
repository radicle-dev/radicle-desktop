<script lang="ts">
  import type { FileDiff as FileDiffType } from "@bindings/diff/FileDiff";
  import type { Commit } from "@bindings/repo/Commit";

  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { cachedGetDiff } from "@app/lib/invoke";
  import { absoluteTimestamp, formatTimestamp } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import FileDiff from "@app/components/FileDiff.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";

  interface Props {
    commit: Commit;
    rid: string;
    draftReviewId?: string;
  }

  const { commit, rid, draftReviewId }: Props = $props();

  let expanded = $state(false);
  let filesExpanded = $state(true);

  const parent = $derived(commit.parents[0]);
  const fullMessage = $derived(commit.message.trim());
  const checked = $derived(
    draftReviewId
      ? draftReviewStorage.isCommitChecked(draftReviewId, commit.id)
      : false,
  );

  function toggle() {
    expanded = !expanded;
  }

  function fileKey(file: FileDiffType): string {
    if (file.status === "moved" || file.status === "copied") {
      return `${file.oldPath}->${file.newPath}`;
    }
    return file.path;
  }
</script>

<style>
  .timeline-item {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    min-width: 0;
    cursor: pointer;
  }
  .wrapper {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    min-width: 0;
    flex: 1 1 0;
  }
  .icon {
    padding-top: 0.1875rem;
    color: var(--color-text-secondary);
  }
  .icon-hover {
    display: none;
  }
  .timeline-item:hover .icon-default,
  .timeline-item:focus-visible .icon-default {
    display: none;
  }
  .timeline-item:hover .icon-hover,
  .timeline-item:focus-visible .icon-hover {
    display: inline;
  }
  .author {
    color: var(--color-text-primary);
  }
  .summary-line {
    flex: 1 1 0;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .full-message {
    margin: 0.25rem 0 0 2rem;
    white-space: pre-wrap;
    color: var(--color-text-secondary);
  }
  .diff-toolbar {
    margin: 0.5rem 0 0 2rem;
    display: flex;
    justify-content: flex-end;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
  .diff {
    margin: 0.5rem 0 0 2rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .fallback {
    margin: 0.5rem 0 0 2rem;
    color: var(--color-text-secondary);
  }
</style>

<div
  class="timeline-item txt-body-m-regular"
  role="button"
  tabindex="0"
  onclick={toggle}
  onkeydown={e => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      toggle();
    }
  }}>
  <div class="icon">
    {#if expanded}
      <Icon name="chevron-down" />
    {:else}
      <span class="icon-default"><Icon name="commit" /></span>
      <span class="icon-hover"><Icon name="chevron-right" /></span>
    {/if}
  </div>
  <div class="wrapper">
    <span class="author">{commit.author.name}</span>
    {#if !expanded}
      <div class="summary-line">
        committed <Id id={commit.id} clipboard={commit.id} /> — {commit.summary}
      </div>
    {:else}
      <div class="summary-line">
        committed <Id id={commit.id} clipboard={commit.id} />
      </div>
    {/if}
    <div
      class="timestamp"
      title={absoluteTimestamp(commit.committer.time * 1000)}>
      {formatTimestamp(commit.committer.time * 1000)}
    </div>
    {#if draftReviewId}
      <Button
        variant={checked ? "ghost" : "naked"}
        active={checked}
        onclick={e => {
          e.stopPropagation();
          draftReviewStorage.toggleCheckedCommit(draftReviewId, commit.id);
        }}
        title={checked
          ? "Mark commit as not reviewed"
          : "Mark commit as reviewed"}>
        <Icon name={checked ? "checkmark" : "eye"} />
        Checked
      </Button>
    {/if}
  </div>
</div>

{#if expanded}
  {#if fullMessage}
    <div class="full-message txt-body-m-regular">{fullMessage}</div>
  {/if}
  {#if !parent}
    <div class="fallback txt-body-m-regular">
      Initial commit; no diff to show.
    </div>
  {:else}
    <div class="diff-toolbar txt-body-m-regular">
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
    </div>
    {#await cachedGetDiff( rid, { base: parent, head: commit.id, unified: 3, highlight: true }, )}
      <div class="fallback txt-body-m-regular">Loading diff…</div>
    {:then diff}
      <div class="diff">
        {#each diff.files as file (fileKey(file))}
          <FileDiff
            {file}
            head={commit.id}
            expanded={filesExpanded}
            {draftReviewId} />
        {/each}
      </div>
    {:catch error}
      <div class="fallback txt-body-m-regular">
        Failed to load diff: {error.message ?? error}
      </div>
    {/await}
  {/if}
{/if}
