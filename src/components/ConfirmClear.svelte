<script lang="ts">
  import { closeFocused } from "@app/components/Popover.svelte";

  import Border from "./Border.svelte";
  import Button from "./Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover from "./Popover.svelte";

  interface Props {
    clear: () => void;
    subject: string;
  }

  const { clear, subject }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

<Popover
  popoverPositionRight="0"
  popoverPositionTop="2.5rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <NakedButton
      stylePadding="0 0.25rem"
      variant="ghost"
      {onclick}
      active={popoverExpanded}>
      <Icon name="broom-double" />
    </NakedButton>
  {/snippet}

  {#snippet popover()}
    <Border variant="ghost" stylePadding="1rem">
      <div class="global-flex txt-small">
        <div style:white-space="nowrap" style:margin-right="1rem">
          Clear all {subject} notifications?
        </div>
        <div class="global-flex" style:justify-content="space-between">
          <OutlineButton variant="ghost" onclick={closeFocused}>
            Cancel
          </OutlineButton>
          <Button variant="ghost" onclick={clear}>Clear all</Button>
        </div>
      </div>
    </Border>
  {/snippet}
</Popover>
