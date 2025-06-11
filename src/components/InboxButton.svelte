<script lang="ts">
  import type { NotificationsByRepo } from "@bindings/cob/inbox/NotificationsByRepo";

  import { getCurrentWindow } from "@tauri-apps/api/window";

  import { onMount } from "svelte";
  import { useOverlayScrollbars } from "overlayscrollbars-svelte";

  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Inbox from "./Inbox.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover from "./Popover.svelte";

  interface Props {
    notificationCount: number;
  }

  let { notificationCount }: Props = $props();

  let borderComponent: HTMLElement | undefined = $state();
  let notificationPopoverExpaneded: boolean = $state(false);
  let buttonActive: boolean = $state(false);

  $effect(() => {
    if (notificationPopoverExpaneded === false) {
      buttonActive = false;
    }
  });

  $effect(() => {
    if (borderComponent) {
      const [initialize] = useOverlayScrollbars({
        options: () => ({
          scrollbars: { theme: "os-theme-radicle", autoHide: "scroll" },
        }),
        defer: true,
      });

      initialize({ target: borderComponent });
    }
  });

  onMount(async () => {
    await loadCounter();
  });

  dynamicInterval("auth", loadCounter, 3_000);

  async function loadCounter() {
    notificationCount = await invoke<number>("notification_count");
    if (window.__TAURI_INTERNALS__) {
      await getCurrentWindow().setBadgeCount(
        notificationCount === 0 ? undefined : notificationCount,
      );
    }
  }

  let notificationsByRepo: NotificationsByRepo[] = $state([]);

  async function loadNotifications() {
    notificationsByRepo = await invoke<NotificationsByRepo[]>(
      "list_notifications",
      { params: { take: 100 } },
    );
  }

  async function clearAll() {
    try {
      await invoke("clear_notifications", {
        params: { type: "all" },
      });
    } catch (error) {
      console.error("Clearing notifications failed", error);
    } finally {
      await loadCounter();
      await loadNotifications();
    }
  }

  async function clearByRepo(rid: string) {
    try {
      await invoke("clear_notifications", {
        params: { type: "repo", content: rid },
      });
    } catch (error) {
      console.error("Clearing notifications failed", error);
    } finally {
      await loadCounter();
      await loadNotifications();
    }
  }

  async function clearByIds(ids: string[]) {
    try {
      await invoke("clear_notifications", {
        params: { type: "ids", content: ids },
      });
    } catch (error) {
      console.error("Clearing notifications failed", error);
    } finally {
      await loadCounter();
      await loadNotifications();
    }
  }

  async function showAll(rid: string) {
    const allNotificationsForRepo = await invoke<NotificationsByRepo[]>(
      "list_notifications",
      { params: { repos: [rid] } },
    );
    notificationsByRepo = [
      ...notificationsByRepo.filter(r => r.rid !== rid),
      ...allNotificationsForRepo,
    ];
  }
</script>

<Popover
  popoverPositionRight="0"
  popoverPositionTop="3rem"
  bind:expanded={notificationPopoverExpaneded}>
  {#snippet toggle(onclick)}
    <OutlineButton
      onclick={async () => {
        buttonActive = true;
        await loadNotifications();
        onclick();
      }}
      variant={notificationCount && notificationCount > 0
        ? "secondary"
        : "ghost"}
      active={buttonActive}>
      <Icon name="inbox" />
      {#if notificationCount !== undefined && notificationCount > 0}
        {notificationCount}
      {/if}
    </OutlineButton>
  {/snippet}

  {#snippet popover()}
    <Border
      bind:innerElement={borderComponent}
      variant="ghost"
      styleWidth="40rem"
      stylePadding="1rem"
      styleAlignItems="flex-start"
      styleOverflow="auto"
      styleMaxHeight="calc(100vh - 5rem)">
      <Inbox
        {clearAll}
        {clearByIds}
        {clearByRepo}
        loadNew={loadNotifications}
        {notificationCount}
        {notificationsByRepo}
        {showAll} />
    </Border>
  {/snippet}
</Popover>
