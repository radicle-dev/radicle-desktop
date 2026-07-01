<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Diff } from "@bindings/diff/Diff";
  import type { Snippet } from "svelte";

  import { SvelteSet } from "svelte/reactivity";

  import type { DiffRow } from "@app/lib/diffRows";
  import { diffRowKey, flattenDiff } from "@app/lib/diffRows";

  import DiffRowView from "@app/components/DiffRow.svelte";
  import FileDiffComponent from "@app/components/FileDiff.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";

  interface Props {
    codeComments?: CodeComments;
    diff: Diff;
    expanded?: boolean;
    head: string;
    // Rendered as the first virtualized row (e.g. commit/patch metadata), so
    // the virtualizer owns the whole scroll content and starts at offset 0.
    header?: Snippet;
  }

  const { codeComments, diff, expanded = true, head, header }: Props = $props();

  // Files the user collapsed (virtualized read-only path only).
  const collapsed = new SvelteSet<number>();
  function isFileExpanded(fileIndex: number): boolean {
    return expanded && !collapsed.has(fileIndex);
  }
  function toggleFile(fileIndex: number) {
    if (collapsed.has(fileIndex)) {
      collapsed.delete(fileIndex);
    } else {
      collapsed.add(fileIndex);
    }
  }

  type Row = { type: "header" } | DiffRow;

  const rows = $derived<Row[]>(
    header
      ? [{ type: "header" }, ...flattenDiff(diff, isFileExpanded)]
      : flattenDiff(diff, isFileExpanded),
  );

  function rowKey(row: Row): string {
    return row.type === "header" ? "header" : diffRowKey(row);
  }

  let topIndex = $state(0);
  let scrollOffset = $state(0);
  let itemOffset = $state<(index: number) => number>(() => 0);
  let stickyBarHeight = $state(0);

  function onScrollState(s: {
    topIndex: number;
    scrollOffset: number;
    itemOffset: (index: number) => number;
  }) {
    topIndex = s.topIndex;
    scrollOffset = s.scrollOffset;
    itemOffset = s.itemOffset;
  }

  // The file whose header is pinned: the topmost row's file, or the file above a
  // gap row (so the bar persists while transitioning between files). Pinned from
  // the moment any of a file's rows reaches the top, including its own header.
  const stickyFileIndex = $derived.by(() => {
    const r = rows[topIndex];
    if (!r) return undefined;
    if (
      r.type === "file-header" ||
      r.type === "hunk-header" ||
      r.type === "line"
    ) {
      return r.fileIndex;
    }
    if (r.type === "file-gap") {
      return r.fileIndex - 1;
    }
    return undefined;
  });

  // The next file's header row, used to push the current sticky bar up and out
  // as the next header rises into it (VSCode-style sticky scroll).
  const nextHeaderIndex = $derived.by(() => {
    for (let i = topIndex + 1; i < rows.length; i++) {
      if (rows[i].type === "file-header") {
        return i;
      }
    }
    return undefined;
  });
  const pushOffset = $derived.by(() => {
    if (nextHeaderIndex === undefined || stickyBarHeight === 0) {
      return 0;
    }
    const nextTop = itemOffset(nextHeaderIndex) - scrollOffset;
    return nextTop < stickyBarHeight ? stickyBarHeight - nextTop : 0;
  });
</script>

<style>
  .diff-list {
    display: flex;
    flex-direction: column;
  }
  .diff:not(:last-of-type) {
    margin-bottom: 1rem;
  }
  /* Zero-height sticky anchor pinned to the viewport top; the bar overflows
     from it (absolute) so it overlays the rows without taking flow space. */
  .sticky-anchor {
    position: sticky;
    top: 0;
    height: 0;
    z-index: 2;
  }
  .sticky-bar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
  }
</style>

{#if codeComments}
  <!-- Patch review keeps the comment-capable path until it is virtualized. -->
  <div class="diff-list">
    {#each diff.files as file}
      <div class="diff">
        <FileDiffComponent {codeComments} {expanded} {file} {head} />
      </div>
    {/each}
  </div>
{:else}
  <div class="sticky-anchor">
    {#if stickyFileIndex !== undefined}
      <div
        class="sticky-bar"
        bind:clientHeight={stickyBarHeight}
        style:transform="translateY({-pushOffset}px)">
        <DiffRowView
          row={{
            type: "file-header",
            fileIndex: stickyFileIndex,
            file: diff.files[stickyFileIndex],
            standalone: false,
          }}
          expanded={isFileExpanded(stickyFileIndex)}
          onToggleFile={toggleFile} />
      </div>
    {/if}
  </div>
  <VirtualList items={rows} getKey={rowKey} {onScrollState}>
    {#snippet row(r)}
      {#if r.type === "header"}
        {@render header?.()}
      {:else}
        <DiffRowView
          row={r}
          expanded={isFileExpanded(r.fileIndex)}
          onToggleFile={toggleFile} />
      {/if}
    {/snippet}
  </VirtualList>
{/if}
