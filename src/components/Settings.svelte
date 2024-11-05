<script lang="ts">
  import type { ComponentProps } from "svelte";

  import AnnounceSwitch from "./AnnounceSwitch.svelte";
  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import Popover from "./Popover.svelte";
  import ThemeSwitch from "./ThemeSwitch.svelte";

  interface Props {
    compact?: boolean;
    styleHeight?: string;
    popoverProps: Partial<ComponentProps<typeof Popover>>;
  }

  const {
    compact = true,
    styleHeight = "40px",
    popoverProps,
  }: Props = $props();
</script>

<Popover {...popoverProps}>
  {#snippet toggle(onclick)}
    <NakedButton title="Settings" variant="ghost" {onclick} {styleHeight}>
      <Icon name="settings" />
      {#if !compact}
        Settings
      {/if}
    </NakedButton>
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost" stylePadding="0.5rem 1rem" styleWidth="27rem">
      <div
        class="global-flex"
        style:flex-direction="column"
        style:align-items="flex-start"
        style:gap="1rem"
        style:width="100%">
        <div
          class="global-flex"
          style:justify-content="space-between"
          style:width="100%">
          Theme <ThemeSwitch />
        </div>
        <div
          class="global-flex"
          style:justify-content="space-between"
          style:width="100%">
          Announce changes <AnnounceSwitch />
        </div>
      </div>
    </Border>
  {/snippet}
</Popover>
