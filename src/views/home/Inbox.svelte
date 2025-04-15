<script lang="ts">
  import type { NotificationCount } from "@bindings/cob/inbox/NotificationCount";

  import { invoke } from "@app/lib/invoke";

  import ConfirmClear from "@app/components/ConfirmClear.svelte";
  import RepoNotifications from "@app/components/RepoNotifications.svelte";
  import { onMount } from "svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    notificationCount: number | undefined;
    loadCounter: () => Promise<void>;
  }

  const { notificationCount, loadCounter }: Props = $props();
  let repoNotificationCounts: NotificationCount[] = $state([]);

  async function clearAll() {
    try {
      await invoke("clear_notifications", {
        params: { type: "all" },
      });
    } catch (error) {
      console.error("Clearing notifications failed", error);
    } finally {
      await loadCounter();
      await loadRepoCounters();
    }
  }

  async function loadRepoCounters() {
    repoNotificationCounts = await invoke<NotificationCount[]>(
      "count_notifications_by_repo",
    );
  }

  onMount(async () => {
    await loadRepoCounters();
  });
</script>

<style>
  .container {
    width: 100%;
  }
  .header {
    font-weight: var(--font-weight-medium);
    font-size: var(--font-size-medium);
    display: flex;
    align-items: center;
  }
</style>

<div class="container">
  <div class="header">
    <div>
      Inbox
      {#if notificationCount !== undefined && notificationCount > 0}
        {notificationCount}
      {/if}
    </div>
    {#if notificationCount && notificationCount > 1}
      <div style:margin-left="auto" style:margin-right="1rem">
        <ConfirmClear count={notificationCount} clear={clearAll} />
      </div>
    {/if}
  </div>

  {#each repoNotificationCounts as { name, count, rid }}
    <RepoNotifications {name} {rid} {count} {loadCounter} {loadRepoCounters} />
  {:else}
    <div
      class="global-flex"
      style:height="100%"
      style:align-items="center"
      style:justify-content="center">
      <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
        <Icon name="none" />
        Yay, inbox zero!
      </div>
    </div>
  {/each}
</div>
