<script lang="ts">
  import type { NotificationsByRepo } from "@bindings/cob/inbox/NotificationsByRepo";

  import Button from "./Button.svelte";
  import ConfirmClear from "./ConfirmClear.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NotificationTeaser from "./NotificationTeaser.svelte";
  import NakedButton from "./NakedButton.svelte";

  interface Props {
    clearByIds: (ids: string[]) => Promise<void>;
    clearByRepo: (rid: string) => Promise<void>;
    count: number;
    groupedNotifications: NotificationsByRepo["notifications"];
    hidden: boolean;
    name: string;
    pinned: boolean;
    rid: string;
    showAll: (rid: string) => Promise<void>;
    toggleHide: (rid: string) => void;
    togglePin: (rid: string) => void;
  }

  const {
    clearByIds,
    clearByRepo,
    count,
    groupedNotifications,
    hidden,
    name,
    pinned,
    rid,
    showAll,
    toggleHide,
    togglePin,
  }: Props = $props();
</script>

<style>
  .header {
    display: flex;
    align-items: center;
    padding-right: 1rem;
    width: 100%;
    min-height: 2rem;
    gap: 0.75rem;
  }
  .container {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .action-buttons {
    display: flex;
    gap: 0.25rem;
  }
  .clear-repo {
    margin-left: auto;
  }
  .action-buttons,
  .clear-repo {
    display: none;
  }
  .header:hover .action-buttons,
  .header:hover .clear-repo {
    display: flex;
  }
</style>

<div>
  <div
    class="header"
    class:txt-missing={hidden}
    style:margin-bottom={!hidden ? "1rem" : undefined}>
    <span class="txt-bold">
      {name}
    </span>
    {count}
    <div
      class="action-buttons"
      style:display={pinned || hidden ? "flex" : undefined}>
      {#if !hidden}
        <NakedButton
          variant="ghost"
          stylePadding="0 0.25rem"
          onclick={() => {
            togglePin(rid);
          }}>
          <Icon name={pinned ? "pin" : "pin-hollow"} />
        </NakedButton>
      {/if}
      {#if !pinned}
        <NakedButton
          variant="ghost"
          stylePadding="0 0.25rem"
          onclick={() => {
            toggleHide(rid);
          }}>
          <Icon name={hidden ? "eye-closed" : "eye"} />
        </NakedButton>
      {/if}
    </div>
    {#if count > 0 && !hidden}
      <div class="clear-repo">
        <ConfirmClear
          {count}
          clear={() => {
            void clearByRepo(rid);
          }} />
      </div>
    {/if}
  </div>

  {#if !hidden}
    <div class="container">
      {#if groupedNotifications.length > 0}
        {#each groupedNotifications as notificationGroup}
          <NotificationTeaser
            {clearByIds}
            {rid}
            kind={notificationGroup[0].type}
            oid={notificationGroup[0].id}
            notificationItems={notificationGroup} />
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

    {#if groupedNotifications.length > 0 && groupedNotifications.length < count}
      <div style:width="100%" style:margin-top="1rem">
        <div style:width="7rem" style:margin="auto">
          <Button
            variant="ghost"
            onclick={async () => {
              await showAll(rid);
            }}>
            <div style:width="100%" style:text-align="center">Show all</div>
          </Button>
        </div>
      </div>
    {/if}
  {/if}
</div>
