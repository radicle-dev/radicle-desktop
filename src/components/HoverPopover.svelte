<script lang="ts">
  import type { Placement } from "@floating-ui/dom";
  import type { Snippet } from "svelte";

  import {
    autoUpdate,
    computePosition,
    flip,
    offset as floatingOffset,
    shift,
  } from "@floating-ui/dom";
  import debounce from "lodash/debounce";

  import { portal } from "@app/lib/portal";

  interface Props {
    popover: Snippet;
    placement?: Placement;
    offset?: number;
    stylePadding?: string;
    toggle: Snippet;
  }

  const {
    popover,
    placement = "top",
    offset: offsetPx = 4,
    stylePadding = "0.5rem 1rem",
    toggle,
  }: Props = $props();

  let visible: boolean = $state(false);
  let anchorEl: HTMLDivElement | undefined = $state();
  let floatingEl: HTMLDivElement | undefined = $state();

  const show = debounce(() => {
    visible = true;
  }, 100);

  const hide = debounce(() => {
    visible = false;
  }, 200);

  $effect(() => {
    if (floatingEl && anchorEl) {
      const cleanup = autoUpdate(anchorEl, floatingEl, () => {
        void computePosition(anchorEl!, floatingEl!, {
          placement,
          middleware: [floatingOffset(offsetPx), flip(), shift({ padding: 8 })],
        }).then(({ x, y }) => {
          if (floatingEl) {
            floatingEl.style.left = `${x}px`;
            floatingEl.style.top = `${y}px`;
            floatingEl.style.visibility = "visible";
          }
        });
      });
      return cleanup;
    }
  });
</script>

<style>
  .container {
    position: relative;
    display: inline-block;
  }
  .popover {
    background: var(--color-surface-subtle);
    border: 1px solid var(--color-border-subtle);
    box-shadow: var(--elevation-low);
    position: fixed;
    top: 0;
    left: 0;
    visibility: hidden;
    border-radius: var(--border-radius-md);
    z-index: 10;
  }
</style>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="container"
  bind:this={anchorEl}
  onmouseenter={() => {
    hide.cancel();
    show();
  }}
  onmouseleave={() => {
    show.cancel();
    hide();
  }}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    role="button"
    tabindex="0"
    onclick={e => {
      e.stopPropagation();
    }}>
    {@render toggle()}
  </div>

  {#if visible}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      use:portal
      bind:this={floatingEl}
      class="popover"
      style:padding={stylePadding}
      onmouseenter={() => {
        hide.cancel();
        show();
      }}
      onmouseleave={() => {
        show.cancel();
        hide();
      }}>
      {@render popover()}
    </div>
  {/if}
</div>
