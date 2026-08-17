<script lang="ts" module>
  import { array, boolean, string } from "zod";

  import useLocalStorage from "@app/lib/useLocalStorage.svelte";

  let filterOpen = $state(false);
  let filterQuery = $state("");

  const reposExpanded = useLocalStorage(
    "sidebarReposExpanded",
    boolean(),
    true,
    !window.localStorage,
  );

  const pinnedRepoIds = useLocalStorage(
    "sidebarPinnedRepos",
    array(string()),
    [],
    !window.localStorage,
  );

  export function revealRepoInSidebar(rid: string) {
    if (pinnedRepoIds.value.includes(rid)) return;

    filterQuery = "";
    filterOpen = false;
    reposExpanded.value = true;

    requestAnimationFrame(() => {
      const row = document.querySelector(
        `[data-unpinned-rid="${CSS.escape(rid)}"]`,
      );
      row?.scrollIntoView({ block: "nearest" });
    });
  }
</script>

<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { RepoSummary } from "@bindings/repo/RepoSummary";

  import { onMount } from "svelte";
  import { flip } from "svelte/animate";
  import { backOut } from "svelte/easing";
  import { crossfade, fade, scale, slide } from "svelte/transition";

  import { nodeRunning } from "@app/lib/events";
  import { dynamicInterval, resetDynamicInterval } from "@app/lib/interval";
  import {
    cachedListReposSummary,
    invalidateReposSummary,
    invoke,
    writeToClipboard,
  } from "@app/lib/invoke";
  import * as router from "@app/lib/router";
  import { sidebarCollapsed } from "@app/lib/sidebar.svelte";
  import {
    explorerHost,
    explorerUrl,
    formatRepositoryId,
  } from "@app/lib/utils";

  import AddRepoButton from "@app/components/AddRepoButton.svelte";
  import ContextMenu from "@app/components/ContextMenu.svelte";
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
    config: Config;
  }

  const {
    initialRepos,
    initialSeededNotReplicated,
    activeRepo = undefined,
    config,
  }: Props = $props();

  let repos: RepoSummary[] = $derived(initialRepos);
  let seededNotReplicated: string[] = $derived(initialSeededNotReplicated);
  let filterInputElement: HTMLInputElement | undefined = $state(undefined);

  let contextMenu = $state<
    { x: number; y: number; repo: RepoSummary; target: HTMLElement } | undefined
  >(undefined);

  function openContextMenu(event: MouseEvent, repo: RepoSummary) {
    event.preventDefault();
    event.stopPropagation();
    contextMenu = {
      x: event.clientX,
      y: event.clientY,
      repo,
      target: event.currentTarget as HTMLElement,
    };
  }

  function closeContextMenu() {
    contextMenu = undefined;
  }

  $effect(() => {
    if (filterOpen && filterInputElement) {
      filterInputElement.focus({ preventScroll: true });
    }
  });

  // Collapsing closes the filter so the rail doesn't show a second search icon.
  $effect(() => {
    if (sidebarCollapsed.value) {
      filterOpen = false;
      filterQuery = "";
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

  const fetchingExpanded = useLocalStorage(
    "sidebarFetchingExpanded",
    boolean(),
    true,
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

  // FLIP the "All Repos" header's icon + search/add buttons and animate its
  // height as it reflows between a row (expanded) and a column (collapsed).
  // Positions are measured RELATIVE to the header so the FLIP only animates the
  // internal reflow; the header's own downward shift (the controls growing
  // above it) is carried by the shared layout transition on the same timeline.
  let allReposHeaderEl = $state<HTMLElement | undefined>();
  let headerFlipFirst = new WeakMap<Element, { x: number; y: number }>();
  let headerFlipHeight = 0;

  $effect.pre(() => {
    if (sidebarCollapsed.value || !sidebarCollapsed.value) {
      const el = allReposHeaderEl;
      const map = new WeakMap<Element, { x: number; y: number }>();
      if (el) {
        const hr = el.getBoundingClientRect();
        headerFlipHeight = hr.height;
        for (const child of el.querySelectorAll("[data-flip]")) {
          const r = child.getBoundingClientRect();
          map.set(child, { x: r.left - hr.left, y: r.top - hr.top });
        }
      }
      headerFlipFirst = map;
    }
  });

  $effect(() => {
    if (!(sidebarCollapsed.value || !sidebarCollapsed.value)) return;
    const el = allReposHeaderEl;
    if (!el) return;
    const hr = el.getBoundingClientRect();
    if (Math.abs(hr.height - headerFlipHeight) > 0.5) {
      el.animate(
        [{ height: `${headerFlipHeight}px` }, { height: `${hr.height}px` }],
        { duration: 200, easing: "ease" },
      );
    }
    for (const child of el.querySelectorAll("[data-flip]")) {
      const first = headerFlipFirst.get(child);
      if (!first) continue;
      const r = child.getBoundingClientRect();
      const dx = first.x - (r.left - hr.left);
      const dy = first.y - (r.top - hr.top);
      if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) continue;
      child.animate(
        [
          { transform: `translate(${dx}px, ${dy}px)` },
          { transform: "translate(0px, 0px)" },
        ],
        { duration: 200, easing: "ease" },
      );
    }
  });

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
    invalidateReposSummary();
    [repos, seededNotReplicated] = await Promise.all([
      cachedListReposSummary(),
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
    overflow: hidden;
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
  /* The rail has no room for the section's header, its filter or its add
     button, so the repo rows below stand on their own there. */
  .section-header.mini {
    display: none;
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
    transition: width 0.2s ease;
  }
  .nav-item .txt-overflow {
    min-width: 0;
  }
  /* Collapsed: the hover row-actions don't fit the rail, and reordering is
     turned off there, so the rows stay ordinary links. */
  :global(.sidebar.mini) .row-actions {
    display: none;
  }
  /* Collapsed: the repo rows are centered on the rail. */
  :global(.sidebar.mini) .repos-list {
    align-items: center;
  }
  .nav-item :global(img),
  .nav-item :global(svg) {
    -webkit-user-drag: none;
  }
  .nav-item:hover,
  .nav-item.context-active {
    background-color: var(--color-surface-subtle);
  }
  .nav-item.active {
    background-color: var(--color-surface-subtle);
  }
  .nav-item .global-counter-badge {
    margin-left: auto;
  }
  /* The rows unroll from behind the repo they belong to: the container clips
     them while its height animates, so they slide out from under the avatar
     rather than appearing in place. */
  .sub-items {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding-top: 0.25rem;
  }
  .sub-item {
    padding-left: 2rem;
    transition: padding-left 0.2s ease;
  }
  .sub-item.mini {
    padding-left: 0.5rem;
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

  /* Taken out of the flow: hidden actions still reserve their width, which cost
     the name a button's worth of room on every row. They sit over the end of
     the name instead, on the same fill the row takes when they show. */
  .nav-item .row-actions {
    visibility: hidden;
    position: absolute;
    top: 0;
    bottom: 0;
    right: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.125rem;
    padding-left: 0.25rem;
    background-color: var(--color-surface-subtle);
    color: var(--color-text-tertiary);
  }
  .nav-item:hover .row-actions,
  .nav-item.context-active .row-actions,
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

  /* No gap here: the sub-items carry their own top padding instead, so it
     animates open with them. A gap on this container would be laid out the
     instant they mount and dropped the instant they leave, jumping either
     side of the transition. */
  .repo-row-group {
    position: relative;
    display: flex;
    flex-direction: column;
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
  .repo-icon {
    position: relative;
    display: inline-flex;
  }
  /* Hovering the header swaps the repository icon for the arrows that say what
     clicking it will do. Keyed off the label so the filter and add buttons
     beside it don't trigger the swap. */
  .icon-stack {
    display: grid;
  }
  .icon-default,
  .icon-hover {
    grid-area: 1 / 1;
    transition: opacity 150ms ease;
  }
  .icon-hover {
    opacity: 0;
  }
  .section-header-label:hover .icon-default,
  .section-header:focus-visible .icon-default {
    opacity: 0;
  }
  .section-header-label:hover .icon-hover,
  .section-header:focus-visible .icon-hover {
    opacity: 1;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    min-height: 2rem;
    padding: 0 0.75rem;
    background: transparent;
    border: 0;
    border-radius: var(--border-radius-sm);
    color: var(--color-text-primary);
    font: var(--txt-body-m-regular);
    text-align: left;
    white-space: nowrap;
    cursor: pointer;
  }
  .menu-item :global(svg) {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }
  .menu-item:hover {
    background-color: var(--color-surface-subtle);
  }
  .menu-item:hover :global(svg) {
    color: var(--color-text-primary);
  }
</style>

{#if seededNotReplicated.length > 0 && !sidebarCollapsed.value}
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
  bind:this={allReposHeaderEl}
  class="section-header"
  class:mini={sidebarCollapsed.value}
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
      <span class="icon repo-icon" data-flip>
        <span class="icon-stack">
          <span class="icon-default"><Icon name="repository" /></span>
          <span class="icon-hover">
            <Icon
              name={reposExpanded.value
                ? "collapse-vertical"
                : "expand-vertical"} />
          </span>
        </span>
      </span>
      <span class="label">All Repos</span>
      {#if !sidebarCollapsed.value && unpinnedReposCount > 1}
        <span
          class="global-counter-badge"
          in:scale={{ duration: 200, easing: backOut, start: 0 }}
          out:fade={{ duration: 200 }}>
          {unpinnedReposCount}
        </span>
      {/if}
    </span>
  {/if}
  <span class="section-header-actions">
    {#if !filterOpen}
      <span data-flip onclick={e => e.stopPropagation()} role="none">
        <button
          class="filter-button"
          title="Filter repos"
          aria-keyshortcuts="ctrl+f"
          onclick={() => {
            sidebarCollapsed.value = false;
            filterOpen = true;
            reposExpanded.value = true;
          }}>
          <span class="icon"><Icon name="search" /></span>
        </button>
      </span>
    {/if}
    <span
      class="add-repo-action"
      data-flip
      onclick={e => e.stopPropagation()}
      role="none">
      <AddRepoButton reload={reloadRepos} {repos} {seededNotReplicated} />
    </span>
  </span>
</div>

{#snippet repoRowInner(repo: RepoSummary, pinned: boolean = false)}
  {@const pinState = pinnedRepoIds.value.includes(repo.rid)}
  <a
    class="nav-item"
    class:active={isRepoHome(repo.rid)}
    class:context-active={contextMenu?.repo.rid === repo.rid}
    class:dragging={pinned && drag.draggingRid === repo.rid}
    draggable="false"
    onmousedown={pinned && !sidebarCollapsed.value
      ? e => drag.onMouseDown(e, repo.rid)
      : undefined}
    onclick={pinned && !sidebarCollapsed.value ? drag.onClick : undefined}
    oncontextmenu={e => openContextMenu(e, repo)}
    href={router.routeToPath({ resource: "repo.home", rid: repo.rid })}>
    <RepoAvatar name={repo.name} rid={repo.rid} styleWidth="1rem" />
    <span class="txt-overflow label">{repo.name}</span>
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
      {#if pinned}
        <span class="drag-handle" title="Drag to reorder">
          <Icon name="drag-handle" />
        </span>
      {/if}
    </span>
  </a>
  {#if activeRid() === repo.rid}
    {@const activeProject = activeRepo?.payloads["xyz.radicle.project"]}
    <div
      class="sub-items"
      transition:slide={{ duration: ANIMATION_DURATION_MS }}>
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
    </div>
  {/if}
{/snippet}

{#snippet subItem(
  href: string,
  icon: "branch" | "issue" | "patch",
  label: string,
  active: boolean,
  count: number | undefined,
)}
  <a
    class="nav-item sub-item"
    class:active
    class:mini={sidebarCollapsed.value}
    {href}>
    <span class="icon"><Icon name={icon} /></span>
    <span class="label">{label}</span>
    {#if !sidebarCollapsed.value && count !== undefined}
      <span
        class="global-counter-badge"
        in:scale={{ duration: 200, easing: backOut, start: 0 }}
        out:fade={{ duration: 200 }}>
        {count}
      </span>
    {/if}
  </a>
{/snippet}

{#if reposExpanded.value && !sidebarCollapsed.value}
  <ScrollArea
    style="flex: 1; min-height: 0; mask-image: linear-gradient(to bottom, transparent 0, black 0.5rem, black calc(100% - 0.5rem), transparent 100%);">
    <div class="repos-list">
      {#each unpinnedFilteredRepos as repo (repo.rid)}
        <div
          class="repo-row-group"
          data-unpinned-rid={repo.rid}
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

{#if contextMenu}
  {@const repo = contextMenu.repo}
  {@const url = explorerUrl(repo.rid, config)}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    target={contextMenu.target}
    onclose={closeContextMenu}>
    <button
      class="menu-item"
      role="menuitem"
      onclick={() => writeToClipboard(repo.rid)}>
      <Icon name="copy" />
      Copy RID
    </button>
    <button
      class="menu-item"
      role="menuitem"
      onclick={() => writeToClipboard(`rad checkout ${repo.rid}`)}>
      <Icon name="checkout" />
      Copy checkout command
    </button>
    <button
      class="menu-item"
      role="menuitem"
      onclick={() => writeToClipboard(url)}>
      <Icon name="link" />
      Copy link to {explorerHost(config)}
    </button>
    <a
      class="menu-item"
      role="menuitem"
      href={url}
      target="_blank"
      rel="noreferrer noopener">
      <Icon name="open-external" />
      Open in {explorerHost(config)}
    </a>
  </ContextMenu>
{/if}
