<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { boolean } from "zod";

  import { checkRadicleCLI } from "@app/lib/checkRadicleCLI.svelte";
  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";
  import { modalStore, show } from "@app/lib/modal";
  import { notificationCount } from "@app/lib/notificationCount.svelte";
  import * as router from "@app/lib/router";
  import type { SidebarData } from "@app/lib/router/definitions";
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
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    width: 16.5rem;
    border-right: 1px solid var(--color-border-subtle);
  }
  .top {
    padding: 0 0.5rem;
    height: 1.75rem;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.25rem;
    flex-shrink: 0;
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
  .icon {
    color: var(--color-text-tertiary);
  }
  .update-badge {
    margin-left: auto;
    font: var(--txt-body-s-regular);
    color: var(--color-text-tertiary);
  }
</style>

<div class="sidebar">
  <div
    class="top"
    style:height={isMac() ? "2.75rem" : "1.75rem"}
    data-tauri-drag-region>
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
  </div>

  <div class="nav">
    <IdentityButton config={sidebarData.config} />

    <a
      class="nav-item"
      class:active={isInbox()}
      href={router.routeToPath({ resource: "inbox" })}>
      <span class="icon"><Icon name="inbox" /></span>
      Inbox
      {#if notificationCount.value > 0}
        <span class="global-counter-badge">{notificationCount.value}</span>
      {/if}
    </a>

    <SidebarRepoList
      initialRepos={sidebarData.repos}
      initialSeededNotReplicated={sidebarData.seededNotReplicated}
      {activeRepo} />
  </div>

  <div class="bottom">
    <Button
      variant="naked"
      styleWidth="100%"
      styleJustifyContent="flex-start"
      active={isGuide()}
      onclick={() => router.push({ resource: "guide" })}>
      <span class="icon"><Icon name="guide" /></span>
      Guide
    </Button>
    <Button
      variant="naked"
      styleWidth="100%"
      styleJustifyContent="flex-start"
      active={isSettings()}
      onclick={() => show({ component: SettingsView, props: {} })}>
      <span class="icon"><Icon name="settings" /></span>
      Settings
      {#if updateChecker.newVersion}
        <span class="update-badge">New Update</span>
      {/if}
    </Button>
    <NodeStatusButton />
  </div>
</div>
