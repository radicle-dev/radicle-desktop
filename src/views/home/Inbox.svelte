<script lang="ts">
  import type { NotificationsByRepo } from "@bindings/cob/inbox/NotificationsByRepo";

  import ConfirmClear from "@app/components/ConfirmClear.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NotificationsByRepoComponent from "@app/components/NotificationsByRepo.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  interface Props {
    clearAll: () => Promise<void>;
    clearByIds: (ids: string[]) => Promise<void>;
    clearByRepo: (rid: string) => Promise<void>;
    loadNew: () => Promise<void>;
    notificationCount: number | undefined;
    notificationsByRepo: NotificationsByRepo[];
    showAll: (rid: string) => Promise<void>;
  }

  const {
    clearAll,
    clearByIds,
    clearByRepo,
    loadNew,
    notificationCount,
    notificationsByRepo,
    showAll,
  }: Props = $props();

  let pinnedRepos: string[] = $state(loadPinnedRepos());
  let hiddenRepos: string[] = $state(loadHiddenRepos());

  function loadPinnedRepos(): string[] {
    const storedPinnedRepos = localStorage
      ? localStorage.getItem("pinnedInboxRepos")
      : null;

    if (storedPinnedRepos === null) {
      return [];
    } else {
      return JSON.parse(storedPinnedRepos);
    }
  }

  function updatePinnedRepos(newRepos: string[]) {
    pinnedRepos = newRepos;
    localStorage.setItem("pinnedInboxRepos", JSON.stringify(newRepos));
  }

  function togglePin(rid: string) {
    const repos = loadPinnedRepos();
    if (repos.includes(rid)) {
      updatePinnedRepos(repos.filter(r => r !== rid));
    } else {
      updatePinnedRepos([rid, ...repos]);
    }
  }

  function loadHiddenRepos(): string[] {
    const storedHiddenRepos = localStorage
      ? localStorage.getItem("hiddenInboxRepos")
      : null;

    if (storedHiddenRepos === null) {
      return [];
    } else {
      return JSON.parse(storedHiddenRepos);
    }
  }

  function updateHiddenRepos(newRepos: string[]) {
    hiddenRepos = newRepos;
    localStorage.setItem("hiddenInboxRepos", JSON.stringify(newRepos));
  }

  function toggleHide(rid: string) {
    const repos = loadHiddenRepos();
    if (repos.includes(rid)) {
      updateHiddenRepos(repos.filter(r => r !== rid));
    } else {
      updateHiddenRepos([rid, ...repos]);
    }
  }

  function sortedRepos(
    allRepos: NotificationsByRepo[],
    pinned: string[],
    hidden: string[],
  ) {
    // Preserve pinning order.
    const pinnedRepos = pinned
      .map(p => allRepos.find(r => r.rid === p))
      .filter((repo): repo is NotificationsByRepo => repo !== undefined);

    const sortedRepos = allRepos
      .filter(r => !pinned.includes(r.rid) && !hidden.includes(r.rid))
      .sort((a, b) => a.name.localeCompare(b.name));
    const hiddenRepos = allRepos
      .filter(r => hidden.includes(r.rid))
      .sort((a, b) => a.name.localeCompare(b.name));

    return [...pinnedRepos, ...sortedRepos, ...hiddenRepos];
  }

  function loadedNotificationCount() {
    return notificationsByRepo.reduce((acc, repo) => {
      return acc + repo.count;
    }, 0);
  }
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
    min-height: 2rem;
  }
  .clear-inbox {
    margin-left: auto;
    margin-right: 1rem;
    display: none;
  }
  .header:hover .clear-inbox {
    display: flex;
  }
  .repo-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 1rem;
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
    {#if notificationCount === undefined || notificationCount === 0}
      <div
        class="txt-missing txt-small global-flex"
        style:gap="0.25rem"
        style:margin-left="auto">
        <Icon name="thumb-up" />
        Yay, inbox zero!
      </div>
    {/if}
    {#if notificationCount !== undefined && notificationCount > loadedNotificationCount()}
      <div class="txt-missing txt-small global-flex" style:margin-left="1rem">
        <NakedButton variant="ghost" onclick={loadNew}>
          See {notificationCount - loadedNotificationCount()} new
        </NakedButton>
      </div>
    {/if}
    {#if notificationCount && notificationCount > 0}
      <div class="clear-inbox">
        <ConfirmClear count={notificationCount} clear={clearAll} />
      </div>
    {/if}
  </div>

  {#if notificationCount !== undefined && notificationCount > 0}
    <div class="repo-list">
      {#each sortedRepos(notificationsByRepo, pinnedRepos, hiddenRepos) as repo}
        <NotificationsByRepoComponent
          count={repo.count}
          groupedNotifications={repo.notifications}
          hidden={hiddenRepos.includes(repo.rid)}
          name={repo.name}
          pinned={pinnedRepos.includes(repo.rid)}
          rid={repo.rid}
          {clearByIds}
          {clearByRepo}
          {showAll}
          {toggleHide}
          {togglePin} />
      {/each}
    </div>
  {/if}
</div>
