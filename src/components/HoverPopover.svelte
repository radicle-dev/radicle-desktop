<script lang="ts">
  import type { Snippet } from "svelte";

  import debounce from "lodash/debounce";

  interface Props {
    popover: Snippet;
    stylePopoverPositionBottom?: string | undefined;
    stylePopoverPositionLeft?: string | undefined;
    stylePopoverPositionRight?: string | undefined;
    stylePadding?: string | undefined;
    toggle: Snippet;
  }

  const {
    popover,
    stylePopoverPositionBottom,
    stylePopoverPositionLeft,
    stylePopoverPositionRight,
    stylePadding = "0.5rem 1rem",
    toggle,
  }: Props = $props();

  let visible: boolean = $state(false);

  const setVisible = debounce((value: boolean) => {
    visible = value;
  }, 50);
</script>

<style>
  .container {
    position: relative;
    display: inline-block;
  }
  .popover {
    background: var(--color-fill-ghost);
    box-shadow: var(--elevation-low);
    position: absolute;
    clip-path: var(--2px-corner-fill);
    z-index: 10;
  }
</style>

<div class="container">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    role="button"
    tabindex="0"
    onclick={e => {
      e.stopPropagation();
    }}
    onmouseenter={() => setVisible(true)}
    onmouseleave={() => setVisible(false)}>
    {@render toggle()}

    {#if visible}
      <div style:position="absolute">
        <div
          class="popover"
          style:padding={stylePadding}
          style:left={stylePopoverPositionLeft}
          style:right={stylePopoverPositionRight}
          style:bottom={stylePopoverPositionBottom}>
          {@render popover()}
        </div>
      </div>
    {/if}
  </div>
</div>
