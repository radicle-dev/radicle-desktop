<script lang="ts">
  import type { Snippet } from "svelte";

  import debounce from "lodash/debounce";

  interface Props {
    popover: Snippet;
    stylePopoverPositionBottom: string | undefined;
    stylePopoverPositionLeft: string | undefined;
    toggle: Snippet;
  }

  const {
    popover,
    stylePopoverPositionBottom,
    stylePopoverPositionLeft,
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
    border-radius: var(--border-radius-regular);
    padding: 0.5rem 1rem;
    box-shadow: var(--elevation-low);
    position: absolute;
    clip-path: var(--2px-corner-fill);
    z-index: 10;
  }
</style>

<div class="container">
  <div
    role="button"
    tabindex="0"
    onmouseenter={() => setVisible(true)}
    onmouseleave={() => setVisible(false)}>
    {@render toggle()}

    {#if visible}
      <div style:position="absolute">
        <div
          class="popover"
          style:left={stylePopoverPositionLeft}
          style:bottom={stylePopoverPositionBottom}>
          {@render popover()}
        </div>
      </div>
    {/if}
  </div>
</div>
