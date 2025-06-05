<script lang="ts">
  import type { Snippet } from "svelte";

  import Icon from "./Icon.svelte";

  const { href, children }: { href: string; children?: Snippet } = $props();
</script>

<style>
  a {
    font-weight: var(--font-weight-semibold);
    color: inherit;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    text-decoration: none;
  }
  a:hover {
    text-decoration: underline;
    text-underline-offset: 2px;
    color: var(--color-fill-secondary);
  }

  .icon {
    color: var(--color-foreground-dim);
    position: relative;
    bottom: 1px;
  }

  a:hover .icon {
    color: var(--color-fill-secondary-hover);
  }
</style>

<a {href} target="_blank" rel="noreferrer">
  {#if children}
    {@render children()}
  {:else}
    {href}
  {/if}
  <span class="icon"><Icon name="open-external" /></span>
</a>
