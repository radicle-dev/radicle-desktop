<script lang="ts">
  import type { Snippet } from "svelte";
  type LayoutState = "one-column" | "two-column";

  import { onMount } from "svelte";

  import Header from "@app/components/Header.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    children: Snippet;
    breadcrumbs: Snippet;
    headerCenter?: Snippet;
    secondColumn: Snippet;
    sidebar: Snippet;
    loadMore?: () => Promise<void>;
    hideSidebar?: boolean;
    styleSecondColumnOverflow?: string;
  }

  const {
    children,
    breadcrumbs,
    headerCenter = undefined,
    secondColumn,
    sidebar,
    loadMore = undefined,
    hideSidebar = false,
    styleSecondColumnOverflow = "scroll",
  }: Props = $props();

  const LAYOUT_KEY = "one-column-layout-enabled";

  let oneColumnLayout = $state(
    localStorage ? localStorage.getItem(LAYOUT_KEY) === "one-column" : false,
  );
  let listElement: HTMLElement | undefined = $state();
  let loading = false;

  function storeLayout(newValue: LayoutState): void {
    oneColumnLayout = newValue === "one-column";
    if (localStorage) {
      localStorage.setItem(LAYOUT_KEY, newValue);
    } else {
      console.warn(
        "localStorage isn't available, not able to persist the selected layout settings without it.",
      );
    }
  }

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
    grid-template-columns: auto auto 1fr auto;
    grid-template-rows: auto 1fr auto;
    height: 100%;
  }

  .header {
    grid-column: 1 / 4;
    border-bottom: 2px solid var(--color-background-default);
  }

  .sidebar {
    grid-column: 1 / 2;
    width: 40px;
    margin: 0 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    margin-top: 13px;
    margin-bottom: 1rem;
  }

  .secondColumn {
    grid-column: 2 / 3;
    margin: 1rem 0 0 0;
    max-width: 30rem;
    min-width: 14rem;
    margin-right: 1rem;
  }

  .content {
    padding-top: 1rem;
    grid-column: 3 / 4;
    width: 100%;
    overflow: scroll;
    overscroll-behavior: none;
  }

  .column-radio {
    display: flex;
    background-color: var(--color-background-dip);
    clip-path: var(--1px-corner-fill);
    gap: 2px;
  }
  .toggle {
    cursor: pointer;
    border: 0;
    height: 24px;
    clip-path: var(--1px-corner-fill);
    margin: 0;
    background-color: var(--color-fill-ghost);
    color: var(--color-foreground-active);
  }
  .toggle:hover,
  .toggle.active {
    background: none;
    color: var(--color-foreground-emphasized);
  }
</style>

<div class="layout">
  <div class="header">
    <Header {breadcrumbs} center={headerCenter}>
      {#snippet columnSwitch()}
        <div class="column-radio">
          <button
            class="toggle"
            class:active={oneColumnLayout}
            onclick={() => {
              storeLayout("one-column");
            }}>
            <Icon name="one" />
          </button>
          <button
            class="toggle"
            class:active={!oneColumnLayout}
            onclick={() => {
              storeLayout("two-column");
            }}>
            <Icon name="two" />
          </button>
        </div>
      {/snippet}
    </Header>
  </div>

  <div
    class="sidebar"
    style:display={hideSidebar && !oneColumnLayout ? "none" : "flex"}>
    {@render sidebar()}
  </div>

  <div
    class="secondColumn"
    style:display={oneColumnLayout ? "none" : undefined}
    style:overflow={styleSecondColumnOverflow}>
    {@render secondColumn()}
  </div>

  <div
    class="content global-reset-scroll-after-navigate"
    bind:this={listElement}>
    {@render children()}
  </div>
</div>
