<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";
  import type { Config } from "@bindings/config/Config";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  import { onDestroy, onMount } from "svelte";

  import * as router from "@app/lib/router";
  import { checkAuth, startup } from "@app/lib/auth.svelte";
  import { dynamicInterval } from "@app/lib/interval";
  import { createEventEmittersOnce } from "@app/lib/startup.svelte";
  import { invoke } from "@app/lib/invoke";
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

  const activeRouteStore = router.activeRouteStore;

  let profile = $state<Config>();
  let unlistenEvents: UnlistenFn | undefined = undefined;
  let unlistenNodeEvents: UnlistenFn | undefined = undefined;
  let unlistenSyncStatus: UnlistenFn | undefined = undefined;

  onMount(async () => {
    try {
      profile = await invoke<Config>("startup");
    } catch (err) {
      startup.error = err as ErrorWrapper;
      return;
    }

    if (window.__TAURI_INTERNALS__) {
      [unlistenEvents, unlistenNodeEvents, unlistenSyncStatus] =
        await createEventEmittersOnce();
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
    if (unlistenEvents) {
      unlistenEvents();
    }
    if (unlistenSyncStatus) {
      unlistenSyncStatus();
    }
    if (unlistenNodeEvents) {
      unlistenNodeEvents();
    }
  });

  $effect(() => document.documentElement.setAttribute("data-theme", $theme));
</script>

{#if $activeRouteStore.resource === "booting"}
  {#if startup.error?.code === "IdentityError.MissingProfile"}
    <CreateIdentity />
  {:else if startup.error?.code === "PassphraseError.InvalidPassphrase" && profile}
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
