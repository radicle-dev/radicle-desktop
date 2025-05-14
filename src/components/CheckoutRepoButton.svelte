<script lang="ts">
  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    rid: string;
  }

  const { rid }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

<Popover
  popoverPositionRight="0"
  popoverPositionTop="3rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <Button
      styleHeight="2.5rem"
      variant="secondary"
      {onclick}
      active={popoverExpanded}>
      <Icon name="checkout" />Checkout repo
    </Button>
  {/snippet}

  {#snippet popover()}
    <Border
      styleAlignItems="flex-start"
      styleBackgroundColor="var(--color-background-float)"
      styleFlexDirection="column"
      styleGap="0.5rem"
      stylePadding="1rem"
      styleWidth="max-content"
      variant="ghost">
      <span class="txt-small">
        To checkout a working copy of this repo, run:
      </span>
      <Command command={`rad checkout ${rid}`} styleWidth="100%" />
    </Border>
  {/snippet}
</Popover>
