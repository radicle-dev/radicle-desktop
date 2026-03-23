<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import delay from "lodash/delay";
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";

  import {
    decreaseFontSize,
    fontSettings,
    increaseFontSize,
    resetFontSize,
  } from "@app/lib/appearance.svelte";
  import { checkAuth, startup } from "@app/lib/auth.svelte";
  import { nodeRunning } from "@app/lib/events";
  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";
  import { hide } from "@app/lib/modal";
  import * as router from "@app/lib/router";
  import { isLoadedRepoRoute } from "@app/lib/router/definitions";
  import {
    setUnlistenNodeEvents,
    unlistenNodeEvents,
  } from "@app/lib/startup.svelte";
  import { isMac, unreachable } from "@app/lib/utils";

  import AppSidebar from "@app/components/AppSidebar.svelte";
  import { codeFont } from "@app/components/CodeFontSwitch.svelte";
  import {
    followSystemTheme,
    loadTheme,
    theme,
  } from "@app/components/ThemeSwitch.svelte";
  import GuideView from "@app/modals/Guide.svelte";
  import Auth from "@app/views/auth/Auth.svelte";
  import CreateIdentity from "@app/views/auth/CreateIdentity.svelte";
  import InboxView from "@app/views/Inbox.svelte";
  import Issue from "@app/views/repo/Issue.svelte";
  import Issues from "@app/views/repo/Issues.svelte";
  import Patch from "@app/views/repo/Patch.svelte";
  import Patches from "@app/views/repo/Patches.svelte";
  import RepoHome from "@app/views/repo/RepoHome.svelte";

  import Command from "./components/Command.svelte";
  import ExternalLink from "./components/ExternalLink.svelte";
  import FullscreenModalPortal from "./components/FullscreenModalPortal.svelte";
  import FullWindowError from "./components/FullWindowError.svelte";
  import Spinner from "./components/Spinner.svelte";

  const activeRouteStore = router.activeRouteStore;

  const activeRepo = $derived.by((): RepoInfo | undefined => {
    const route = $activeRouteStore;
    return isLoadedRepoRoute(route) ? route.params.repo : undefined;
  });

  const DRAG_REGION_HEIGHT = 32;
  const INTERACTIVE_TAGS = new Set([
    "a",
    "button",
    "input",
    "select",
    "textarea",
  ]);

  function isDraggableArea(e: MouseEvent): boolean {
    if (e.clientY > DRAG_REGION_HEIGHT) return false;
    let el = e.target as HTMLElement | null;
    while (el && el !== document.body) {
      if (INTERACTIVE_TAGS.has(el.tagName.toLowerCase())) return false;
      if (el.getAttribute("role") === "button") return false;
      if (el.classList.contains("txt-selectable")) return false;
      el = el.parentElement;
    }
    return true;
  }

  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", ({ matches }) => {
      if (get(followSystemTheme)) {
        theme.set(matches ? "dark" : "light");
      }
    });

  if (get(followSystemTheme)) {
    theme.set(loadTheme());
  }

  let profile = $state<Config>();

  let showSpinner = $state(false);
  delay(() => (showSpinner = true), 1000);

  onMount(async () => {
    try {
      profile = await invoke<Config>("startup");
    } catch (err) {
      startup.error = err as ErrorWrapper;
      return;
    }

    if (window.__TAURI_INTERNALS__) {
      setUnlistenNodeEvents(
        await listen<boolean>("node_running", event => {
          nodeRunning.set(event.payload);
        }),
      );
    }

    try {
      await invoke("authenticate");
      void router.loadFromLocation();
      dynamicInterval(
        "auth",
        checkAuth,
        import.meta.env.VITE_AUTH_LONG_DELAY || 30_000,
      );
    } catch (err) {
      startup.error = err as ErrorWrapper;
      void router.push({ resource: "booting" });
      dynamicInterval("auth", checkAuth, 5_000);
    }
  });

  onDestroy(() => {
    if (unlistenNodeEvents) {
      unlistenNodeEvents();
    }
  });

  $effect(() =>
    document.documentElement.style.setProperty(
      "--font-size",
      `${fontSettings.size}px`,
    ),
  );
  $effect(() => document.documentElement.setAttribute("data-theme", $theme));
  $effect(() =>
    document.documentElement.setAttribute("data-codefont", $codeFont),
  );
</script>

<style>
  .spinner {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
  }
  .layout {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>

<svelte:document
  onmousedown={e => {
    if (window.__TAURI_INTERNALS__ && isDraggableArea(e)) {
      void getCurrentWindow().startDragging();
    }
  }}
  onkeydown={e => {
    const auxiliarKey = isMac() ? e.metaKey : e.ctrlKey;
    // Handles the position of the plus key on different keyboard layouts.
    const plusKey = e.key === "1" || e.key === "=";
    if (e.key === "Escape") {
      hide();
    } else if (auxiliarKey && (e.key === "+" || plusKey)) {
      increaseFontSize();
    } else if (auxiliarKey && e.key === "-") {
      decreaseFontSize();
    } else if (auxiliarKey && e.key.toLowerCase() === "0") {
      resetFontSize();
    }
  }} />
<FullscreenModalPortal />

{#if $activeRouteStore.resource === "booting"}
  {#if startup.error?.code === "IdentityError.MissingProfile"}
    <CreateIdentity />
  {:else if startup.error?.code === "PassphraseError.InvalidPassphrase" && profile}
    <Auth profile={{ did: profile.publicKey, alias: profile.alias }} />
  {:else if startup.error}
    <FullWindowError title="An error occurred" error={startup.error}>
      We were unable to load your Radicle identity, or your Radicle installation
      is outdated.
      <br />
      If you have an existing Radicle installation, make sure you have
      <ExternalLink href="https://radicle.xyz/download">
        the latest version.
      </ExternalLink>
      <br />
      <br />
      <div
        style="display: flex; flex-direction: column; align-items: center; gap: 0.5rem;">
        To migrate your Radicle storage, make sure to restart your radicle-node
        or run:
        <Command styleWidth="30rem" command="rad cob migrate" />
      </div>
    </FullWindowError>
  {:else if showSpinner}
    <div class="spinner"><Spinner /></div>
  {/if}
{:else}
  <div class="layout">
    <AppSidebar
      sidebarData={$activeRouteStore.params.sidebarData}
      {activeRepo} />
    {#if $activeRouteStore.resource === "inbox"}
      <InboxView {...$activeRouteStore.params} />
    {:else if $activeRouteStore.resource === "guide"}
      <GuideView {...$activeRouteStore.params} />
    {:else if $activeRouteStore.resource === "repo.home"}
      <RepoHome {...$activeRouteStore.params} />
    {:else if $activeRouteStore.resource === "repo.issue"}
      <Issue {...$activeRouteStore.params} />
    {:else if $activeRouteStore.resource === "repo.issues"}
      <Issues {...$activeRouteStore.params} />
    {:else if $activeRouteStore.resource === "repo.patch"}
      <Patch {...$activeRouteStore.params} />
    {:else if $activeRouteStore.resource === "repo.patches"}
      <Patches {...$activeRouteStore.params} />
    {:else}
      {unreachable($activeRouteStore)}
    {/if}
  </div>
{/if}
