<script lang="ts">
  import type { Snippet } from "svelte";

  import ScrollArea from "@app/components/ScrollArea.svelte";

  interface Props {
    children: Snippet;
    loadMoreContent?: () => Promise<void>;
    selfScroll?: boolean;
  }

  const {
    children,
    loadMoreContent = undefined,
    selfScroll = false,
  }: Props = $props();

  let loadingContent = false;
</script>

{#if selfScroll}
  <div
    style="height: 100%; overflow: hidden; min-width: 0; background-color: var(--color-surface-canvas);">
    {@render children()}
  </div>
{:else}
  <ScrollArea
    style="height: 100%; width: 100%; background-color: var(--color-surface-canvas);"
    onScrollHalf={loadMoreContent
      ? () => {
          if (!loadingContent) {
            loadingContent = true;
            void loadMoreContent().finally(() => (loadingContent = false));
          }
        }
      : undefined}>
    {@render children()}
  </ScrollArea>
{/if}
