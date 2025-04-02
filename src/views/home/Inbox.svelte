<script lang="ts">
  import type { HomeInboxTab } from "@app/lib/router/definitions";
  import type { Config } from "@bindings/config/Config";
  import type { NotificationCount } from "@bindings/cob/inbox/NotificationCount";
  import type { NotificationItem } from "@bindings/cob/inbox/NotificationItem";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { RepoCount } from "@bindings/repo/RepoCount";

  import { SvelteMap } from "svelte/reactivity";
  import { invoke } from "@app/lib/invoke";
  import * as router from "@app/lib/router";

  import CopyableId from "@app/components/CopyableId.svelte";
  import HomeSidebar from "@app/components/HomeSidebar.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Layout from "@app/views/repo/Layout.svelte";
  import RepoNotifications from "@app/components/RepoNotifications.svelte";
  import Border from "@app/components/Border.svelte";

  interface Props {
    activeTab?: HomeInboxTab;
    notificationCount: SvelteMap<string, NotificationCount>;
    notifications: SvelteMap<
      string,
      {
        repo: HomeInboxTab;
        items: Record<string, NotificationItem[]>;
        pagination: { cursor: number; more: boolean };
      }
    >;
    repoCount: RepoCount;
    config: Config;
  }

  /* eslint-disable prefer-const */
  let {
    notificationCount,
    repoCount,
    activeTab,
    config,
    notifications,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let cursor: number | undefined = undefined;
  let more: boolean | undefined = undefined;

  // If we are focused on a repo populate the pagination vars.
  $effect(() => {
    if (activeTab && notifications.has(activeTab.rid)) {
      const n = notifications.get(activeTab.rid);
      cursor = n!.pagination.cursor;
      more = n!.pagination.more;
    }
  });

  async function clearAll() {
    try {
      await invoke("clear_notifications", {
        params: { type: "all" },
      });
    } catch (error) {
      console.error("Clearing notifications failed", error);
    } finally {
      notificationCount.clear();
      notifications.clear();
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
      await reload([rid]);
    }
  }

  async function clearByIds(rid: string, ids: string[]) {
    try {
      await invoke("clear_notifications", {
        params: { type: "ids", content: ids },
      });
    } catch (error) {
      console.error("Clearing notifications failed", error);
    } finally {
      await reload([rid]);
    }
  }

  async function reload(rids: string[]) {
    for (const rid of rids) {
      const [n, count] = await Promise.all([
        invoke<PaginatedQuery<Record<string, NotificationItem[]>>>(
          "list_notifications",
          {
            params: {
              repo: rid,
            },
          },
        ),
        invoke<Record<string, NotificationCount>>(
          "count_notifications_by_repo",
        ),
      ]);
      notificationCount = new SvelteMap(Object.entries(count));

      notifications.set(rid, {
        repo: notificationCount.get(rid)!,
        items: n.content,
        pagination: { cursor: n.cursor, more: n.more },
      });

      // If we are looking at a single repo and there are no more notifications left after a reload, push the user to the general inbox
      if (activeTab && Object.values(n.content).length === 0) {
        void router.push({ resource: "inbox" });
      }
    }
  }

  async function loadMoreContent() {
    if (more && activeTab) {
      const c = cursor ? cursor : 20;
      const p = await invoke<
        PaginatedQuery<Record<string, NotificationItem[]>>
      >("list_notifications", {
        params: {
          repo: activeTab.rid,
          skip: more ? c + 20 : c,
          take: 20,
        },
      });

      cursor = p.cursor;
      more = p.more;

      const currentNotifications = notifications.get(activeTab.rid);
      notifications.set(activeTab.rid, {
        repo: currentNotifications!.repo,
        items: { ...currentNotifications!.items, ...p.content },
        pagination: { cursor: p.cursor, more: p.more },
      });
    }
  }
</script>

<style>
  .container {
    padding: 1rem 1rem 1rem 0;
  }
  .header {
    font-weight: var(--font-weight-medium);
    font-size: var(--font-size-medium);
    display: flex;
    justify-content: space-between;
    padding-right: 1.5rem;
    align-items: center;
    min-height: 2.5rem;
  }
</style>

<Layout
  loadMoreContent={async () => {
    if (activeTab) {
      await loadMoreContent();
    }
  }}
  hideSidebar
  styleSecondColumnOverflow="visible"
  publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={config.publicKey} />
  {/snippet}
  {#snippet secondColumn()}
    <HomeSidebar
      activeTab={{ type: "inbox", repo: activeTab }}
      {notificationCount}
      {repoCount} />
  {/snippet}
  <div class="container">
    <div class="header">
      <div>Inbox</div>
      {#if notifications.size > 0}
        <Icon onclick={clearAll} name="broom-double" />
      {/if}
    </div>
    {#each notifications.values() as { repo, pagination, items }}
      <RepoNotifications
        all={Boolean(activeTab)}
        {clearByIds}
        {clearByRepo}
        {repo}
        more={pagination.more}
        {items} />
    {:else}
      <Border
        variant="ghost"
        styleAlignItems="center"
        styleJustifyContent="center">
        <div
          class="global-flex"
          style:height="74px"
          style:justify-content="center">
          <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
            <Icon name="none" />
            No notifications.
          </div>
        </div>
      </Border>
    {/each}
  </div>
</Layout>
