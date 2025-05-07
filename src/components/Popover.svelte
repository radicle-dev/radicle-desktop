<script lang="ts" module>
  let focused = $state<{ element: HTMLDivElement; id: string } | undefined>(
    undefined,
  );

  export function closeFocused() {
    focused = undefined;
  }

  export function setFocused(id: string) {
    const thisComponent = document.querySelector(`[data-popover-id="${id}"]`);
    if (thisComponent) {
      if (focused?.element === thisComponent) {
        closeFocused();
      } else {
        focused = { element: thisComponent as HTMLDivElement, id };
      }
    }
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    toggle: Snippet<[() => void]>;
    popover: Snippet;
    popoverId?: string;
    popoverContainerMinWidth?: string;
    popoverPadding?: string;
    popoverPositionBottom?: string;
    popoverPositionLeft?: string;
    popoverPositionRight?: string;
    popoverPositionTop?: string;
  }

  /* eslint-disable prefer-const */
  let {
    toggle,
    popover,
    popoverId,
    popoverContainerMinWidth,
    popoverPadding,
    popoverPositionBottom,
    popoverPositionLeft,
    popoverPositionRight,
    popoverPositionTop,
  }: Props = $props();
  /* eslint-enable prefer-const */

  const id = popoverId ?? crypto.randomUUID();
  let thisComponent: HTMLDivElement | undefined = $state();

  function clickOutside(ev: MouseEvent | TouchEvent) {
    const toggleElement = document.querySelector(
      `[data-popover-toggle="${id}"]`,
    );
    if (focused && !ev.composedPath().includes(focused.element)) {
      if (
        thisComponent === focused.element &&
        !ev.composedPath().includes(toggleElement!)
      ) {
        closeFocused();
      }
    }
  }

  function toggleFn() {
    if (thisComponent) {
      if (focused?.element === thisComponent) {
        closeFocused();
      } else {
        focused = { element: thisComponent, id };
      }
    }
  }
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
  data-popover-id={id}
  bind:this={thisComponent}
  class="container"
  style:min-width={popoverContainerMinWidth}>
  {@render toggle(toggleFn)}

  {#if focused?.element === thisComponent}
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
