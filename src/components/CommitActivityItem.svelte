<script lang="ts">
  import type { FileDiff as FileDiffType } from "@bindings/diff/FileDiff";
  import type { Commit } from "@bindings/repo/Commit";

  import { cachedGetDiff } from "@app/lib/invoke";
  import { absoluteTimestamp, formatTimestamp } from "@app/lib/utils";

  import FileDiff from "@app/components/FileDiff.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";

  interface Props {
    commit: Commit;
    rid: string;
  }

  const { commit, rid }: Props = $props();

  let expanded = $state(false);

  const parent = $derived(commit.parents[0]);

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
    <Icon name={expanded ? "chevron-down" : "commit"} />
  </div>
  <div class="wrapper">
    <span class="author">{commit.author.name}</span>
    <div class="summary-line">
      committed <Id id={commit.id} clipboard={commit.id} /> — {commit.summary}
    </div>
    <div
      class="timestamp"
      title={absoluteTimestamp(commit.committer.time * 1000)}>
      {formatTimestamp(commit.committer.time * 1000)}
    </div>
  </div>
</div>

{#if expanded}
  {#if !parent}
    <div class="fallback txt-body-m-regular">
      Initial commit; no diff to show.
    </div>
  {:else}
    {#await cachedGetDiff( rid, { base: parent, head: commit.id, unified: 3, highlight: true }, )}
      <div class="fallback txt-body-m-regular">Loading diff…</div>
    {:then diff}
      <div class="diff">
        {#each diff.files as file (fileKey(file))}
          <FileDiff {file} head={commit.id} expanded />
        {/each}
      </div>
    {:catch error}
      <div class="fallback txt-body-m-regular">
        Failed to load diff: {error.message ?? error}
      </div>
    {/await}
  {/if}
{/if}
