<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Embed } from "@bindings/cob/Embed";
  import type { FileDiff } from "@bindings/diff/FileDiff";

  import { Buffer } from "buffer";

  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { invoke } from "@app/lib/invoke";

  import Button from "@app/components/Button.svelte";
  import Diff from "@app/components/Diff.svelte";
  import FileBlock from "@app/components/FileBlock.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Path from "@app/components/Path.svelte";

  interface Props {
    expanded: boolean;
    expandable?: boolean;
    file: FileDiff;
    head: string;
    rid: string;
    codeComments?: CodeComments;
    draftReviewId?: string;
  }

  const {
    expanded,
    expandable = true,
    file,
    head,
    rid,
    codeComments,
    draftReviewId,
  }: Props = $props();

  function binaryBlobOid(f: FileDiff): string {
    return f.status === "deleted" ? f.old.oid : f.new.oid;
  }
  function displayPath(f: FileDiff): string {
    return f.status === "moved" || f.status === "copied" ? f.newPath : f.path;
  }
  async function loadBinaryEmbed(f: FileDiff): Promise<Embed> {
    return invoke<Embed>("get_embed", {
      rid,
      name: displayPath(f),
      oid: binaryBlobOid(f),
    });
  }
  function blobUrl(content: Array<number>): string {
    return URL.createObjectURL(new Blob([Buffer.from(content)]));
  }

  let expandedState = $derived(expanded);

  const filePathKey = $derived(
    file.status === "moved" || file.status === "copied"
      ? file.newPath
      : file.path,
  );
  const checked = $derived(
    draftReviewId
      ? draftReviewStorage.isFileChecked(draftReviewId, filePathKey)
      : false,
  );

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
  .binary-image {
    max-width: 100%;
    max-height: 32rem;
    object-fit: contain;
    display: block;
  }
</style>

{#snippet emptyPlaceholder()}
  <div class="global-flex" style:margin="1rem 0" style:justify-content="center">
    Empty file
  </div>
{/snippet}

<FileBlock bind:expanded={expandedState} {expandable}>
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
    {#if draftReviewId}
      <Button
        variant="ghost"
        active={checked}
        bordered
        styleHeight="1.5rem"
        stylePadding="0 0.375rem"
        onclick={e => {
          e.stopPropagation();
          const wasChecked = checked;
          draftReviewStorage.toggleCheckedFile(draftReviewId, filePathKey);
          if (!wasChecked) expandedState = false;
        }}
        title={checked ? "Mark file as not reviewed" : "Mark file as reviewed"}>
        <Icon name={checked ? "checkmark" : "eye"} />
        {checked ? "Reviewed" : "Review"}
      </Button>
    {/if}
  {/snippet}

  {#if file.diff.type === "plain"}
    {#if file.diff.hunks.length > 0}
      <Diff {file} {head} codeComments={commentsOfThisFile} />
    {:else}
      {@render emptyPlaceholder()}
    {/if}
  {:else if file.diff.type === "binary"}
    {#await loadBinaryEmbed(file)}
      <div
        class="global-flex"
        style:margin="1rem 0"
        style:justify-content="center"
        style:color="var(--color-text-tertiary)">
        Loading binary…
      </div>
    {:then embed}
      {#if embed.mimeType?.startsWith("image/")}
        <div
          class="global-flex"
          style:margin="0.5rem"
          style:justify-content="center">
          <img
            class="binary-image"
            src={blobUrl(embed.content)}
            alt={displayPath(file)} />
        </div>
      {:else}
        <div
          class="global-flex"
          style:margin="1rem 0"
          style:justify-content="center">
          <Icon name="binary" />Binary file
        </div>
      {/if}
    {:catch}
      <div
        class="global-flex"
        style:margin="1rem 0"
        style:justify-content="center">
        <Icon name="binary" />Binary file
      </div>
    {/await}
  {:else}
    {@render emptyPlaceholder()}
  {/if}
</FileBlock>
