<script lang="ts">
  import type { NotificationItem } from "@bindings/cob/inbox/NotificationItem";

  import { invoke } from "@app/lib/invoke";
  import { onMount } from "svelte";

  import Button from "./Button.svelte";
  import ConfirmClear from "./ConfirmClear.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NotificationTeaser from "./NotificationTeaser.svelte";

  interface Props {
    rid: string;
    name: string;
    count: number;
    loadCounter: () => Promise<void>;
    loadRepoCounters: () => Promise<void>;
  }

  const { rid, name, count, loadCounter, loadRepoCounters }: Props = $props();

  let notifications: NotificationItem[][] = $state([]);
  let all = $state(false);

  onMount(async () => {
    await getNotifications(rid, all);
  });

  async function clearByRepo(rid: string) {
    try {
      await invoke("clear_notifications", {
        params: { type: "repo", content: rid },
      });
      await loadCounter();
      await loadRepoCounters();
      await getNotifications(rid, all);
    } catch (error) {
      console.error("Clearing notifications failed", error);
    }
  }

  async function clearByIds(ids: string[]) {
    try {
      await invoke("clear_notifications", {
        params: { type: "ids", content: ids },
      });
      await loadCounter();
      await loadRepoCounters();
      await getNotifications(rid, all);
    } catch (error) {
      console.error("Clearing notifications failed", error);
    }
  }

  async function getNotifications(rid: string, all: boolean) {
    notifications = await invoke<NotificationItem[][]>("list_notifications", {
      params: {
        repo: rid,
        all,
        take: 100,
      },
    });
  }
</script>

<style>
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-right: 1rem;
  }
  .container {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>

<div class="header">
  <div class="global-flex" style:margin="1rem 0">
    <span class="txt-bold">
      {name}
    </span>
    {count}
  </div>
  {#if count > 1}
    <ConfirmClear
      {count}
      clear={() => {
        void clearByRepo(rid);
      }} />
  {/if}
</div>

<div class="container">
  {#if notifications.length > 0}
    {#each notifications as not}
      <NotificationTeaser
        {clearByIds}
        {rid}
        kind={not[0].type}
        oid={not[0].id}
        notificationItems={not} />
    {/each}
  {:else}
    <div
      class="global-flex"
      style:height="100%"
      style:align-items="center"
      style:justify-content="center">
      <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
        <Icon name="none" />
        No notifications.
      </div>
    </div>
  {/if}
</div>

{#if notifications.length > 0 && notifications.length < count}
  <div style:width="100%" style:margin-top="1rem">
    <div style:width="7rem" style:margin="auto">
      <Button
        variant="ghost"
        onclick={async () => {
          all = true;
          await getNotifications(rid, all);
        }}>
        <div style:width="100%" style:text-align="center">Show all</div>
      </Button>
    </div>
  </div>
{/if}
