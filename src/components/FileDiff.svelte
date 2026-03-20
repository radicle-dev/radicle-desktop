<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { FileDiff } from "@bindings/diff/FileDiff";

  import Diff from "@app/components/Diff.svelte";
  import FileBlock from "@app/components/FileBlock.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Path from "@app/components/Path.svelte";

  interface Props {
    expanded: boolean;
    file: FileDiff;
    head: string;
    codeComments?: CodeComments;
  }

  const { expanded, file, head, codeComments }: Props = $props();

  // Pass down only the comments that apply to the given diff.
  function filterThreadsByFilePath() {
    if (codeComments) {
      if (
        file.status === "added" ||
        file.status === "deleted" ||
        file.status === "modified"
      ) {
        return {
          ...codeComments,
          threads:
            codeComments.threads.filter(t => {
              return t.root.location?.path === file.path;
            }) ?? [],
        };
      } else {
        return { ...codeComments, threads: [] };
      }
    } else {
      return undefined;
    }
  }

  const commentsOfThisFile = $derived(filterThreadsByFilePath());
  const resolvedThreads = $derived(
    commentsOfThisFile
      ? commentsOfThisFile.threads.filter(t => {
          return t.root.resolved === true;
        }).length
      : 0,
  );
  const unresolvedThreads = $derived(
    commentsOfThisFile
      ? commentsOfThisFile.threads.filter(t => {
          return t.root.resolved === false;
        }).length
      : 0,
  );
</script>

<style>
  .added {
    color: var(--color-feedback-success-text);
    background-color: var(--color-feedback-success-bg);
  }
  .deleted {
    color: var(--color-feedback-error-text);
    background-color: var(--color-feedback-error-bg);
  }
  .moved,
  .copied {
    color: var(--color-text-secondary);
    background: var(--color-surface-subtle);
  }
  .stats {
    font: var(--txt-code-regular);
  }
</style>

{#snippet emptyPlaceholder()}
  <div class="global-flex" style:margin="1rem 0" style:justify-content="center">
    Empty file
  </div>
{/snippet}

<FileBlock {expanded}>
  {#snippet leftHeader()}
    {#if file.status === "moved" || file.status === "copied"}
      <span style="display: flex; align-items: center; flex-wrap: wrap;">
        <Path fullPath={file.oldPath} />
        <span style:padding="0 0.5rem">→</span>
        <Path fullPath={file.newPath} />
      </span>
    {:else}
      <Path fullPath={file.path} />
    {/if}

    {#if file.status === "added"}
      <span class="global-chip added">Added</span>
    {:else if file.status === "deleted"}
      <span class="global-chip deleted">Deleted</span>
    {:else if file.status === "moved"}
      <span class="global-chip moved">Moved</span>
    {:else if file.status === "copied"}
      <span class="global-chip copied">Copied</span>
    {/if}
  {/snippet}

  {#snippet rightHeader()}
    {#if file.diff.type === "plain" && file.diff.hunks.length > 0}
      <div class="stats">
        <span style:color="var(--color-feedback-success-text)">
          +{file.diff.stats.additions}
        </span>
        <span style:color="var(--color-feedback-error-text)">
          -{file.diff.stats.deletions}
        </span>
      </div>
    {/if}
    {#if commentsOfThisFile && commentsOfThisFile.threads.length > 0}
      {#if unresolvedThreads > 0}
        <div class="global-flex">
          <Icon name="comment-cross" />
          {unresolvedThreads}
        </div>
      {/if}
      {#if resolvedThreads > 0}
        <div class="global-flex">
          <Icon name="comment-checkmark" />
          {resolvedThreads}
        </div>
      {/if}
    {/if}
  {/snippet}

  {#if file.diff.type === "plain"}
    {#if file.diff.hunks.length > 0}
      <Diff {file} {head} codeComments={commentsOfThisFile} />
    {:else}
      {@render emptyPlaceholder()}
    {/if}
  {:else if file.diff.type === "binary"}
    <div
      class="global-flex"
      style:margin="1rem 0"
      style:justify-content="center">
      <Icon name="binary" />Binary file
    </div>
  {:else}
    {@render emptyPlaceholder()}
  {/if}
</FileBlock>
