<script lang="ts" generics="T">
  import type { Snippet } from "svelte";

  import { Virtualizer } from "virtua/svelte";

  import { getScrollViewport } from "@app/components/ScrollArea.svelte";

  interface Props {
    items: readonly T[];
    row: Snippet<[T, number]>;
    hasMore?: boolean;
    loadingMore?: boolean;
    onLoadMore?: () => void;
    getKey?: (item: T, index: number) => string | number;
    // Estimated item size in pixels; reduces scrollbar jump before measurement.
    estimatedItemSize?: number;
    // Extra pixels rendered before/after the viewport.
    bufferSize?: number;
    // Height of any content rendered before this list inside the same scroller.
    startMargin?: number;
    // Distance from the bottom (in pixels) at which to request the next page.
    prefetchPx?: number;
  }

  const {
    items,
    row,
    hasMore = false,
    loadingMore = false,
    onLoadMore = undefined,
    getKey = undefined,
    estimatedItemSize = undefined,
    bufferSize = undefined,
    startMargin = undefined,
    prefetchPx = 800,
  }: Props = $props();

  const getViewport = getScrollViewport();
  // Resolves once ScrollArea publishes its viewport (set in OverlayScrollbars'
  // `initialized`, which is deferred).
  const viewport = $derived(getViewport());

  function maybeLoadMore() {
    if (!onLoadMore || !hasMore || loadingMore) return;
    const el = viewport;
    if (!el) return;
    if (el.scrollTop + el.clientHeight >= el.scrollHeight - prefetchPx) {
      onLoadMore();
    }
  }

  $effect(() => {
    // After the dataset grows (or the viewport first appears), the list may not
    // fill the viewport yet, so no scroll event would fire. Re-check on the next
    // frame, once Virtualizer has sized the scroll container.
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    (items.length, hasMore, loadingMore, viewport);
    const id = requestAnimationFrame(() => maybeLoadMore());
    return () => cancelAnimationFrame(id);
  });
</script>

{#if viewport}
  <Virtualizer
    data={items}
    scrollRef={viewport}
    {getKey}
    itemSize={estimatedItemSize}
    {bufferSize}
    {startMargin}
    onscroll={maybeLoadMore}>
    {#snippet children(item: T, index: number)}
      {@render row(item, index)}
    {/snippet}
  </Virtualizer>
{/if}
