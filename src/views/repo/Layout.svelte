<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Snippet } from "svelte";

  import type { SidebarData } from "@app/lib/router/definitions";

  import AppSidebar from "@app/components/AppSidebar.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";

  interface Props {
    children: Snippet;
    sidebarData: SidebarData;
    activeRepo?: RepoInfo;
    loadMoreContent?: () => Promise<void>;
    selfScroll?: boolean;
  }

  const {
    children,
    sidebarData,
    activeRepo = undefined,
    loadMoreContent = undefined,
    selfScroll = false,
  }: Props = $props();

  let loadingContent = false;
</script>

<style>
  .layout {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>

<div class="layout">
  <AppSidebar {sidebarData} {activeRepo} />
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
</div>
