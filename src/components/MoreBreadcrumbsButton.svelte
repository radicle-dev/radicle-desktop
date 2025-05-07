<script lang="ts">
  import type { Snippet } from "svelte";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import Popover from "./Popover.svelte";

  interface Props {
    children: Snippet;
  }

  const { children }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

<Popover
  popoverPositionLeft="0"
  popoverPositionTop="2.5rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <NakedButton
      variant="ghost"
      stylePadding="0 4px"
      {onclick}
      active={popoverExpanded}>
      <Icon name="more-vertical" />
    </NakedButton>
  {/snippet}

  {#snippet popover()}
    <Border variant="ghost">
      <div
        class="global-flex txt-monospace"
        style:flex-direction="column"
        style:align-items="flex-start">
        {@render children()}
      </div>
    </Border>
  {/snippet}
</Popover>
