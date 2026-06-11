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
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    height: 2.5rem;
    padding: 0 0.5rem;
    font: var(--txt-body-m-regular);
    cursor: pointer;
    background-color: var(--color-surface-canvas);
  }
  .header.expanded {
    border-bottom: 1px solid var(--color-border-subtle);
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
  .icon-stack,
  .icon-expanded {
    display: grid;
    width: 1rem;
    height: 1rem;
    place-items: center;
  }
  .icon-default,
  .icon-hover {
    grid-area: 1 / 1;
    transition:
      opacity 150ms ease,
      transform 150ms ease;
  }
  .icon-hover {
    opacity: 0;
    transform: rotate(-90deg);
  }
  .header:hover .icon-default,
  .header:focus-visible .icon-default {
    opacity: 0;
    transform: rotate(90deg);
  }
  .header:hover .icon-hover,
  .header:focus-visible .icon-hover {
    opacity: 1;
    transform: rotate(0);
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
  }
  .collapsible-inner {
    overflow: hidden;
    min-height: 0;
  }
</style>

<div
  style:display="flex"
  style:align-items="flex-start"
  style:flex-direction="column"
  style:width="100%">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="header"
    class:expanded
    role="button"
    tabindex="0"
    onclick={() => (expanded = !expanded)}
    style:width="100%">
    <div class="left">
      {#if expanded}
        <span class="icon-expanded"><Icon name="chevron-down" /></span>
      {:else}
        <span class="icon-stack">
          <span class="icon-default"><Icon name="commit" /></span>
          <span class="icon-hover"><Icon name="chevron-down" /></span>
        </span>
      {/if}
      {@render leftHeader()}
    </div>
  </div>

  <div class="collapsible" class:open={expanded}>
    <div class="collapsible-inner">
      {@render children()}
    </div>
  </div>
</div>
