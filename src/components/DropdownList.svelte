<script lang="ts" generics="T">
  import type { Snippet } from "svelte";

  interface Props {
    item: Snippet<[T]>;
    empty?: Snippet;
    items: T[];
    styleDropdownMinWidth?: string;
  }

  const { item, empty, items, styleDropdownMinWidth }: Props = $props();
</script>

<style>
  .dropdown {
    align-items: center;
    border-radius: var(--border-radius-small);
    max-height: 60vh;
    overflow-y: auto;
  }
  .dropdown-item {
    padding: 2px;
    font-size: var(--font-size-small);
  }
</style>

<div class="dropdown" style:min-width={styleDropdownMinWidth}>
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
