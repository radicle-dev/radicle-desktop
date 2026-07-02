<script lang="ts">
  import type { Diff } from "@bindings/diff/Diff";
  import type { FileDiff } from "@bindings/diff/FileDiff";
  import type { Modification } from "@bindings/diff/Modification";
  import type { Snippet } from "svelte";

  import { SvelteSet } from "svelte/reactivity";

  import type {
    CodeComments,
    CommentContext,
    Selection,
    Side,
  } from "@app/lib/diffComments";
  import {
    buildSelection,
    buildThreadByLine,
    buildThreadCountsByFile,
  } from "@app/lib/diffComments";
  import type { DiffRow } from "@app/lib/diffRows";
  import { diffRowKey, flattenDiff } from "@app/lib/diffRows";

  import DiffRowView from "@app/components/DiffRow.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";

  interface Props {
    codeComments?: CodeComments;
    diff: Diff;
    expanded?: boolean;
    rid: string;
    // Base of the commit range `diff` was built from; unset means `head`'s
    // first parent (a single-commit diff).
    base?: string;
    head: string;
    // Context lines `diff` was built with, so patch text fetched by diff
    // actions matches the rendered diff.
    unified?: number;
    // Rendered as the first virtualized row (e.g. commit/patch metadata), so
    // the virtualizer owns the whole scroll content and starts at offset 0.
    header?: Snippet;
  }

  const {
    codeComments,
    diff,
    expanded = true,
    rid,
    base,
    head,
    unified = 3,
    header,
  }: Props = $props();

  const diffContext = $derived({ rid, base, head, unified });

  // Per-file collapse state. The `expanded` prop is the "expand all"/"collapse
  // all" command: when it flips, re-seed every file; individual chevrons then
  // deviate from that until the next flip.
  const collapsed = new SvelteSet<number>();
  $effect(() => {
    if (expanded) {
      collapsed.clear();
    } else {
      for (let i = 0; i < diff.files.length; i++) {
        collapsed.add(i);
      }
    }
  });
  function isFileExpanded(fileIndex: number): boolean {
    return !collapsed.has(fileIndex);
  }
  function toggleFile(fileIndex: number) {
    if (collapsed.has(fileIndex)) {
      collapsed.delete(fileIndex);
    } else {
      collapsed.add(fileIndex);
    }
  }

  // Patch-review comment state (lifted from the old per-file Diff.svelte so a
  // single virtualizer can own the whole changeset).
  let selection = $state<Selection | undefined>(undefined);

  const threadByLine = $derived(buildThreadByLine(codeComments?.threads ?? []));
  const threadCountsByFile = $derived(
    buildThreadCountsByFile(codeComments?.threads ?? []),
  );

  // eslint-disable-next-line svelte/prefer-writable-derived -- needs a $state proxy so toggleCommentExpand's property mutation triggers reactivity
  let threadExpandedStates: Record<string, boolean> = $state({});
  $effect(() => {
    threadExpandedStates = codeComments
      ? Object.fromEntries(
          codeComments.threads.map(t => [t.root.id, t.root.resolved]),
        )
      : {};
  });
  function toggleCommentExpand(commentId: string) {
    threadExpandedStates[commentId] = !threadExpandedStates[commentId];
  }

  function selectLine(
    fileIdx: number,
    side: Side,
    line: Modification,
    hunkIdx: number,
    lineIdx: number,
    file: FileDiff,
  ) {
    selection = buildSelection(
      head,
      file,
      fileIdx,
      side,
      line,
      hunkIdx,
      lineIdx,
    );
  }

  const comments = $derived<CommentContext | undefined>(
    codeComments
      ? {
          codeComments,
          threadByLine,
          threadCountsByFile,
          selection,
          expandedStates: threadExpandedStates,
          onSelectLine: selectLine,
          onClearSelection: () => {
            selection = undefined;
          },
          onToggleThread: toggleCommentExpand,
        }
      : undefined,
  );

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
      r.type === "file-note" ||
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
        onToggleFile={toggleFile}
        {comments}
        {diffContext} />
    </div>
  {/if}
</div>
<VirtualList items={rows} getKey={rowKey} autoStartMargin {onScrollState}>
  {#snippet row(r)}
    {#if r.type === "header"}
      {@render header?.()}
    {:else}
      <DiffRowView
        row={r}
        expanded={isFileExpanded(r.fileIndex)}
        onToggleFile={toggleFile}
        {comments}
        {diffContext} />
    {/if}
  {/snippet}
</VirtualList>
