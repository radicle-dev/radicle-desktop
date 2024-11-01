<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    selected: boolean;
    onclick: () => void;
    disabled?: boolean;
    title?: string;
    style?: string;
  }

  const {
    onclick,
    children,
    selected,
    disabled = false,
    title,
    style,
  }: Props = $props();
</script>

<style>
  .item {
    cursor: pointer;
    display: flex;
    align-items: center;
    flex-direction: row;
    gap: 0.375rem;
    padding: 0.5rem 0.375rem;
    white-space: nowrap;
    font-size: var(--font-size-small);
    font-weight: var(--font-weight-regular);
    color: var(--color-foreground-contrast);
    clip-path: var(--1px-corner-fill);
  }
  .item.disabled {
    color: var(--color-foreground-disabled);
  }
  .item:hover,
  .selected {
    background-color: var(--color-fill-ghost);
  }
  .selected {
    font-weight: var(--font-weight-semibold);
    color: var(--color-foreground-contrast);
    background-color: var(--color-fill-ghost);
  }
  .item:hover.selected {
    background-color: var(--color-fill-ghost-hover);
  }
  .item:hover.selected.disabled {
    background-color: var(--color-fill-ghost);
  }
  .item:hover.disabled {
    cursor: not-allowed;
    background-color: var(--color-background-float);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="button"
  tabindex="0"
  class="item"
  class:selected
  class:disabled
  {style}
  {title}
  {onclick}>
  {@render children()}
</div>
