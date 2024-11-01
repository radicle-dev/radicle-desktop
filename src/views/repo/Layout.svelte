<script lang="ts">
  import type { Snippet } from "svelte";

  import { onMount } from "svelte";

  import Header from "@app/components/Header.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  interface Props {
    children: Snippet;
    breadcrumbs: Snippet;
    headerCenter?: Snippet;
    sidebar: Snippet;
    loadMore?: () => Promise<void>;
  }

  const {
    children,
    breadcrumbs,
    headerCenter = undefined,
    sidebar,
    loadMore = undefined,
  }: Props = $props();

  let hidden = $state(false);
  let listElement: HTMLElement | undefined = $state();
  let loading = false;

  onMount(() => {
    if (listElement && loadMore) {
      listElement.addEventListener("scroll", async () => {
        if (
          listElement &&
          listElement.scrollTop + listElement.clientHeight >=
            listElement.scrollHeight - 600 &&
          loading === false
        ) {
          loading = true;
          void loadMore().finally(() => (loading = false));
        }
      });
    }
  });
</script>

<style>
  .layout {
    display: grid;
    grid-template: auto 1fr auto / auto 1fr auto;
    height: 100%;
  }

  .header {
    grid-column: 1 / 4;
    border-bottom: 2px solid var(--color-background-default);
  }

  .sidebar {
    grid-column: 1 / 2;
    margin: 1rem 0.5rem 0 1rem;
    min-width: 14rem;
    overflow: scroll;
  }

  .content {
    padding-top: 1rem;
    grid-column: 2 / 3;
    overflow: scroll;
    overscroll-behavior: none;
  }

  .hidden {
    display: none;
  }
</style>

<div class="layout">
  <div class="header">
    <Header {breadcrumbs} center={headerCenter}>
      {#snippet iconLeft()}
        <NakedButton
          variant="ghost"
          onclick={() => {
            hidden = !hidden;
          }}>
          <Icon name="sidebar" />
        </NakedButton>
      {/snippet}
    </Header>
  </div>

  <div class="sidebar" class:hidden>
    {@render sidebar()}
  </div>

  <div
    class="content global-reset-scroll-after-navigate"
    bind:this={listElement}>
    {@render children()}
  </div>
</div>
