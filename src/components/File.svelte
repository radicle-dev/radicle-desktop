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
    border: 1px solid var(--color-border-hint);
    background-color: var(--color-background-default);
    z-index: 2;
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
    border: 1px solid var(--color-border-hint);
    border-top: 0;
    background: var(--color-background-float);
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
