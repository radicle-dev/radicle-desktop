<script lang="ts">
  import type { ComponentProps } from "svelte";

  import { onMount } from "svelte";
  import { invoke } from "@app/lib/invoke";

  import AnnounceSwitch from "./AnnounceSwitch.svelte";
  import Border from "./Border.svelte";
  import CopyableId from "./CopyableId.svelte";
  import FontSizeSwitch from "./FontSizeSwitch.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import Popover from "./Popover.svelte";
  import ThemeSwitch from "./ThemeSwitch.svelte";

  interface Props {
    compact?: boolean;
    styleHeight?: ComponentProps<typeof NakedButton>["styleHeight"];
    popoverProps: Partial<ComponentProps<typeof Popover>>;
  }

  let version = $state("");

  onMount(async () => {
    version = await invoke<string>("version");
  });

  const {
    compact = true,
    styleHeight = "2.5rem",
    popoverProps,
  }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

<Popover {...popoverProps} bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <NakedButton
      title="Settings"
      variant="ghost"
      {onclick}
      {styleHeight}
      active={popoverExpanded}>
      <Icon name="settings" />
      {#if !compact}
        Settings
      {/if}
    </NakedButton>
  {/snippet}
  {#snippet popover()}
    <Border variant="ghost" stylePadding="0.5rem 1rem" styleWidth="27rem">
      <div
        class="global-flex txt-small"
        style:flex-direction="column"
        style:align-items="flex-start"
        style:gap="1rem"
        style:width="100%">
        <div
          class="global-flex"
          style:justify-content="space-between"
          style:width="100%"
          style:min-height="2rem">
          Version <CopyableId id={version} />
        </div>

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
        <div
          class="global-flex"
          style:justify-content="space-between"
          style:width="100%">
          Font size <FontSizeSwitch />
        </div>
      </div>
    </Border>
  {/snippet}
</Popover>
