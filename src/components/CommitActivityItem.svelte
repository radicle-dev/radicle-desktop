<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Diff } from "@bindings/diff/Diff";
  import type { FileDiff as FileDiffType } from "@bindings/diff/FileDiff";
  import type { Commit } from "@bindings/repo/Commit";

  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { cachedGetDiff } from "@app/lib/invoke";
  import {
    absoluteTimestamp,
    formatTimestamp,
    pluralize,
  } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import FileDiff from "@app/components/FileDiff.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";

  interface Props {
    commit: Commit;
    rid: string;
    draftReviewId?: string;
    hideAuthor?: boolean;
    codeComments?: CodeComments;
  }

  const { commit, rid, draftReviewId, hideAuthor, codeComments }: Props =
    $props();

  let expanded = $state(false);
  let hasEverExpanded = $state(false);
  let filesExpanded = $state(true);
  let commitDiff: Diff | undefined = $state();

  $effect(() => {
    if (expanded) hasEverExpanded = true;
  });

  const parent = $derived(commit.parents[0]);
  const fullMessage = $derived(commit.message.trim());
  const expandedBody = $derived(
    fullMessage === commit.summary ? undefined : fullMessage,
  );
  const authoredAt = $derived(commit.author.time * 1000);

  function filePathOf(file: FileDiffType): string {
    return file.status === "moved" || file.status === "copied"
      ? file.newPath
      : file.path;
  }

  $effect(() => {
    if (!parent) return;
    let cancelled = false;
    void cachedGetDiff(rid, {
      base: parent,
      head: commit.id,
      unified: 0,
      highlight: false,
    }).then(diff => {
      if (cancelled) return;
      commitDiff = diff;
    });
    return () => {
      cancelled = true;
    };
  });

  const totalFiles = $derived(commitDiff?.files.length ?? 0);
  const reviewedFiles = $derived.by(() => {
    if (!draftReviewId || !commitDiff) return 0;
    return commitDiff.files.filter(f =>
      draftReviewStorage.isFileChecked(draftReviewId, filePathOf(f)),
    ).length;
  });
  const commitCodeComments = $derived(
    codeComments
      ? {
          ...codeComments,
          threads: codeComments.threads.filter(
            thread => thread.root.location?.commit === commit.id,
          ),
        }
      : undefined,
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
    display: grid;
    grid-template-columns: 1.25rem minmax(0, 1fr);
    column-gap: 0.5rem;
    align-items: flex-start;
    min-width: 0;
    cursor: pointer;
    padding: 0.125rem 0.25rem;
    margin: 0 -0.25rem;
    border-radius: var(--border-radius-sm);
  }
  .timeline-item:hover,
  .timeline-item:focus-visible {
    background-color: var(--color-surface-subtle);
  }
  .wrapper {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    min-width: 0;
    flex: 1 1 0;
  }
  .icon {
    width: 1.25rem;
    padding-top: 0.1875rem;
    color: var(--color-text-secondary);
    display: flex;
    justify-content: center;
  }
  .icon-stack {
    display: grid;
    width: 1rem;
    place-items: center;
  }
  .icon-default,
  .icon-hover {
    grid-area: 1 / 1;
    transition:
      opacity 150ms ease,
      transform 150ms ease;
  }
  .icon-hover {
    opacity: 0;
    transform: rotate(-90deg);
  }
  .timeline-item:hover .icon-default,
  .timeline-item:focus-visible .icon-default {
    opacity: 0;
    transform: rotate(90deg);
  }
  .timeline-item:hover .icon-hover,
  .timeline-item:focus-visible .icon-hover {
    opacity: 1;
    transform: rotate(0);
  }
  .author {
    color: var(--color-text-tertiary);
  }
  .summary-line {
    flex: 1 1 0;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .action-verb {
    color: var(--color-text-tertiary);
  }
  .summary-content {
    color: var(--color-text-primary);
  }
  .full-message {
    margin: 1rem 0 0;
    white-space: pre-wrap;
    color: var(--color-text-secondary);
  }
  .expanded-header {
    margin: 1rem 0 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 0.75rem;
    align-items: start;
  }
  .expanded-header .full-message {
    margin: 0 0 1rem;
    padding-top: 0.375rem;
  }
  .diff-summary {
    margin-top: 1rem;
    color: var(--color-text-secondary);
  }
  .expanded-header .diff-summary {
    margin-top: 0;
  }
  .diff-toolbar {
    margin: 0;
    display: flex;
    justify-content: flex-end;
  }
  .timestamp {
    color: var(--color-text-quaternary);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }
  .meta-hash {
    color: var(--color-text-quaternary);
  }
  .meta-hash :global(.txt-id) {
    color: inherit;
  }
  .meta-hash :global(.txt-id:hover) {
    color: inherit;
  }
  .reviewed-count {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--color-text-quaternary);
    flex-shrink: 0;
  }
  .diff-skeleton {
    margin: 1rem 0 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .skeleton-file {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding: 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
  }
  .skeleton-row {
    height: 0.75rem;
    background-color: var(--color-surface-subtle);
    border-radius: var(--border-radius-sm);
    animation: skeleton-pulse 1.4s ease-in-out infinite;
  }
  .skeleton-row-summary {
    width: 60%;
  }
  .skeleton-row-header {
    width: 40%;
    height: 1rem;
    margin-bottom: 0.25rem;
  }
  .skeleton-row-hunk {
    width: 100%;
  }
  @keyframes skeleton-pulse {
    0%,
    100% {
      opacity: 0.5;
    }
    50% {
      opacity: 0.9;
    }
  }
  .diff {
    margin: 1rem 0 0;
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .fallback {
    margin: 1rem 0 0;
    color: var(--color-text-secondary);
  }
  .collapsible {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows 180ms ease-out;
  }
  .collapsible.open {
    grid-template-rows: 1fr;
  }
  .collapsible-inner {
    overflow: hidden;
    min-height: 0;
  }
</style>

<div class="commit-entry">
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
      <span class="icon-stack">
        <span class="icon-default"><Icon name="commit" /></span>
        <span class="icon-hover">
          <Icon name={expanded ? "collapse-vertical" : "expand-vertical"} />
        </span>
      </span>
    </div>
    <div class="wrapper">
      {#if !hideAuthor}
        <span class="author">{commit.author.name}</span>
      {/if}
      <div class="summary-line">
        {#if !hideAuthor}
          <span class="action-verb">committed</span>
        {/if}
        <span class="txt-body-m-medium summary-content">{commit.summary}</span>
      </div>
      <div class="meta">
        <div class="meta-hash">
          <Id id={commit.id} clipboard={commit.id} />
        </div>
        <div class="timestamp" title={absoluteTimestamp(authoredAt)}>
          {formatTimestamp(authoredAt)}
        </div>
      </div>
      {#if draftReviewId && totalFiles > 0}
        <span
          class="reviewed-count txt-body-s-regular"
          title="{reviewedFiles} of {totalFiles} {pluralize(
            'file',
            totalFiles,
          )} reviewed in this commit">
          <Icon name="eye" />
          {reviewedFiles}/{totalFiles}
        </span>
      {/if}
    </div>
  </div>

  <div class="collapsible" class:open={expanded}>
    <div class="collapsible-inner">
      {#if hasEverExpanded}
        {#if !parent}
          {#if expandedBody}
            <div class="full-message txt-body-m-regular">{expandedBody}</div>
          {/if}
          <div class="fallback txt-body-m-regular">
            Initial commit; no diff to show.
          </div>
        {:else}
          {#await cachedGetDiff( rid, { base: parent, head: commit.id, unified: 3, highlight: true }, )}
            <div class="diff-skeleton" aria-label="Loading diff">
              <div class="skeleton-row skeleton-row-summary"></div>
              <div class="skeleton-file">
                <div class="skeleton-row skeleton-row-header"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
              </div>
              <div class="skeleton-file">
                <div class="skeleton-row skeleton-row-header"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
                <div class="skeleton-row skeleton-row-hunk"></div>
              </div>
            </div>
          {:then diff}
            {#if expandedBody || diff.files.length > 1}
              <div class="expanded-header">
                <div>
                  {#if expandedBody}
                    <div class="full-message txt-body-m-regular">
                      {expandedBody}
                    </div>
                  {/if}
                  <div class="diff-summary txt-body-m-regular">
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
                </div>
                {#if diff.files.length > 1}
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
                {/if}
              </div>
            {:else}
              <div class="diff-summary txt-body-m-regular">
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
            {/if}
            <div class="diff">
              {#each diff.files as file (fileKey(file))}
                <FileDiff
                  {file}
                  head={commit.id}
                  expanded={filesExpanded}
                  codeComments={commitCodeComments}
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
    </div>
  </div>
</div>
