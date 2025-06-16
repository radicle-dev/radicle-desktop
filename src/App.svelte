<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";
  import type { Config } from "@bindings/config/Config";

  import delay from "lodash/delay";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  import * as router from "@app/lib/router";
  import { checkAuth, startup } from "@app/lib/auth.svelte";
  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import {
    resetFontSize,
    increaseFontSize,
    decreaseFontSize,
    fontSettings,
  } from "@app/lib/appearance.svelte";
  import {
    setUnlistenNodeEvents,
    unlistenNodeEvents,
  } from "@app/lib/startup.svelte";
  import { theme } from "@app/components/ThemeSwitch.svelte";
  import { unreachable, isMac } from "@app/lib/utils";

  import Auth from "@app/views/booting/Auth.svelte";
  import CreateIdentity from "@app/views/booting/CreateIdentity.svelte";
  import CreateIssue from "@app/views/repo/CreateIssue.svelte";
  import Issue from "@app/views/repo/Issue.svelte";
  import Issues from "@app/views/repo/Issues.svelte";
  import Patch from "@app/views/repo/Patch.svelte";
  import Patches from "@app/views/repo/Patches.svelte";
  import RepoHome from "@app/views/repo/RepoHome.svelte";
  import Repos from "@app/views/home/Repos.svelte";
  import Spinner from "./components/Spinner.svelte";
  import FullWindowError from "./components/FullWindowError.svelte";
  import ExternalLink from "./components/ExternalLink.svelte";
  import Command from "./components/Command.svelte";

  const activeRouteStore = router.activeRouteStore;

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
</script>

<style>
  .spinner {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
  }
</style>

<svelte:document
  onkeydown={e => {
    const auxiliarKey = isMac() ? e.metaKey : e.ctrlKey;
    // Handles the position of the plus key on different keyboard layouts.
    const plusKey = e.key === "1" || e.key === "=";
    if (auxiliarKey && (e.key === "+" || plusKey)) {
      increaseFontSize();
    } else if (auxiliarKey && e.key === "-") {
      decreaseFontSize();
    } else if (auxiliarKey && e.key.toLowerCase() === "0") {
      resetFontSize();
    }
  }} />

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
{:else if $activeRouteStore.resource === "home"}
  <Repos {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "repo.home"}
  <RepoHome {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "repo.createIssue"}
  <CreateIssue {...$activeRouteStore.params} />
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
