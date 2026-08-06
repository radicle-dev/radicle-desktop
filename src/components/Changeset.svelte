<script lang="ts">
  import type { CodeComments } from "@app/components/Diff.svelte";
  import type { Diff } from "@bindings/diff/Diff";
  import type { FileDiff } from "@bindings/diff/FileDiff";

  import { tick } from "svelte";
  import { SvelteSet } from "svelte/reactivity";

  import { fileDiffPath } from "@app/lib/diffText";
  import { isIgnoredFile } from "@app/lib/ignoredFiles";

  import FileDiffComponent from "@app/components/FileDiff.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";

  interface Props {
    codeComments?: CodeComments;
    diff: Diff;
    expanded?: boolean;
    head: string;
    rid: string;
    draftReviewId?: string;
  }

  const {
    codeComments,
    diff,
    expanded = true,
    head,
    rid,
    draftReviewId,
  }: Props = $props();

  // Collapse state lives here rather than in each FileDiff: a virtualized row
  // is unmounted once scrolled out of view, which would otherwise reset it.
  // The `expanded` prop is the expand-all/collapse-all command, so re-seed the
  // whole set whenever it flips; individual chevrons deviate until it flips
  // again.
  const collapsed = new SvelteSet<string>();
  $effect(() => {
    const all = expanded;
    collapsed.clear();
    for (const file of diff.files) {
      if (!all || isIgnoredFile(file)) {
        collapsed.add(fileDiffPath(file));
      }
    }
  });
  function isExpanded(file: FileDiff): boolean {
    return !collapsed.has(fileDiffPath(file));
  }
  function setExpanded(file: FileDiff, next: boolean) {
    if (next) {
      collapsed.delete(fileDiffPath(file));
    } else {
      collapsed.add(fileDiffPath(file));
    }
  }

  // virtua takes a single estimate for every item, so aim it at the right
  // total rather than the right file: individual files are still wrong, but the
  // scrollbar starts near its true length and measurement corrections stay
  // local instead of accumulating into a visible jump. Derived from the initial
  // expand state rather than the live one, so collapsing a file mid-scroll
  // doesn't move the estimate under the user.
  const LINE_HEIGHT = 20;
  const HEADER_HEIGHT = 40;
  const FILE_GAP = 16;
  const estimatedItemSize = $derived.by(() => {
    if (diff.files.length === 0) {
      return HEADER_HEIGHT;
    }
    let total = 0;
    for (const file of diff.files) {
      total += HEADER_HEIGHT + FILE_GAP;
      if (!expanded || isIgnoredFile(file) || file.diff.type !== "plain") {
        continue;
      }
      for (const hunk of file.diff.hunks) {
        total += (hunk.lines.length + 1) * LINE_HEIGHT;
      }
    }
    return Math.round(total / diff.files.length);
  });

  // Sticky file header. A row's own `position: sticky` can't work inside the
  // virtualizer's absolutely-positioned items, so the header of the topmost
  // visible file is drawn separately and pinned here.
  let topIndex = $state(0);
  let scrollOffset = $state(0);
  let itemOffset = $state<(index: number) => number>(() => 0);
  let stickyBarHeight = $state(0);

  let list = $state<ReturnType<typeof VirtualList> | undefined>();

  /// Bring a code comment thread into view, expanding and rendering whatever is
  /// needed to get there. Returns false when the file isn't in this diff.
  ///
  /// Two stages, because the file list is virtualised: scroll the file into
  /// existence first, then scroll to the thread inside it. The retry covers the
  /// gap between virtua committing the scroll and the row actually mounting,
  /// which is more than one tick when the row has never been measured.
  export async function revealThread(
    threadId: string,
    path: string,
  ): Promise<boolean> {
    const index = diff.files.findIndex(file => fileDiffPath(file) === path);
    if (index === -1) return false;

    // An ignored file (a lockfile, say) starts collapsed, and a collapsed file
    // renders no threads at all.
    collapsed.delete(path);
    await tick();
    list?.scrollToIndex(index, { align: "start" });

    for (let attempt = 0; attempt < 20; attempt++) {
      await tick();
      // Queried from the document: thread ids are object ids (or a draft's
      // UUID), unique across the page, and the rows live inside the virtualiser
      // rather than in a subtree this component holds a reference to.
      const el = document.querySelector(
        `[data-thread-id="${CSS.escape(threadId)}"]`,
      );
      if (el) {
        el.scrollIntoView({ block: "center", behavior: "smooth" });
        return true;
      }
      await new Promise(resolve => requestAnimationFrame(() => resolve(null)));
    }
    // The file is on screen even if the thread never appeared, which is still
    // closer than where the reader started.
    return true;
  }

  function onScrollState(s: {
    topIndex: number;
    scrollOffset: number;
    itemOffset: (index: number) => number;
  }) {
    topIndex = s.topIndex;
    scrollOffset = s.scrollOffset;
    itemOffset = s.itemOffset;
  }

  const stickyFile = $derived(diff.files[topIndex]);

  // As the next file's header rises into the pinned bar, push the bar up and
  // out rather than letting the two overlap.
  const pushOffset = $derived.by(() => {
    const next = topIndex + 1;
    if (next >= diff.files.length || stickyBarHeight === 0) {
      return 0;
    }
    const nextTop = itemOffset(next) - scrollOffset;
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
  .diff {
    padding-bottom: 1rem;
  }
</style>

<div class="sticky-anchor">
  {#if stickyFile}
    <div
      class="sticky-bar"
      bind:clientHeight={stickyBarHeight}
      style:transform="translateY({-pushOffset}px)">
      <FileDiffComponent
        headerOnly
        sticky={false}
        expanded={isExpanded(stickyFile)}
        onToggle={next => setExpanded(stickyFile, next)}
        file={stickyFile}
        {head}
        {rid}
        {codeComments}
        {draftReviewId} />
    </div>
  {/if}
</div>

<VirtualList
  bind:this={list}
  items={diff.files}
  getKey={file => fileDiffPath(file)}
  {estimatedItemSize}
  autoStartMargin
  {onScrollState}>
  {#snippet row(file)}
    <div class="diff">
      <FileDiffComponent
        sticky={false}
        expanded={isExpanded(file)}
        onToggle={next => setExpanded(file, next)}
        {codeComments}
        {file}
        {head}
        {rid}
        {draftReviewId} />
    </div>
  {/snippet}
</VirtualList>
