<script lang="ts">
  import type { Snippet } from "svelte";

  import Clipboard from "./Clipboard.svelte";

  const {
    inline = false,
    children,
    id,
  }: { inline?: boolean; children?: Snippet; id: string } = $props();

  let clipboard: Clipboard;
</script>

<style>
  .copyable-id {
    cursor: pointer;
    color: var(--color-foreground-dim);
    gap: 0.25rem;
  }

  .copyable-id:hover {
    color: var(--color-foreground-contrast);
  }
  .inline {
    display: inline-flex;
    gap: 0;
    white-space: nowrap;
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="button"
  tabindex="0"
  onclick={() => clipboard.copy()}
  class:inline
  class="copyable-id global-flex txt-small txt-monospace">
  {#if children}
    {@render children()}
  {:else}
    {id}
  {/if}
  <Clipboard bind:this={clipboard} text={id} />
</div>
