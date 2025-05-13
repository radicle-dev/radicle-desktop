<script lang="ts">
  import { formatOid } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    patchId: string;
    selectedRevisionId: string;
    tab: "patch" | "revisions" | "timeline";
  }

  const { patchId, selectedRevisionId, tab }: Props = $props();

  const checkoutCommand = $derived.by(() => {
    if (tab === "revisions" && selectedRevisionId !== patchId) {
      return `rad patch checkout ${formatOid(patchId)} --revision ${formatOid(selectedRevisionId)}`;
    } else {
      return `rad patch checkout ${formatOid(patchId)}`;
    }
  });
</script>

<Popover popoverPositionRight="0" popoverPositionTop="3rem">
  {#snippet toggle(onclick)}
    <Button styleHeight="2.5rem" variant="secondary" {onclick}>
      <Icon name="checkout" />Checkout patch<Icon name="chevron-down" />
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
        To checkout this patch in your working copy, run:
      </span>
      <Command command={checkoutCommand} styleWidth="100%" />
    </Border>
  {/snippet}
</Popover>
