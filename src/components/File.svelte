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
    height: 3rem;
    align-items: center;
    padding: 0 0.5rem 0 1rem;
    z-index: 2;
    font-size: var(--font-size-small);
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
