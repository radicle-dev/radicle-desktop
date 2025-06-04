<script lang="ts">
  import { nodeRunning } from "@app/lib/events";

  import Border from "@app/components/Border.svelte";
  import Command from "@app/components/Command.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import Popover from "@app/components/Popover.svelte";

  let popoverExpanded: boolean = $state(false);
</script>

<Popover
  popoverPadding="0"
  popoverPositionTop="2.5rem"
  bind:expanded={popoverExpanded}
  popoverPositionRight="0">
  {#snippet toggle(onclick)}
    <NakedButton variant="ghost" {onclick} active={popoverExpanded}>
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
      <div class="txt-small" style:line-height="1.625rem">
        {#if $nodeRunning}
          Your node is up and running, your changes will be synced
          automatically.
        {:else}
          Your node is not running, changes you make are safe but won't be
          announced.

          <div style:margin-top="1rem">
            You can start your node with:
            <div style:margin-top="0.5rem">
              <Command styleWidth="fit-content" command="rad node start" />
            </div>
          </div>
        {/if}
      </div>
    </Border>
  {/snippet}
</Popover>
