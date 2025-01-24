<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    sticky?: boolean;
    leftHeader: Snippet;
    children: Snippet;
  }

  const { sticky = true, leftHeader, children }: Props = $props();
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    height: 2.5rem;
    padding-left: 1rem;
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

<div class="header" class:sticky>
  <div class="left">
    {@render leftHeader()}
  </div>
</div>

<div class="container">
  {@render children()}
</div>
