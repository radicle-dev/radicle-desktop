<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { backOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";
  import { boolean } from "zod";

  import { checkRadicleCLI } from "@app/lib/checkRadicleCLI.svelte";
  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";
  import { modalStore, show } from "@app/lib/modal";
  import { notificationCount } from "@app/lib/notificationCount.svelte";
  import * as router from "@app/lib/router";
  import type { SidebarData } from "@app/lib/router/definitions";
  import {
    MIN_SIDEBAR_WIDTH,
    RAIL_WIDTH_REM,
    setSidebarWidth,
    sidebarCollapsed as collapsed,
    sidebarResizing,
    sidebarWidth,
    toggleSidebar,
  } from "@app/lib/sidebar.svelte";
  import { updateChecker } from "@app/lib/updateChecker.svelte";
  import useLocalStorage from "@app/lib/useLocalStorage.svelte";
  import { isMac } from "@app/lib/utils";

  import { badgeCounter } from "@app/components/BadgeCounterSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import IdentityButton from "@app/components/IdentityButton.svelte";
  import NodeStatusButton from "@app/components/NodeStatusButton.svelte";
  import SidebarRepoList from "@app/components/SidebarRepoList.svelte";
  import SettingsView from "@app/modals/Settings.svelte";

  interface Props {
    sidebarData: SidebarData;
    activeRepo?: RepoInfo;
  }

  const { sidebarData, activeRepo = undefined }: Props = $props();

  const toggleShortcut = isMac() ? "⌘B" : "Ctrl+B";
  const dragStripHeight = isMac() ? "2.75rem" : "1.75rem";

  const mini = $derived(collapsed.value);

  // FLIP the window controls (back/forward/refresh) so the same button
  // elements glide between their row (expanded) and column (collapsed) spots.
  let controlsEl = $state<HTMLElement | undefined>();
  let controlsFirst = new WeakMap<Element, DOMRect>();

  function captureControls(): WeakMap<Element, DOMRect> {
    const map = new WeakMap<Element, DOMRect>();
    if (controlsEl) {
      for (const child of controlsEl.children) {
        map.set(child, child.getBoundingClientRect());
      }
    }
    return map;
  }

  $effect.pre(() => {
    // Read collapsed so this runs (before the DOM updates) on every toggle.
    if (collapsed.value || !collapsed.value) controlsFirst = captureControls();
  });

  $effect(() => {
    if (!(collapsed.value || !collapsed.value)) return;
    if (!controlsEl) return;
    for (const child of controlsEl.children) {
      const first = controlsFirst.get(child);
      if (!first) continue;
      const last = child.getBoundingClientRect();
      const dx = first.left - last.left;
      const dy = first.top - last.top;
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

  // How far below the minimum width the drag has to go before it collapses,
  // rather than the edge just sticking at the minimum.
  const COLLAPSE_DRAG_SLACK_REM = 2;

  // While collapsed, the rail follows the drag at a fraction of its distance so
  // it feels attached to the pointer, instead of sitting still until the
  // threshold and then jumping open.
  const RAIL_STRETCH_FACTOR = 0.35;
  let railStretch = $state(0);

  // Crossing the threshold is a jump between two widths rather than tracking
  // the pointer, so the transition is switched back on to carry it. Matches the
  // width transition on .slot.
  const SNAP_DURATION_MS = 200;
  let snapping = $state(false);
  let snapTimeout: ReturnType<typeof setTimeout> | undefined;

  // Movement under this is a click on the edge, not a drag of it.
  const CLICK_SLOP_PX = 3;

  function snapWidth() {
    snapping = true;
    if (snapTimeout !== undefined) clearTimeout(snapTimeout);
    snapTimeout = setTimeout(() => (snapping = false), SNAP_DURATION_MS);
  }

  function onEdgeMouseDown(e: MouseEvent) {
    e.preventDefault();
    sidebarResizing.value = true;
    document.body.classList.add("resizing-sidebar");
    const startX = e.clientX;
    // From the rail's own width when collapsed, so dragging it back out starts
    // where the edge visually sits rather than from the stored expanded width.
    const startWidth = collapsed.value ? RAIL_WIDTH_REM : sidebarWidth.value;
    const pxPerRem =
      parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
    let dragged = false;
    const onMove = (ev: MouseEvent) => {
      // Nothing is resized until the pointer clears the slop, so a click that
      // wobbles a pixel still reads as a click.
      if (!dragged) {
        if (Math.abs(ev.clientX - startX) <= CLICK_SLOP_PX) return;
        dragged = true;
      }
      const width = startWidth + (ev.clientX - startX) / pxPerRem;
      // One boundary serves both directions: below it the sidebar is collapsed,
      // above it expanded, so a single drag can cross either way. Written only
      // on a change, so dragging doesn't persist the same value every frame.
      const collapse = width < MIN_SIDEBAR_WIDTH - COLLAPSE_DRAG_SLACK_REM;
      if (collapsed.value !== collapse) {
        collapsed.value = collapse;
        snapWidth();
      }
      if (collapse) {
        railStretch = Math.max(
          0,
          (width - RAIL_WIDTH_REM) * RAIL_STRETCH_FACTOR,
        );
      } else {
        railStretch = 0;
        setSidebarWidth(width);
      }
    };
    const onUp = () => {
      sidebarResizing.value = false;
      railStretch = 0;
      document.body.classList.remove("resizing-sidebar");
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      // Clearing the resize flag first lets the toggle animate rather than snap.
      if (!dragged) toggleSidebar();
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  const firstLaunchStorage = useLocalStorage(
    "appFirstLaunch",
    boolean(),
    true,
    !window.localStorage,
  );

  onMount(async () => {
    try {
      await checkRadicleCLI();
    } catch {
      dynamicInterval("checkRadicleCLI", checkRadicleCLI, 1_000);
    }

    const isDefaultRoute =
      window.location.pathname === "/" || window.location.pathname === "/inbox";
    if (firstLaunchStorage.value === true && isDefaultRoute) {
      await router.push({ resource: "guide" });
      firstLaunchStorage.value = false;
    }

    await updateNotificationCount();
    dynamicInterval("notificationCount", updateNotificationCount, 3_000);
  });

  async function updateNotificationCount() {
    notificationCount.value = await invoke<number>("notification_count");
    if (window.__TAURI_INTERNALS__ && $badgeCounter) {
      await getCurrentWindow().setBadgeCount(
        notificationCount.value === 0 ? undefined : notificationCount.value,
      );
    } else if (window.__TAURI_INTERNALS__) {
      await getCurrentWindow().setBadgeCount(undefined);
    }
  }

  $effect(() => {
    if (window.__TAURI_INTERNALS__) {
      void getCurrentWindow().setBadgeCount(
        $badgeCounter && notificationCount.value > 0
          ? notificationCount.value
          : undefined,
      );
    }
  });

  const activeRoute = router.activeRouteStore;

  function isInbox(): boolean {
    return $activeRoute.resource === "inbox";
  }

  function isGuide(): boolean {
    return $activeRoute.resource === "guide";
  }

  function isSettings(): boolean {
    return $modalStore?.component === SettingsView;
  }
</script>

<style>
  .slot {
    position: relative;
    height: 100%;
    min-height: 0;
    flex-shrink: 0;
    transition: width 0.2s ease;
  }
  .slot.resizing {
    transition: none;
  }

  .sidebar {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 5;
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-right: 1px solid var(--color-border-subtle);
    background-color: var(--color-surface-base);
    transition: background-color 0.2s ease;
  }
  /* The rail keeps the border so the rules and edges of the content area have
     something to terminate against. */
  .sidebar.mini {
    background-color: var(--color-surface-canvas);
  }
  /* Buttons shrink to a square when collapsed, same as nav items. */
  .sidebar :global(.button) {
    transition: width 0.2s ease;
  }
  /* All text labels simply fade out while collapsing (no truncation). They're
     non-interactive so the part overflowing the collapsed square doesn't create
     a phantom hover target. */
  .sidebar :global(.label) {
    white-space: nowrap;
    pointer-events: none;
    transition: opacity 0.2s ease;
  }
  /* Collapsed: a label still fading at full width would push the icon it sits
     beside off the rail's centre line, so it gives up its space as well. */
  .sidebar.mini :global(.label) {
    opacity: 0;
    width: 0;
    min-width: 0;
    overflow: hidden;
  }

  .drag-strip {
    flex-shrink: 0;
  }
  .controls {
    display: flex;
    align-items: flex-start;
    gap: 0.25rem;
    padding: 0 0.5rem 0.25rem;
    flex-shrink: 0;
    /* Explicit heights (one button row vs. a 4-button column) so the space
       animates between states; auto heights can't transition. The FLIP keeps
       the buttons inside the growing/shrinking box, so no clipping is needed
       (clipping cut off the toggle at the start of the animation). Top-aligned
       in both states so the toggle stays fixed while the rest reflow. */
    height: 2.25rem;
    transition: height 0.2s ease;
  }
  .window-controls {
    display: flex;
    gap: 0.25rem;
  }
  /* Expanded: the window controls lift into the drag strip beside the traffic
     lights. They stay in the controls row's DOM rather than moving into the
     strip, so the same elements survive the toggle and can glide into the
     rail's column when it collapses. */
  .sidebar:not(.mini) .window-controls {
    position: absolute;
    top: 0;
    right: 0.5rem;
    /* Centred on the traffic lights rather than on the strip, which is taller
       than them and would sit these too low. Twice their centre, measured by
       eye against the real window: the buttons are drawn by macOS, so their
       position cannot be read from the page. */
    height: 2rem;
    align-items: center;
  }
  /* Expanded: with every control lifted into the strip, the row they came from
     has nothing left in flow, so it gives its space back to the nav. */
  .sidebar:not(.mini) .controls {
    height: 0;
    padding: 0;
  }
  .sidebar.mini .controls,
  .sidebar.mini .window-controls {
    flex-direction: column;
    gap: 0.125rem;
    align-items: center;
  }
  .sidebar.mini .controls {
    height: 8.625rem;
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .nav {
    flex: 1;
    overflow: visible;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-height: 0;
  }
  .bottom {
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  /* Collapsed: the rail is a single centered column, so every stack that makes
     it up centers its items rather than aligning them to the sidebar's edge. */
  .sidebar.mini .nav {
    align-items: center;
  }
  /* The rail carries navigation only; guide, settings and node status wait for
     the sidebar to be expanded. */
  .sidebar.mini .bottom {
    display: none;
  }
  /* Collapsed: every interactive item is the same 2rem square hover target, so
     the hover/selected highlight is a uniform square, not a full-width bar. */
  .sidebar.mini :global(.nav-item),
  .sidebar.mini :global(.button),
  .sidebar.mini :global(.filter-button) {
    width: 2rem;
    min-width: 2rem;
    height: 2rem;
    box-sizing: border-box;
    border-radius: var(--border-radius-sm);
    /* The row's gap would still be held open by the zero-width label. */
    gap: 0;
  }
  /* A nav item's 0.5rem padding either side of a 1rem icon already centres it
     in the 2rem square, so it is left alone. Centring it against its content
     instead would make the icon lurch the moment a fixed-width label snaps to
     zero, while labels that shrink gradually would drift smoothly. */
  .sidebar.mini :global(.button),
  .sidebar.mini :global(.filter-button) {
    justify-content: center;
  }
  /* Collapsed: trailing badges are pushed out by `margin-left: auto`, which on
     a centred square drags the icon off centre. */
  .sidebar.mini :global(.update-badge) {
    display: none;
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
    white-space: nowrap;
    transition: width 0.2s ease;
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
  .icon {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }
  .update-badge {
    margin-left: auto;
    font: var(--txt-body-s-regular);
    color: var(--color-text-tertiary);
  }

  .edge {
    position: absolute;
    top: 0;
    right: -3px;
    width: 7px;
    height: 100%;
    z-index: 110;
    cursor: col-resize;
  }
  /* The pointer leaves the 7px edge as soon as the drag starts, so the cursor
     is held across the whole window until it ends. */
  :global(body.resizing-sidebar),
  :global(body.resizing-sidebar *) {
    cursor: col-resize !important;
  }
</style>

<div
  class="slot"
  class:resizing={sidebarResizing.value && !snapping}
  style:width={collapsed.value
    ? `${RAIL_WIDTH_REM + railStretch}rem`
    : `${sidebarWidth.value}rem`}>
  <div
    class="sidebar"
    class:mini
    style:--drag-strip-height={dragStripHeight}
    role="navigation">
    <div
      class="drag-strip"
      style:height={dragStripHeight}
      data-tauri-drag-region>
    </div>

    <div class="controls" bind:this={controlsEl}>
      <span class="window-controls">
        <Button
          variant="naked"
          title="{collapsed.value
            ? 'Expand'
            : 'Collapse'} sidebar ({toggleShortcut})"
          keyShortcuts={isMac() ? "Meta+b" : "Control+b"}
          onclick={toggleSidebar}
          stylePadding="0 4px">
          <span class="icon"><Icon name="sidebar-left" /></span>
        </Button>
        <Button
          variant="naked"
          onclick={() => window.history.back()}
          stylePadding="0 4px">
          <span class="icon"><Icon name="arrow-left" /></span>
        </Button>
        <Button
          variant="naked"
          onclick={() => window.history.forward()}
          stylePadding="0 4px">
          <span class="icon"><Icon name="arrow-right" /></span>
        </Button>
        <Button
          variant="naked"
          title="Reload"
          onclick={() => window.location.reload()}
          stylePadding="0 4px">
          <span class="icon"><Icon name="refresh" /></span>
        </Button>
      </span>
    </div>

    <div class="body">
      <div class="nav">
        {#if !mini}
          <IdentityButton config={sidebarData.config} />
        {/if}

        <a
          class="nav-item"
          class:active={isInbox()}
          title="Inbox"
          href={router.routeToPath({ resource: "inbox" })}>
          <span class="icon"><Icon name="inbox" /></span>
          <span class="label">Inbox</span>
          {#if !mini && notificationCount.value > 0}
            <span
              class="global-counter-badge"
              in:scale={{ duration: 200, easing: backOut, start: 0 }}
              out:fade={{ duration: 200 }}>
              {notificationCount.value}
            </span>
          {/if}
        </a>

        <SidebarRepoList
          initialRepos={sidebarData.repos}
          initialSeededNotReplicated={sidebarData.seededNotReplicated}
          config={sidebarData.config}
          {activeRepo} />
      </div>

      <div class="bottom">
        <Button
          variant="naked"
          title="Guide"
          styleWidth={mini ? "2rem" : "100%"}
          styleJustifyContent="flex-start"
          active={isGuide()}
          onclick={() => router.push({ resource: "guide" })}>
          <span class="icon"><Icon name="guide" /></span>
          <span class="label">Guide</span>
        </Button>
        <Button
          variant="naked"
          title="Settings"
          styleWidth={mini ? "2rem" : "100%"}
          styleJustifyContent="flex-start"
          active={isSettings()}
          onclick={() => show({ component: SettingsView, props: {} })}>
          <span class="icon"><Icon name="settings" /></span>
          <span class="label">Settings</span>
          {#if updateChecker.newVersion}
            <span class="update-badge">New Update</span>
          {/if}
        </Button>
        <NodeStatusButton collapsed={mini} />
      </div>
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="edge"
      onmousedown={onEdgeMouseDown}
      role="separator"
      aria-orientation="vertical"
      tabindex="-1">
    </div>
  </div>
</div>
