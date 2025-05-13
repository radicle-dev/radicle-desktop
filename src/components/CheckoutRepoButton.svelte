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
</script>

<Popover popoverPositionRight="0" popoverPositionTop="3rem">
  {#snippet toggle(onclick)}
    <Button styleHeight="2.5rem" variant="secondary" {onclick}>
      <Icon name="checkout" />Checkout repo<Icon name="chevron-down" />
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
