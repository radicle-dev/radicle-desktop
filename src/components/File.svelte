<script lang="ts">
  import type { Snippet } from "svelte";

  import { tick } from "svelte";

  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";

  interface Props {
    sticky?: boolean;
    leftHeader: Snippet;
    children: Snippet;
    expanded: boolean;
  }

  /* eslint-disable prefer-const */
  let { sticky = true, leftHeader, children, expanded }: Props = $props();
  /* eslint-enable prefer-const */

  let header: HTMLElement | undefined = $state();
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    height: 2.5rem;
    padding-left: 0.5rem;
    z-index: 2;
    font-size: var(--font-size-small);
    background-color: var(--color-background-default);
  }
  .header::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-fill-float-hover);
    clip-path: var(--2px-top-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
  }
  .header.collapsed {
    clip-path: var(--2px-corner-fill);
  }

  .sticky {
    position: sticky;
    top: 0;
  }

  .left {
    display: flex;
    gap: 0.5rem;
    margin-right: 1rem;
    align-items: center;
  }

  .container {
    position: relative;
    overflow-x: auto;
    z-index: 1;
  }
  .container::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--2px-bottom-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
  }
</style>

<div class="header" class:sticky class:collapsed={!expanded} bind:this={header}>
  <div class="left">
    <NakedButton
      stylePadding="0 4px"
      variant="ghost"
      onclick={async () => {
        expanded = !expanded;
        if (!expanded && header) {
          await tick();
          header.scrollIntoView({ behavior: "smooth", block: "nearest" });
        }
      }}>
      <Icon name={expanded ? "chevron-down" : "chevron-right"} />
    </NakedButton>
    {@render leftHeader()}
  </div>
</div>

{#if expanded}
  <div class="container">
    {@render children()}
  </div>
{/if}
