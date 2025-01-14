<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    onclick?: () => void;
    disabled?: boolean;
    active?: boolean;
    flatLeft?: boolean;
    flatRight?: boolean;
  }

  const {
    children,
    onclick = undefined,
    disabled = false,
    active = false,
    flatLeft = false,
    flatRight = false,
  }: Props = $props();
</script>

<style>
  .container {
    white-space: nowrap;

    -webkit-touch-callout: none;
    -webkit-user-select: none;
    user-select: none;
    display: flex;
    flex-direction: row;
    font-size: var(--font-size-small);
  }

  .wrapper {
    position: relative;
    display: flex;
    gap: 0.5rem;
    padding: 5px 0;
  }

  .active {
    font-weight: var(--font-weight-semibold);
    color: var(--color-foreground-emphasized);
  }

  .wrapper.active::after {
    position: absolute;
    z-index: 1;
    content: " ";
    background-color: var(--color-fill-secondary);
    height: 2px;
    bottom: -4px;
    width: 100%;
  }

  .container.disabled {
    color: var(--color-foreground-disabled);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="container"
  style:cursor={!disabled ? "pointer" : "default"}
  class:disabled
  class:flat-right={flatRight}
  class:flat-left={flatLeft}
  onclick={!disabled ? onclick : undefined}
  role="button"
  tabindex="0">
  <div class="wrapper" class:active>
    {@render children()}
  </div>
</div>
