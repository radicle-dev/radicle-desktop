<script lang="ts">
  import type { DiffFileHeaderState } from "./diffFileHeaderState.svelte";

  import Button from "@app/components/Button.svelte";
  import DiffActions from "@app/components/DiffActions.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Path from "@app/components/Path.svelte";

  interface Props {
    state: DiffFileHeaderState;
  }

  const { state }: Props = $props();

  const name = $derived(state.fileDiff?.name ?? "");
  const prevName = $derived(state.fileDiff?.prevName);
  const isRename = $derived(
    state.status === "moved" || state.status === "copied",
  );

  const stats = $derived.by(() => {
    let additions = 0;
    let deletions = 0;
    for (const hunk of state.fileDiff?.hunks ?? []) {
      additions += hunk.additionLines;
      deletions += hunk.deletionLines;
    }
    return { additions, deletions };
  });

  const fileName = $derived(`${name.split("/").pop() || "file"}.diff`);
  const text = $derived(state.text);

  const statusLabel = $derived(
    state.status === "added"
      ? "Added"
      : state.status === "deleted"
        ? "Deleted"
        : state.status === "moved"
          ? "Moved"
          : state.status === "copied"
            ? "Copied"
            : undefined,
  );
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    min-width: 0;
    min-height: 2.5rem;
    padding: 0 0.5rem;
    font: var(--txt-body-m-regular);
  }
  /* Binary/empty marker matches the collapse Button's footprint so filenames
     stay column-aligned across all files. */
  .marker {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 1.75rem;
    height: 1.75rem;
    line-height: 0;
    color: var(--color-text-secondary);
  }
  .marker :global(svg) {
    display: block;
  }
  .arrow {
    color: var(--color-text-secondary);
  }
  .status {
    padding: 0.125rem 0.5rem;
    border-radius: var(--border-radius-sm);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .status.added {
    color: var(--color-feedback-success-text);
    background-color: var(--color-feedback-success-bg);
  }
  .status.deleted {
    color: var(--color-feedback-error-text);
    background-color: var(--color-feedback-error-bg);
  }
  .status.moved,
  .status.copied {
    color: var(--color-text-secondary);
    background: var(--color-surface-subtle);
  }
  .stats {
    margin-left: auto;
    font: var(--txt-code-regular);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .add {
    color: var(--color-feedback-success-text);
  }
  .del {
    color: var(--color-feedback-error-text);
  }
</style>

<div class="header">
  {#if state.note === "binary"}
    <span class="marker" title="Binary file"><Icon name="binary" /></span>
  {:else if state.note === "empty"}
    <span class="marker" title="Empty file"><Icon name="none" /></span>
  {:else}
    <Button
      variant="naked"
      styleHeight="1.75rem"
      styleWidth="1.75rem"
      stylePadding="0"
      styleJustifyContent="center"
      title={state.collapsed ? "Expand file" : "Collapse file"}
      onclick={() => state.onToggleCollapse()}>
      <Icon name={state.collapsed ? "chevron-right" : "chevron-down"} />
    </Button>
  {/if}

  {#if isRename && prevName}
    <Path fullPath={prevName} />
    <span class="arrow">→</span>
    <Path fullPath={name} />
  {:else}
    <Path fullPath={name} />
  {/if}

  {#if statusLabel}
    <span class="status {state.status}">{statusLabel}</span>
  {/if}

  <span class="stats">
    {#if !state.note && stats.additions + stats.deletions > 0}
      <span class="add">+{stats.additions}</span>
      <span class="del">-{stats.deletions}</span>
    {/if}
  </span>

  {#if text !== undefined}
    <DiffActions {text} {fileName} title="File diff actions" />
  {/if}
</div>
