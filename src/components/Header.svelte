<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, type Snippet } from "svelte";

  import * as router from "@app/lib/router";
  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";

  import Avatar from "./Avatar.svelte";
  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Inbox from "@app/views/home/Inbox.svelte";
  import NakedButton from "./NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover from "./Popover.svelte";

  interface Props {
    publicKey: string;
    center?: Snippet;
  }

  const { center, publicKey }: Props = $props();

  let notificationCount: number | undefined = $state(undefined);
  let notificationPopoverExpaneded: boolean = $state(false);

  onMount(async () => {
    await loadCounter();
  });

  dynamicInterval("auth", loadCounter, 3_000);

  async function loadCounter() {
    notificationCount = await invoke<number>("count_total_notifications");
    if (window.__TAURI_INTERNALS__) {
      await getCurrentWindow().setBadgeCount(
        notificationCount === 0 ? undefined : notificationCount,
      );
    }
  }
</script>

<style>
  .header {
    height: 3rem;
    padding: 0.5rem 1rem;
    display: flex;
    align-items: flex-start;
  }
  .header:after {
    content: " ";
    position: absolute;
    top: 0;
    left: 0.5rem;
    right: 0.5rem;
    height: 3rem;
    z-index: -1;
    background-color: var(--color-background-float);
    clip-path: var(--3px-bottom-corner-fill);
  }
  .wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    row-gap: 8px;
    z-index: 50;
  }
  .top-row {
    display: flex;
    width: 100%;
    justify-content: space-between;
  }
</style>

<div class="header global-flex">
  <div class="wrapper">
    <div class="top-row">
      <div class="global-flex" style:gap="0.25rem">
        <NakedButton
          variant="ghost"
          onclick={() => {
            void router.push({ resource: "home", activeTab: "all" });
          }}
          stylePadding="0 4px">
          <Avatar {publicKey} />
        </NakedButton>
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.back();
          }}
          stylePadding="0 4px">
          <Icon name="arrow-left" />
        </NakedButton>
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.forward();
          }}
          stylePadding="0 4px">
          <Icon name="arrow-right" />
        </NakedButton>
      </div>

      {@render center?.()}

      <div class="global-flex">
        <div
          class="global-flex txt-semibold txt-small"
          style:margin-right="0.25rem">
          {#if $nodeRunning}
            <Icon name="online" />
            Online
          {:else}
            <Icon name="offline" />
            Offline
          {/if}
        </div>

        <Popover
          popoverPositionRight="0"
          popoverPositionTop="3rem"
          bind:expanded={notificationPopoverExpaneded}>
          {#snippet toggle(onclick)}
            <OutlineButton
              {onclick}
              variant={notificationCount && notificationCount > 0
                ? "secondary"
                : "ghost"}
              active={notificationPopoverExpaneded}>
              <Icon name="inbox" />
              {#if notificationCount !== undefined && notificationCount > 0}
                {notificationCount}
              {/if}
            </OutlineButton>
          {/snippet}

          {#snippet popover()}
            <Border
              variant="ghost"
              styleWidth="40rem"
              stylePadding="1rem"
              styleAlignItems="flex-start"
              styleOverflow="scroll"
              styleHeight="calc(100vh - 4rem)">
              <Inbox {notificationCount} {loadCounter} />
            </Border>
          {/snippet}
        </Popover>
      </div>
    </div>
  </div>
</div>
