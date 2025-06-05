<script lang="ts">
  import Clipboard from "@app/components/Clipboard.svelte";
  import Border from "./Border.svelte";

  interface Props {
    command: string;
    styleWidth: string;
    showPrompt?: boolean;
  }

  const { command, styleWidth, showPrompt = true }: Props = $props();

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

<div class="cmd txt-monospace txt-small" style:width={styleWidth}>
  <Border
    hoverable
    onclick={() => clipboard.copy()}
    styleOverflow="hidden"
    styleBackgroundColor="var(--color-background-float)"
    styleCursor="pointer"
    styleJustifyContent="space-between"
    stylePadding="0.25rem 0.5rem"
    {styleWidth}
    variant="ghost">
    <span class="txt-overflow">
      {showPrompt ? "$ " : ""}{command}
    </span>
    <Clipboard bind:this={clipboard} text={command} />
  </Border>
</div>
