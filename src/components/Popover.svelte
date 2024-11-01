<script lang="ts" module>
  import { writable } from "svelte/store";
  const focused = writable<HTMLDivElement | undefined>(undefined);

  export function closeFocused() {
    focused.set(undefined);
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    toggle: Snippet<[() => void]>;
    popover: Snippet;
    popoverContainerMinWidth?: string;
    popoverPadding?: string;
    popoverPositionBottom?: string;
    popoverPositionLeft?: string;
    popoverPositionRight?: string;
    popoverPositionTop?: string;
    expanded?: boolean;
  }

  /* eslint-disable prefer-const */
  let {
    toggle,
    popover,
    popoverContainerMinWidth,
    popoverPadding,
    popoverPositionBottom,
    popoverPositionLeft,
    popoverPositionRight,
    popoverPositionTop,
    expanded = $bindable(false),
  }: Props = $props();
  /* eslint-enable prefer-const */

  let thisComponent: HTMLDivElement | undefined = $state();

  function clickOutside(ev: MouseEvent | TouchEvent) {
    if ($focused && !ev.composedPath().includes($focused)) {
      closeFocused();
    }
  }

  function toggleFn() {
    expanded = !expanded;
    if ($focused === thisComponent) {
      closeFocused();
    } else {
      focused.set(thisComponent);
    }
  }

  $effect(() => {
    expanded = $focused === thisComponent;
  });
</script>

<style>
  .container {
    position: relative;
  }
  .popover {
    box-shadow: var(--elevation-low);
    position: absolute;
    z-index: 10;
  }
</style>

<svelte:window onclick={clickOutside} ontouchstart={clickOutside} />

<div
  bind:this={thisComponent}
  class="container"
  style:min-width={popoverContainerMinWidth}>
  {@render toggle(toggleFn)}

  {#if expanded}
    <div
      class="popover"
      style:bottom={popoverPositionBottom}
      style:left={popoverPositionLeft}
      style:right={popoverPositionRight}
      style:top={popoverPositionTop}
      style:padding={popoverPadding}>
      {@render popover()}
    </div>
  {/if}
</div>
