<script lang="ts">
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { CodeComments } from "./Diff.svelte";

  import Diff from "@app/components/Diff.svelte";
  import File from "@app/components/File.svelte";
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
</script>

<style>
  .added {
    color: var(--color-foreground-success);
    background-color: var(--color-fill-diff-green-light);
  }
  .deleted {
    color: var(--color-foreground-red);
    background-color: var(--color-fill-diff-red-light);
  }
  .moved,
  .copied {
    color: var(--color-foreground-dim);
    background: var(--color-fill-ghost);
  }
  .stats {
    font-size: var(--font-size-tiny);
    font-family: var(--font-family-monospace);
    font-weight: var(--font-weight-semibold);
  }
</style>

{#snippet emptyPlaceholder()}
  <div class="global-flex" style:margin="1rem 0" style:justify-content="center">
    <Icon name="none" />Empty file
  </div>
{/snippet}

<File {expanded}>
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
      <span class="global-counter added">added</span>
    {:else if file.status === "deleted"}
      <span class="global-counter deleted">deleted</span>
    {:else if file.status === "moved"}
      <span class="global-counter moved">moved</span>
    {:else if file.status === "copied"}
      <span class="global-counter copied">copied</span>
    {/if}
  {/snippet}

  {#snippet rightHeader()}
    {#if file.diff.type === "plain" && file.diff.hunks.length > 0}
      <div class="stats">
        <span style:color="var(--color-foreground-success)">
          +{file.diff.stats.additions}
        </span>
        <span style:color="var(--color-foreground-red)">
          -{file.diff.stats.deletions}
        </span>
      </div>
    {/if}
    {#if commentsOfThisFile && commentsOfThisFile.threads.length > 0}
      <Icon name="comment" />
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
</File>
