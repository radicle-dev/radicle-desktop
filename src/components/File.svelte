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
    border-top-left-radius: var(--border-radius-small);
    border-top-right-radius: var(--border-radius-small);
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
    border-bottom-left-radius: var(--border-radius-small);
    border-bottom-right-radius: var(--border-radius-small);
  }
  @media (max-width: 719.98px) {
    .header {
      border-radius: 0;
      border-left: 0;
      border-right: 0;
      padding: 0 1rem 0 1rem;
    }
    .container {
      border-radius: 0;
      border-left: 0;
      border-right: 0;
    }
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
