<script lang="ts">
  import Clipboard from "./Clipboard.svelte";

  const { id }: { id: string } = $props();

  let clipboard: Clipboard;
</script>

<style>
  .copyable-id {
    cursor: pointer;
    color: var(--color-foreground-dim);
  }

  .copyable-id:hover {
    color: var(--color-foreground-contrast);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="button"
  tabindex="0"
  onclick={() => clipboard.copy()}
  class="copyable-id global-flex txt-small txt-monospace">
  {id}
  <Clipboard bind:this={clipboard} text={id} />
</div>
