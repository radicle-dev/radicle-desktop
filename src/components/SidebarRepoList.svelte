<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { RepoSummary } from "@bindings/repo/RepoSummary";

  import { onMount } from "svelte";
  import { flip } from "svelte/animate";
  import { crossfade } from "svelte/transition";
  import { array, boolean, string } from "zod";

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
  import usePinnedDragReorder, {
    DRAG_RID_ATTRIBUTE,
    PINNED_LIST_CLASS,
  } from "@app/components/usePinnedDragReorder.svelte";

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

  let repos: RepoSummary[] = $derived(initialRepos);
  let seededNotReplicated: string[] = $derived(initialSeededNotReplicated);
  let activeCommitCount = $state<number | undefined>(undefined);
  let filterOpen = $state(false);
  let filterQuery = $state("");
  let filterInputElement: HTMLInputElement | undefined = $state(undefined);

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

  const pinnedRepoIds = useLocalStorage<string[]>(
    "sidebarPinnedRepos",
    array(string()),
    [],
    !window.localStorage,
  );

  const pinnedRepos = $derived.by(() => {
    const byRid = new Map(repos.map(r => [r.rid, r]));
    return pinnedRepoIds.value
      .map(rid => byRid.get(rid))
      .filter((r): r is RepoSummary => r !== undefined);
  });

  const unpinnedFilteredRepos = $derived(
    filteredRepos.filter(r => !pinnedRepoIds.value.includes(r.rid)),
  );

  const unpinnedReposCount = $derived(
    repos.filter(r => !pinnedRepoIds.value.includes(r.rid)).length,
  );

  const ANIMATION_DURATION_MS = 220;
  let animatingPinnedList = $state(false);
  let animationTimeout: ReturnType<typeof setTimeout> | undefined;
  const animationDuration = $derived(
    animatingPinnedList ? ANIMATION_DURATION_MS : 0,
  );
  const [send, receive] = crossfade({
    duration: ANIMATION_DURATION_MS,
    fallback: () => ({ duration: 0 }),
  });

  function withPinAnimation(fn: () => void) {
    animatingPinnedList = true;
    if (animationTimeout !== undefined) clearTimeout(animationTimeout);
    animationTimeout = setTimeout(() => {
      animatingPinnedList = false;
    }, ANIMATION_DURATION_MS);
    fn();
  }

  function togglePin(rid: string) {
    withPinAnimation(() => {
      if (pinnedRepoIds.value.includes(rid)) {
        pinnedRepoIds.value = pinnedRepoIds.value.filter(r => r !== rid);
      } else {
        pinnedRepoIds.value = [rid, ...pinnedRepoIds.value];
      }
    });
  }

  const drag = usePinnedDragReorder({
    pinnedRepos: () => pinnedRepos,
    getOrder: () => pinnedRepoIds.value,
    setOrder: rids => withPinAnimation(() => (pinnedRepoIds.value = rids)),
  });

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
  .pinned-list.empty {
    height: 0;
    padding: 0;
    overflow: visible;
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

  .nav-item {
    position: relative;
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
    user-select: none;
    -webkit-user-select: none;
  }
  .nav-item :global(img),
  .nav-item :global(svg) {
    -webkit-user-drag: none;
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

  .nav-item .row-actions {
    visibility: hidden;
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.125rem;
    color: var(--color-text-tertiary);
  }
  .nav-item:hover .row-actions,
  .nav-item .row-actions:has(:focus-visible) {
    visibility: visible;
  }
  .nav-item .row-actions :global(.clipboard),
  .nav-item .row-actions .pin-button,
  .nav-item .row-actions .drag-handle {
    width: 1.5rem;
    height: 1.5rem;
    border-radius: var(--border-radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .nav-item .row-actions :global(.clipboard):hover,
  .nav-item .row-actions .pin-button:hover {
    background-color: var(--color-surface-mid);
  }
  .pin-button {
    background: none;
    border: 0;
    color: inherit;
    cursor: pointer;
  }
  .drag-handle {
    cursor: grab;
  }

  .repo-row-group {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .repo-row-group.drop-before::before,
  .repo-row-group.drop-after::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background-color: var(--color-border-mid);
    border-radius: 1px;
    pointer-events: none;
  }
  .repo-row-group.drop-before::before {
    top: -3px;
  }
  .repo-row-group.drop-after::after {
    bottom: -3px;
  }
  .nav-item.dragging {
    opacity: 0.35;
    background-color: var(--color-surface-subtle);
  }
  :global(body.dragging-pinned-repo),
  :global(body.dragging-pinned-repo *) {
    cursor: grabbing !important;
  }
  .drag-ghost {
    position: fixed;
    pointer-events: none;
    z-index: 9999;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    background-color: var(--color-surface-strong);
    border: 1px solid var(--color-border-mid);
    border-radius: var(--border-radius-sm);
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    max-width: 14rem;
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
  class="repos-list {PINNED_LIST_CLASS}"
  class:empty={pinnedRepos.length === 0}>
  {#each pinnedRepos as repo (repo.rid)}
    <div
      class="repo-row-group"
      class:drop-before={drag.dropTargetRid === repo.rid &&
        drag.dropPosition === "before" &&
        drag.draggingRid !== repo.rid}
      class:drop-after={drag.dropTargetRid === repo.rid &&
        drag.dropPosition === "after" &&
        drag.draggingRid !== repo.rid}
      {...{ [DRAG_RID_ATTRIBUTE]: repo.rid }}
      animate:flip={{ duration: animationDuration }}
      in:receive={{ key: repo.rid, duration: animationDuration }}
      out:send={{ key: repo.rid, duration: animationDuration }}>
      {@render repoRowInner(repo, true)}
    </div>
  {/each}
</div>

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
          } else if (e.key === "Enter" && unpinnedFilteredRepos.length > 0) {
            void router.push({
              resource: "repo.home",
              rid: unpinnedFilteredRepos[0].rid,
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
      All Repos{unpinnedReposCount > 1 ? ` (${unpinnedReposCount})` : ""}
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

{#snippet repoRowInner(repo: RepoSummary, pinned: boolean = false)}
  {@const pinState = pinnedRepoIds.value.includes(repo.rid)}
  <a
    class="nav-item"
    class:active={isRepoHome(repo.rid)}
    class:dragging={pinned && drag.draggingRid === repo.rid}
    draggable="false"
    onmousedown={pinned ? e => drag.onMouseDown(e, repo.rid) : undefined}
    onclick={pinned ? drag.onClick : undefined}
    href={router.routeToPath({ resource: "repo.home", rid: repo.rid })}>
    <RepoAvatar name={repo.name} rid={repo.rid} styleWidth="1rem" />
    <span class="txt-overflow">{repo.name}</span>
    <span
      class="row-actions"
      role="none"
      onclick={e => {
        e.preventDefault();
        e.stopPropagation();
      }}>
      <button
        class="pin-button"
        title={pinState ? "Unpin repository" : "Pin repository"}
        onclick={() => togglePin(repo.rid)}>
        <Icon name={pinState ? "pin-filled" : "pin-hollow"} />
      </button>
      <span title="Copy RID">
        <Clipboard text={repo.rid} noPopover />
      </span>
      {#if pinned}
        <span class="drag-handle" title="Drag to reorder">
          <Icon name="drag-handle" />
        </span>
      {/if}
    </span>
  </a>
  {#if activeRid() === repo.rid}
    {@const activeProject = activeRepo?.payloads["xyz.radicle.project"]}
    {@render subItem(
      router.routeToPath({ resource: "repo.commits", rid: repo.rid }),
      "branch",
      "Commits",
      isCommits(repo.rid),
      activeCommitCount,
    )}
    {@render subItem(
      router.routeToPath({
        resource: "repo.issues",
        rid: repo.rid,
        status: "open",
      }),
      "issue",
      "Issues",
      isIssues(repo.rid),
      activeProject?.meta.issues.open || undefined,
    )}
    {@render subItem(
      router.routeToPath({
        resource: "repo.patches",
        rid: repo.rid,
        status: "open",
      }),
      "patch",
      "Patches",
      isPatches(repo.rid),
      activeProject?.meta.patches.open || undefined,
    )}
  {/if}
{/snippet}

{#snippet subItem(
  href: string,
  icon: "branch" | "issue" | "patch",
  label: string,
  active: boolean,
  count: number | undefined,
)}
  <a class="nav-item sub-item" class:active {href}>
    <span class="icon"><Icon name={icon} /></span>
    {label}
    {#if count !== undefined}
      <span class="global-counter-badge">{count}</span>
    {/if}
  </a>
{/snippet}

{#if reposExpanded.value}
  <ScrollArea
    style="flex: 1; min-height: 0; mask-image: linear-gradient(to bottom, transparent 0, black 0.5rem, black calc(100% - 0.5rem), transparent 100%);">
    <div class="repos-list">
      {#each unpinnedFilteredRepos as repo (repo.rid)}
        <div
          class="repo-row-group"
          animate:flip={{ duration: animationDuration }}
          in:receive={{ key: repo.rid, duration: animationDuration }}
          out:send={{ key: repo.rid, duration: animationDuration }}>
          {@render repoRowInner(repo, false)}
        </div>
      {/each}
    </div>
  </ScrollArea>
{/if}

{#if drag.draggedRepo}
  <div
    class="drag-ghost"
    style:left="{drag.ghostX + 12}px"
    style:top="{drag.ghostY + 8}px">
    <RepoAvatar
      name={drag.draggedRepo.name}
      rid={drag.draggedRepo.rid}
      styleWidth="1rem" />
    <span class="txt-overflow">{drag.draggedRepo.name}</span>
  </div>
{/if}
