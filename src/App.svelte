<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/Error";
  import type { Config } from "@bindings/config/Config";

  import { onDestroy, onMount } from "svelte";

  import * as router from "@app/lib/router";
  import { theme } from "@app/components/ThemeSwitch.svelte";
  import { unreachable } from "@app/lib/utils";

  import Auth from "@app/views/booting/Auth.svelte";
  import CreateIdentity from "@app/views/booting/CreateIdentity.svelte";
  import CreateIssue from "@app/views/repo/CreateIssue.svelte";
  import Inbox from "@app/views/home/Inbox.svelte";
  import Issue from "@app/views/repo/Issue.svelte";
  import Issues from "@app/views/repo/Issues.svelte";
  import Patch from "@app/views/repo/Patch.svelte";
  import Patches from "@app/views/repo/Patches.svelte";
  import Repos from "@app/views/home/Repos.svelte";
  import { invoke } from "@app/lib/invoke";
  import {
    createEventEmitters,
    unlistenEvents,
    unlistenNodeEvents,
    unlistenSyncStatus,
  } from "@app/lib/startup.svelte";

  const activeRouteStore = router.activeRouteStore;

  const passphrase = $state("");
  let profile = $state<Config>();
  let error = $state<ErrorWrapper>();

  onMount(async () => {
    try {
      profile = await invoke<Config>("startup");
      await invoke("authenticate", { passphrase });

      if (window.__TAURI_INTERNALS__) {
        await createEventEmitters();
      }

      void router.loadFromLocation();
    } catch (err) {
      error = err as ErrorWrapper;
      console.error(err);
    }
  });

  onDestroy(() => {
    if (unlistenEvents.fn) {
      unlistenEvents.fn();
    }
    if (unlistenSyncStatus.fn) {
      unlistenSyncStatus.fn();
    }
    if (unlistenNodeEvents.fn) {
      unlistenNodeEvents.fn();
    }
  });

  $effect(() => document.documentElement.setAttribute("data-theme", $theme));
</script>

{#if $activeRouteStore.resource === "booting"}
  {#if error?.type === "ProfileError"}
    <CreateIdentity />
  {:else if error?.type === "IdentityError.Crypto" && profile}
    <Auth profile={{ did: profile.publicKey, alias: profile.alias }} />
  {/if}
{:else if $activeRouteStore.resource === "home"}
  <Repos {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "inbox"}
  <Inbox {...$activeRouteStore.params} />
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
