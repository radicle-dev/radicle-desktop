<script lang="ts">
  import Clipboard from "@app/components/Clipboard.svelte";
  import Border from "./Border.svelte";

  interface Props {
    command: string;
    styleWidth: string;
  }

  const { command, styleWidth }: Props = $props();

  let clipboard: Clipboard;
</script>

<style>
  .cmd {
    color: var(--color-foreground-dim);
  }
  .cmd:hover {
    color: var(--color-foreground-contrast);
  }
</style>

<div class="cmd txt-monospace" style:width={styleWidth}>
  <Border
    hoverable
    onclick={() => clipboard.copy()}
    styleBackgroundColor="var(--color-background-float)"
    styleCursor="pointer"
    styleJustifyContent="space-between"
    stylePadding="0.25rem 0.5rem"
    {styleWidth}
    variant="ghost">
    $ {command}
    <Clipboard bind:this={clipboard} text={command} />
  </Border>
</div>
