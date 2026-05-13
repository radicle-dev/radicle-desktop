<script lang="ts">
  import type { NotificationsByRepo } from "@bindings/cob/inbox/NotificationsByRepo";

  import { getCurrentWindow } from "@tauri-apps/api/window";

  import { invoke } from "@app/lib/invoke";
  import { notificationCount } from "@app/lib/notificationCount.svelte";
  import type { SidebarData } from "@app/lib/router/definitions";

  import { badgeCounter } from "@app/components/BadgeCounterSwitch.svelte";
  import InboxList from "@app/components/InboxList.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Layout from "@app/views/repo/Layout.svelte";

  interface Props {
    sidebarData: SidebarData;
    notificationsByRepo: NotificationsByRepo[];
  }

  /* eslint-disable prefer-const */
  let { sidebarData, notificationsByRepo }: Props = $props();
  /* eslint-enable prefer-const */

  $effect(() => {
    notificationCount.value = sidebarData.notificationCount;
  });

  async function updateCount() {
    const count = await invoke<number>("notification_count");
    notificationCount.value = count;
    if (window.__TAURI_INTERNALS__ && $badgeCounter) {
      await getCurrentWindow().setBadgeCount(count === 0 ? undefined : count);
    } else if (window.__TAURI_INTERNALS__) {
      await getCurrentWindow().setBadgeCount(undefined);
    }
  }

  async function clearAll() {
    try {
      await invoke("clear_notifications", { params: { type: "all" } });
    } finally {
      await updateCount();
      await loadNotifications();
    }
  }

  async function clearByRepo(rid: string) {
    try {
      await invoke("clear_notifications", {
        params: { type: "repo", content: rid },
      });
    } finally {
      await updateCount();
      await loadNotifications();
    }
  }

  async function clearByIds(ids: string[]) {
    try {
      await invoke("clear_notifications", {
        params: { type: "ids", content: ids },
      });
    } finally {
      await updateCount();
      await loadNotifications();
    }
  }

  async function loadNotifications() {
    notificationsByRepo = await invoke<NotificationsByRepo[]>(
      "list_notifications",
      { params: { take: 100 } },
    );
  }

  async function showAll(rid: string) {
    const all = await invoke<NotificationsByRepo[]>("list_notifications", {
      params: { repos: [rid] },
    });
    notificationsByRepo = [
      ...notificationsByRepo.filter(r => r.rid !== rid),
      ...all,
    ];
  }
</script>

<style>
  .content {
    padding: 1rem;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }
  .inner {
    width: 100%;
    max-width: 75rem;
    margin: 0 auto;
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .inbox-zero {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }
</style>

<Layout selfScroll>
  {#if notificationCount.value === 0}
    <div class="inbox-zero">
      <div
        class="txt-missing txt-body-m-regular global-flex"
        style:gap="0.25rem">
        You're all caught up
      </div>
    </div>
  {:else}
    <ScrollArea
      style="height: 100%; width: 100%; background-color: var(--color-surface-canvas);">
      <div class="content">
        <div class="inner">
          <InboxList
            {clearAll}
            {clearByIds}
            {clearByRepo}
            loadNew={loadNotifications}
            notificationCount={notificationCount.value}
            {notificationsByRepo}
            {showAll} />
        </div>
      </div>
    </ScrollArea>
  {/if}
</Layout>
