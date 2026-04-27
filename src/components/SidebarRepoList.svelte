<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { RepoSummary } from "@bindings/repo/RepoSummary";

  import { onMount } from "svelte";
  import { boolean } from "zod";

  import { nodeRunning } from "@app/lib/events";
  import { dynamicInterval, resetDynamicInterval } from "@app/lib/interval";
  import { cachedRepoCommitCount, invoke } from "@app/lib/invoke";
  import * as router from "@app/lib/router";
  import useLocalStorage from "@app/lib/useLocalStorage.svelte";
  import { formatRepositoryId } from "@app/lib/utils";

  import AddRepoButton from "@app/components/AddRepoButton.svelte";
  import Clipboard from "@app/components/Clipboard.svelte";
  import Icon from "@app/components/Icon.svelte";
  import RepoAvatar from "@app/components/RepoAvatar.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";

  interface Props {
    initialRepos: RepoSummary[];
    initialSeededNotReplicated: string[];
    activeRepo?: RepoInfo;
  }

  const {
    initialRepos,
    initialSeededNotReplicated,
    activeRepo = undefined,
  }: Props = $props();

  let repos = $state<RepoSummary[]>(initialRepos);
  let seededNotReplicated = $state<string[]>(initialSeededNotReplicated);
  let activeCommitCount = $state<number | undefined>(undefined);
  let filterOpen = $state(false);
  let filterQuery = $state("");
  let filterInputElement: HTMLInputElement | undefined = $state(undefined);

  $effect(() => {
    repos = initialRepos;
  });

  $effect(() => {
    seededNotReplicated = initialSeededNotReplicated;
  });

  $effect(() => {
    const rid = activeRepo?.rid;
    const head = activeRepo?.payloads["xyz.radicle.project"]?.meta.head;

    activeCommitCount = undefined;

    if (!rid || !head) return;

    void cachedRepoCommitCount(rid, head)
      .then(count => {
        if (activeRepo?.rid === rid) {
          activeCommitCount = count;
        }
      })
      .catch(error => {
        console.error("Failed to load commit count", error);
      });
  });

  $effect(() => {
    if (filterOpen && filterInputElement) {
      filterInputElement.focus({ preventScroll: true });
    }
  });

  $effect(() => {
    if (seededNotReplicated.length > 0) {
      dynamicInterval("seededNotReplicated", reloadRepos, 5_000);
    } else {
      resetDynamicInterval("seededNotReplicated");
    }
  });

  onMount(() => {
    return () => resetDynamicInterval("seededNotReplicated");
  });

  const filteredRepos = $derived(
    filterQuery.trim()
      ? repos.filter(r =>
          r.name.toLowerCase().includes(filterQuery.trim().toLowerCase()),
        )
      : repos,
  );

  const reposExpanded = useLocalStorage(
    "sidebarReposExpanded",
    boolean(),
    true,
    !window.localStorage,
  );

  const fetchingExpanded = useLocalStorage(
    "sidebarFetchingExpanded",
    boolean(),
    true,
    !window.localStorage,
  );

  async function reloadRepos() {
    [repos, seededNotReplicated] = await Promise.all([
      invoke<RepoSummary[]>("list_repos_summary"),
      invoke<string[]>("seeded_not_replicated"),
    ]);
  }

  async function unseed(rid: string) {
    try {
      await invoke<null>("unseed", { rid });
      await reloadRepos();
    } catch (error) {
      console.error("Unseed failed", error);
    }
  }

  const activeRoute = router.activeRouteStore;

  function activeRid(): string | undefined {
    return activeRepo?.rid;
  }

  function isRepoHome(rid: string): boolean {
    return $activeRoute.resource === "repo.home" && activeRid() === rid;
  }

  function isIssues(rid: string): boolean {
    return (
      ($activeRoute.resource === "repo.issues" ||
        $activeRoute.resource === "repo.issue") &&
      activeRid() === rid
    );
  }

  function isCommits(rid: string): boolean {
    return (
      ($activeRoute.resource === "repo.commits" ||
        $activeRoute.resource === "repo.commit") &&
      activeRid() === rid
    );
  }

  function isPatches(rid: string): boolean {
    return (
      ($activeRoute.resource === "repo.patches" ||
        $activeRoute.resource === "repo.patch") &&
      activeRid() === rid
    );
  }
</script>

<style>
  .repos-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.5rem 0;
  }
  .section-header {
    font: var(--txt-body-m-regular);
    font-variant-ligatures: none;
    color: var(--color-text-secondary);
    padding: 0.5rem 0 0.25rem 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    justify-content: space-between;
    cursor: pointer;
    user-select: none;
  }
  .section-header-label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex: 1;
    min-width: 0;
  }
  .section-header-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    border-radius: var(--border-radius-sm);
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    cursor: pointer;
    width: 100%;
    text-decoration: none;
  }
  .nav-item:hover {
    background-color: var(--color-surface-subtle);
  }
  .nav-item.active {
    background-color: var(--color-surface-subtle);
  }
  .nav-item .global-counter-badge {
    margin-left: auto;
  }
  .sub-item {
    padding-left: 2rem;
  }
  .pending-item {
    color: var(--color-text-secondary);
    cursor: default;
  }
  .pending-avatar {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
    border: 1px solid var(--color-border-subtle);
  }
  .pending-item .remove-icon {
    display: none;
    margin-left: auto;
    color: var(--color-text-tertiary);
    border-radius: var(--border-radius-sm);
  }
  .pending-item:hover .remove-icon {
    display: flex;
  }
  .pending-item .remove-icon:hover {
    background-color: var(--color-surface-mid);
  }
  .nav-item .copy-rid {
    visibility: hidden;
    margin-left: auto;
    color: var(--color-text-tertiary);
    border-radius: var(--border-radius-sm);
  }
  .nav-item:hover .copy-rid {
    visibility: visible;
  }
  .nav-item .copy-rid:hover {
    background-color: var(--color-surface-mid);
  }
  .filter-button {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 0;
    padding: 0.125rem;
    margin-left: -0.125rem;
    border-radius: var(--border-radius-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }
  .filter-button:hover {
    color: var(--color-text-primary);
    background-color: var(--color-surface-subtle);
  }
  .filter-input {
    background: none;
    border: 0;
    outline: none;
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    flex: 1;
    min-width: 0;
  }
  .filter-input::placeholder {
    color: var(--color-text-secondary);
  }
  .icon {
    color: var(--color-text-tertiary);
  }
</style>

{#if seededNotReplicated.length > 0}
  <div
    class="section-header"
    onclick={() => (fetchingExpanded.value = !fetchingExpanded.value)}
    role="button"
    tabindex="0"
    onkeydown={e => {
      if (e.key === "Enter" || e.key === " ") {
        fetchingExpanded.value = !fetchingExpanded.value;
      }
    }}>
    <span class="section-header-label">
      <span class="icon"><Icon name="hourglass" /></span>
      Fetching {seededNotReplicated.length > 1
        ? ` (${seededNotReplicated.length})`
        : ""}
      <span class="icon">
        <Icon name={fetchingExpanded.value ? "chevron-down" : "chevron-up"} />
      </span>
    </span>
  </div>

  {#if fetchingExpanded.value}
    <div style:display="flex" style:flex-direction="column" style:gap="0.25rem">
      {#each seededNotReplicated as rid (rid)}
        <div
          class="nav-item pending-item"
          title="{$nodeRunning ? 'Fetching' : 'Queued'} {rid}">
          <span class="pending-avatar"></span>
          <span class="txt-overflow">{formatRepositoryId(rid)}</span>
          <button
            class="remove-icon filter-button"
            title="Remove"
            onclick={() => unseed(rid)}>
            <span class="icon"><Icon name="trash" /></span>
          </button>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<div
  class="section-header"
  onclick={() => {
    if (!filterOpen) {
      reposExpanded.value = !reposExpanded.value;
    }
  }}
  role="button"
  tabindex="0"
  onkeydown={e => {
    if (e.key === "Enter" || e.key === " ") {
      if (!filterOpen) {
        reposExpanded.value = !reposExpanded.value;
      }
    }
  }}>
  {#if filterOpen}
    <span
      class="section-header-label"
      onclick={e => e.stopPropagation()}
      role="none">
      <button
        class="filter-button"
        title="Clear filter"
        onclick={() => {
          filterOpen = false;
          filterQuery = "";
        }}>
        <span class="icon"><Icon name="search" /></span>
      </button>
      <input
        bind:this={filterInputElement}
        class="filter-input"
        placeholder="Filter repos…"
        bind:value={filterQuery}
        onkeydown={e => {
          if (e.key === "Escape") {
            filterOpen = false;
            filterQuery = "";
          } else if (e.key === "Enter" && filteredRepos.length > 0) {
            void router.push({
              resource: "repo.home",
              rid: filteredRepos[0].rid,
            });
            filterQuery = "";
          }
        }} />
    </span>
  {:else}
    <span class="section-header-label">
      <span onclick={e => e.stopPropagation()} role="none">
        <button
          class="filter-button"
          title="Filter repos"
          aria-keyshortcuts="ctrl+f"
          onclick={() => {
            filterOpen = true;
            reposExpanded.value = true;
          }}>
          <span class="icon"><Icon name="filter" /></span>
        </button>
      </span>
      All Repos{repos.length > 1 ? ` (${repos.length})` : ""}
      <span class="icon">
        <Icon name={reposExpanded.value ? "chevron-down" : "chevron-up"} />
      </span>
    </span>
  {/if}
  <span class="section-header-actions">
    <span onclick={e => e.stopPropagation()} role="none">
      <AddRepoButton
        onOpen={() => (reposExpanded.value = true)}
        reload={reloadRepos}
        {repos}
        {seededNotReplicated} />
    </span>
  </span>
</div>

{#if reposExpanded.value}
  <ScrollArea
    style="flex: 1; min-height: 0; mask-image: linear-gradient(to bottom, transparent 0, black 0.5rem, black calc(100% - 0.5rem), transparent 100%);">
    <div class="repos-list">
      {#each filteredRepos as repo (repo.rid)}
        <a
          class="nav-item"
          class:active={isRepoHome(repo.rid)}
          href={router.routeToPath({ resource: "repo.home", rid: repo.rid })}>
          <RepoAvatar name={repo.name} rid={repo.rid} styleWidth="1rem" />
          <span class="txt-overflow">{repo.name}</span>
          <span
            class="copy-rid"
            role="none"
            title="Copy RID"
            onclick={e => {
              e.preventDefault();
              e.stopPropagation();
            }}>
            <Clipboard text={repo.rid} noPopover />
          </span>
        </a>
        {#if activeRid() === repo.rid}
          {@const activeProject = activeRepo?.payloads["xyz.radicle.project"]}
          <a
            class="nav-item sub-item"
            class:active={isCommits(repo.rid)}
            href={router.routeToPath({
              resource: "repo.commits",
              rid: repo.rid,
            })}>
            <span class="icon"><Icon name="branch" /></span>
            Commits
            {#if activeCommitCount !== undefined}
              <span class="global-counter-badge">{activeCommitCount}</span>
            {/if}
          </a>
          <a
            class="nav-item sub-item"
            class:active={isIssues(repo.rid)}
            href={router.routeToPath({
              resource: "repo.issues",
              rid: repo.rid,
              status: "open",
            })}>
            <span class="icon"><Icon name="issue" /></span>
            Issues
            {#if activeProject && activeProject.meta.issues.open > 0}
              <span class="global-counter-badge">
                {activeProject.meta.issues.open}
              </span>
            {/if}
          </a>
          <a
            class="nav-item sub-item"
            class:active={isPatches(repo.rid)}
            href={router.routeToPath({
              resource: "repo.patches",
              rid: repo.rid,
              status: "open",
            })}>
            <span class="icon"><Icon name="patch" /></span>
            Patches
            {#if activeProject && activeProject.meta.patches.open > 0}
              <span class="global-counter-badge">
                {activeProject.meta.patches.open}
              </span>
            {/if}
          </a>
        {/if}
      {/each}
    </div>
  </ScrollArea>
{/if}
