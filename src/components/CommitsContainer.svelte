<script lang="ts">
  import type { Snippet } from "svelte";

  import Icon from "@app/components/Icon.svelte";

  interface Props {
    leftHeader: Snippet;
    children: Snippet;
  }

  const { leftHeader, children }: Props = $props();

  let expanded = $state(true);
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    height: 2.5rem;
    padding: 0 0.5rem;
    font: var(--txt-body-m-regular);
    cursor: pointer;
    border-radius: var(--border-radius-md);
  }
  .header.expanded {
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
  }
  .header:hover,
  .header:focus-visible {
    background-color: var(--color-surface-subtle);
  }
  .left {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  .collapsible {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows 180ms ease-out;
    overflow: hidden;
    width: 100%;
  }
  .collapsible.open {
    grid-template-rows: 1fr;
    border-top: 1px solid var(--color-border-subtle);
  }
  .collapsible-inner {
    overflow: hidden;
    min-height: 0;
  }
</style>

<div
  style:border="1px solid var(--color-border-subtle)"
  style:border-radius="var(--border-radius-md)"
  style:display="flex"
  style:align-items="flex-start"
  style:background-color="var(--color-surface-canvas)"
  style:flex-direction="column">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="header"
    class:expanded
    role="button"
    tabindex="0"
    onclick={() => (expanded = !expanded)}
    style:width="100%">
    <div class="left">
      <Icon name={expanded ? "chevron-down" : "chevron-right"} />
      {@render leftHeader()}
    </div>
  </div>

  <div class="collapsible" class:open={expanded}>
    <div class="collapsible-inner">
      {@render children()}
    </div>
  </div>
</div>
