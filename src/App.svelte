<script lang="ts">
  import { onDestroy } from "svelte";

  import * as router from "@app/lib/router";
  import { theme } from "@app/components/ThemeSwitch.svelte";
  import { unreachable } from "@app/lib/utils";
  import {
    unlistenEvents,
    unlistenNodeEvents,
    unlistenSyncStatus,
  } from "@app/lib/init.svelte";

  import Booting from "@app/views/boot/Booting.svelte";
  import CreateIssue from "@app/views/repo/CreateIssue.svelte";
  import Inbox from "@app/views/home/Inbox.svelte";
  import Issue from "@app/views/repo/Issue.svelte";
  import Issues from "@app/views/repo/Issues.svelte";
  import Patch from "@app/views/repo/Patch.svelte";
  import Patches from "@app/views/repo/Patches.svelte";
  import Repos from "@app/views/home/Repos.svelte";

  const activeRouteStore = router.activeRouteStore;

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
  <Booting />
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
