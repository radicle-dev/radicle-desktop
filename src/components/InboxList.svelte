<script lang="ts">
  import type { NotificationsByRepo } from "@bindings/cob/inbox/NotificationsByRepo";

  import {
    type FilteredRepo,
    filterNotifications,
    rowIdsOf,
  } from "@app/lib/inboxFilter";
  import { modifierKey, preserveFocus } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import ConfirmClear from "@app/components/ConfirmClear.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import NotificationsByRepoComponent from "@app/components/NotificationsByRepo.svelte";

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

  let searchInput = $state("");
  let showSearch = $state(false);
  let excludedGroupIds = $state<string[]>([]);

  const isFiltering = $derived(searchInput.trim() !== "");

  function excludeGroup(id: string) {
    if (!excludedGroupIds.includes(id)) {
      excludedGroupIds = [...excludedGroupIds, id];
    }
  }

  function resetExclusions() {
    excludedGroupIds = [];
  }

  const searchableRepos = $derived(
    isFiltering
      ? notificationsByRepo.filter(r => !hiddenRepos.includes(r.rid))
      : notificationsByRepo,
  );

  const filteredRepos = $derived(
    filterNotifications(searchableRepos, searchInput, excludedGroupIds),
  );

  const matchedTotal = $derived(
    filteredRepos.reduce((acc, r) => acc + r.count, 0),
  );

  const matchedRowIdsByRepo = $derived(
    new Map(filteredRepos.map(r => [r.repo.rid, rowIdsOf(r.groups)])),
  );

  const allMatchedRowIds = $derived([...matchedRowIdsByRepo.values()].flat());

  async function effectiveClearAll() {
    if (isFiltering) {
      const ids = allMatchedRowIds;
      await clearByIds(ids);
      searchInput = "";
      showSearch = false;
      excludedGroupIds = [];
    } else {
      await clearAll();
    }
  }

  function effectiveClearByRepo(rid: string) {
    return isFiltering
      ? clearByIds(matchedRowIdsByRepo.get(rid) ?? [])
      : clearByRepo(rid);
  }

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

  const displayRepos = $derived(
    sortedRepos(filteredRepos, pinnedRepos, hiddenRepos),
  );

  function sortedRepos(
    allRepos: FilteredRepo[],
    pinned: string[],
    hidden: string[],
  ) {
    // Preserve pinning order.
    const pinnedRepos = pinned
      .map(p => allRepos.find(r => r.repo.rid === p))
      .filter((r): r is FilteredRepo => r !== undefined);

    const sortedRepos = allRepos
      .filter(r => !pinned.includes(r.repo.rid) && !hidden.includes(r.repo.rid))
      .sort((a, b) => a.repo.name.localeCompare(b.repo.name));
    const hiddenRepos = allRepos
      .filter(r => hidden.includes(r.repo.rid))
      .sort((a, b) => a.repo.name.localeCompare(b.repo.name));

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
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }
  .header {
    font: var(--txt-heading-m);
    display: flex;
    align-items: center;
    min-height: 2rem;
  }
  .clear-inbox {
    margin-left: auto;
    visibility: hidden;
    display: flex;
    color: var(--color-text-tertiary);
  }
  .header:hover .clear-inbox {
    visibility: visible;
  }
  .filter-search {
    display: flex;
    margin-left: 0.5rem;
  }
  .filter-search.expanded {
    width: 16rem;
  }
  .repo-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 1rem;
  }
  .no-matches {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>

<div class="container">
  {#if notificationCount !== undefined && notificationCount > 0}
    <div class="header">
      <div class="global-flex">
        Inbox
        <span class="global-counter-badge">
          {isFiltering ? matchedTotal : notificationCount}
        </span>
      </div>
      {#if !isFiltering && notificationCount > loadedNotificationCount()}
        <div
          class="txt-missing txt-body-m-regular global-flex"
          style:margin-left="1rem">
          <Button variant="naked" onclick={loadNew}>
            See {notificationCount - loadedNotificationCount()} new
          </Button>
        </div>
      {/if}
      {#if isFiltering && excludedGroupIds.length > 0}
        <div
          class="txt-missing txt-body-m-regular global-flex"
          style:margin-left="1rem">
          <Button variant="naked" onclick={resetExclusions}>
            Reset {excludedGroupIds.length} excluded
          </Button>
        </div>
      {/if}
      <div class="clear-inbox" use:preserveFocus>
        {#key isFiltering}
          <ConfirmClear
            count={isFiltering ? matchedTotal : notificationCount}
            matching={isFiltering}
            clear={effectiveClearAll} />
        {/key}
      </div>
      <div class="filter-search" class:expanded={showSearch}>
        <FuzzySearch
          placeholder={`Fuzzy filter notifications ${modifierKey()} + f`}
          bind:show={showSearch}
          bind:value={searchInput} />
      </div>
    </div>
  {/if}

  {#if notificationCount !== undefined && notificationCount > 0 && isFiltering && filteredRepos.length === 0}
    <div class="no-matches">
      <div
        class="txt-missing txt-body-m-regular global-flex"
        style:gap="0.25rem">
        Filter didn't match anything
      </div>
    </div>
  {:else if notificationCount !== undefined && notificationCount > 0}
    <div class="repo-list">
      {#each displayRepos as r (r.repo.rid)}
        <NotificationsByRepoComponent
          count={r.count}
          excludeGroup={isFiltering ? excludeGroup : undefined}
          {isFiltering}
          groupedNotifications={r.groups}
          hidden={hiddenRepos.includes(r.repo.rid)}
          name={r.repo.name}
          pinned={pinnedRepos.includes(r.repo.rid)}
          rid={r.repo.rid}
          {clearByIds}
          clearByRepo={effectiveClearByRepo}
          {showAll}
          {toggleHide}
          {togglePin} />
      {/each}
    </div>
  {/if}
</div>
