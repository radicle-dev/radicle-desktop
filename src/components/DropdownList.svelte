<script lang="ts" generics="T">
  import type { Snippet } from "svelte";

  import { useOverlayScrollbars } from "overlayscrollbars-svelte";

  interface Props {
    item: Snippet<[T]>;
    empty?: Snippet;
    items: T[];
    styleDropdownMinWidth?: string;
  }

  const { item, empty, items, styleDropdownMinWidth }: Props = $props();

  let dropdownElement: HTMLDivElement | undefined = undefined;

  $effect(() => {
    if (dropdownElement) {
      const [initialize] = useOverlayScrollbars({
        options: () => ({
          scrollbars: {
            theme: "global-os-theme-radicle",
            autoHide: "scroll",
          },
        }),
        defer: true,
      });

      initialize({ target: dropdownElement });
    }
  });
</script>

<style>
  .dropdown {
    align-items: center;
    max-height: 60vh;
    overflow-y: auto;
  }
  .dropdown-item {
    padding: 2px;
    font-size: var(--font-size-small);
  }
</style>

<div
  class="dropdown"
  bind:this={dropdownElement}
  style:min-width={styleDropdownMinWidth}>
  {#each items as i}
    <div class="dropdown-item">
      {@render item(i)}
    </div>
  {:else}
    <div class="dropdown-item">
      {@render empty?.()}
    </div>
  {/each}
</div>
