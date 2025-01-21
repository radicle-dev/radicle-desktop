<script lang="ts">
  import type { HomeInboxTab } from "@app/lib/router/definitions";
  import type { NotificationItem } from "@bindings/cob/inbox/NotificationItem";

  import * as router from "@app/lib/router";
  import Button from "./Button.svelte";
  import NotificationTeaser from "./NotificationTeaser.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    all?: boolean;
    clearByIds: (rid: string, ids: string[]) => Promise<void>;
    clearByRepo: (rid: string) => Promise<void>;
    repo: HomeInboxTab;
    items: Record<string, NotificationItem[]>;
    more: boolean;
  }

  const {
    all = false,
    more,
    clearByRepo,
    clearByIds,
    repo,
    items,
  }: Props = $props();
</script>

<style>
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-right: 1.5rem;
  }
</style>

{#if Object.entries(items).length > 0}
  <div class="header">
    <div class="global-flex" style:margin="1rem 0">
      <span class="txt-bold">
        {repo.name}
      </span>
      {repo.count}
    </div>
    <Icon onclick={() => clearByRepo(repo.rid)} name="broom-double" />
  </div>
{/if}

<div>
  {#each Object.entries(items).sort((a, b) => b[1][0].timestamp - a[1][0].timestamp) as [_, notificationItems]}
    <NotificationTeaser
      {clearByIds}
      rid={repo.rid}
      kind={notificationItems[0].type}
      oid={notificationItems[0].id}
      {notificationItems} />
  {/each}
</div>
{#if all === false && more}
  <div style:width="100%" style:margin-top="1rem">
    <div style:width="7rem" style:margin="auto">
      <Button
        variant="ghost"
        onclick={() =>
          router.push({
            resource: "inbox",
            activeTab: repo,
          })}>
        <div style:width="100%" style:text-align="center">See all</div>
      </Button>
    </div>
  </div>
{/if}
