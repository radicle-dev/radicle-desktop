<script lang="ts">
  import { nodeRunning } from "@app/lib/events";

  import Border from "@app/components/Border.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import Popover from "@app/components/Popover.svelte";
</script>

<Popover
  popoverPadding="0"
  popoverPositionTop="2.5rem"
  popoverPositionRight="0">
  {#snippet toggle(onclick)}
    <NakedButton variant="ghost" {onclick}>
      {#if $nodeRunning}
        <Icon name="online" />
        Online
      {:else}
        <Icon name="offline" />
        Offline
      {/if}
    </NakedButton>
  {/snippet}
  {#snippet popover()}
    <Border
      variant="ghost"
      stylePadding="1rem"
      styleMinWidth="20rem"
      styleAlignItems="flex-start"
      styleFlexDirection="column">
      <div class="txt-small txt-missing">
        {#if $nodeRunning}
          Your node is up and running, your changes will be synced
          automatically.
        {:else}
          Your node is not running, changes you make are safe but won't be
          announced.
        {/if}
      </div>
    </Border>
  {/snippet}
</Popover>
