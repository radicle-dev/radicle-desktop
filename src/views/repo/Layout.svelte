<script lang="ts" module>
  type LayoutState = "one-column" | "two-column";

  const LAYOUT_KEY = "one-column-layout-enabled";

  let oneColumnLayout = $state(
    localStorage ? localStorage.getItem(LAYOUT_KEY) === "one-column" : false,
  );

  export function getLayout() {
    return oneColumnLayout;
  }

  export function storeLayout(newValue: LayoutState): void {
    oneColumnLayout = newValue === "one-column";
    if (localStorage) {
      localStorage.setItem(LAYOUT_KEY, newValue);
    } else {
      console.warn(
        "localStorage isn't available, not able to persist the selected layout settings without it.",
      );
    }
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  import { onMount } from "svelte";

  import Header from "@app/components/Header.svelte";

  interface Props {
    children: Snippet;
    publicKey: string;
    headerCenter?: Snippet;
    secondColumn: Snippet;
    sidebar: Snippet;
    loadMoreContent?: () => Promise<void>;
    loadMoreSecondColumn?: () => Promise<void>;
    hideSidebar?: boolean;
    styleSecondColumnOverflow?: string;
  }

  const {
    children,
    publicKey,
    headerCenter = undefined,
    secondColumn,
    sidebar,
    loadMoreContent = undefined,
    loadMoreSecondColumn = undefined,
    hideSidebar = false,
    styleSecondColumnOverflow = "scroll",
  }: Props = $props();

  let contentContainer: HTMLElement | undefined = $state();
  let secondColumnContainer: HTMLElement | undefined = $state();
  let loadingContent = false;
  let loadingSecondColumn = false;

  onMount(() => {
    if (contentContainer && loadMoreContent) {
      contentContainer.addEventListener("scroll", async () => {
        if (
          contentContainer &&
          contentContainer.scrollTop + contentContainer.clientHeight >=
            contentContainer.scrollHeight - 600 &&
          loadingContent === false
        ) {
          loadingContent = true;
          void loadMoreContent().finally(() => (loadingContent = false));
        }
      });
    }

    if (secondColumnContainer && loadMoreSecondColumn) {
      secondColumnContainer.addEventListener("scroll", async () => {
        if (
          secondColumnContainer &&
          secondColumnContainer.scrollTop +
            secondColumnContainer.clientHeight >=
            secondColumnContainer.scrollHeight - 600 &&
          loadingSecondColumn === false
        ) {
          loadingSecondColumn = true;
          void loadMoreSecondColumn().finally(
            () => (loadingSecondColumn = false),
          );
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
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
  }

  .secondColumn {
    grid-column: 2 / 3;
    max-width: 28rem;
    min-width: 14rem;
    padding: 1rem 1rem 1rem 0;
  }

  .content {
    grid-column: 3 / 4;
    width: 100%;
    overflow: scroll;
    overscroll-behavior: none;
  }
</style>

<div class="layout">
  <div class="header">
    <Header {publicKey} center={headerCenter}></Header>
  </div>

  <div
    class="sidebar"
    style:display={hideSidebar ? "none" : "flex"}
    style:padding-right="1rem">
    {@render sidebar()}
  </div>

  <div
    class="secondColumn"
    style:padding-left={hideSidebar ? "1rem" : "0"}
    bind:this={secondColumnContainer}
    style:display={oneColumnLayout && !hideSidebar ? "none" : undefined}
    style:overflow={styleSecondColumnOverflow}>
    {@render secondColumn()}
  </div>

  <div
    class="content global-reset-scroll-after-navigate"
    bind:this={contentContainer}>
    {@render children()}
  </div>
</div>
