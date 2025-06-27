<script lang="ts" module>
  export const settingsPopoverToggleId = "settings-popover-toggle";
</script>

<script lang="ts">
  import type { ComponentProps } from "svelte";

  import { updateChecker } from "@app/lib/updateChecker.svelte";

  import AnnounceSwitch from "./AnnounceSwitch.svelte";
  import Border from "./Border.svelte";
  import CopyableId from "./CopyableId.svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import FontSizeSwitch from "./FontSizeSwitch.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover from "./Popover.svelte";
  import ThemeSwitch from "./ThemeSwitch.svelte";
  import UpdateSwitch from "./UpdateSwitch.svelte";

  interface Props {
    compact?: boolean;
    styleHeight?: ComponentProps<typeof NakedButton>["styleHeight"];
    popoverProps: Partial<ComponentProps<typeof Popover>>;
  }

  const {
    compact = true,
    styleHeight = "2.5rem",
    popoverProps,
  }: Props = $props();

  let popoverExpanded: boolean = $state(false);
</script>

<Popover {...popoverProps} bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    {#if updateChecker.newVersion}
      <OutlineButton
        {styleHeight}
        id={settingsPopoverToggleId}
        title="Settings"
        {onclick}
        variant="secondary"
        active={popoverExpanded}>
        <Icon name="settings" />
        {#if !compact}
          Settings
        {/if}
      </OutlineButton>
    {:else}
      <NakedButton
        id={settingsPopoverToggleId}
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
    {/if}
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
          Version

          <div class="global-flex">
            {#if updateChecker.currentVersion}
              <CopyableId id={updateChecker.currentVersion} />
            {/if}
            {#if updateChecker.newVersion}
              -> <ExternalLink href="https://radicle.xyz/desktop">
                Update to {updateChecker.newVersion}
              </ExternalLink>
            {/if}
          </div>
        </div>

        <div
          class="global-flex"
          style:justify-content="space-between"
          style:width="100%">
          Check for updates <UpdateSwitch
            active={updateChecker.isEnabled}
            disable={updateChecker.disable}
            enable={updateChecker.enable} />
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
