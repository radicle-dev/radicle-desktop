<script lang="ts">
  import { onMount } from "svelte";

  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  import * as router from "@app/lib/router";
  import { nodeRunning } from "@app/lib/events";
  import { theme } from "@app/components/ThemeSwitch.svelte";
  import { unreachable } from "@app/lib/utils";

  import AuthenticationError from "@app/views/AuthenticationError.svelte";
  import Home from "@app/views/Home.svelte";
  import Issue from "@app/views/repo/Issue.svelte";
  import Issues from "@app/views/repo/Issues.svelte";
  import Patch from "@app/views/repo/Patch.svelte";
  import Patches from "@app/views/repo/Patches.svelte";

  const activeRouteStore = router.activeRouteStore;

  onMount(async () => {
    try {
      await listen("event", event => {
        console.log(event.payload);
      });

      await listen<boolean>("node_running", event => {
        nodeRunning.set(event.payload);
      });
      // For development purposes don't run tauri commands when viewing from
      // a browser.
      if (window.__TAURI_INTERNALS__) {
        await invoke("authenticate");
      }

      void router.loadFromLocation();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (e: any) {
      void router.push({
        resource: "authenticationError",
        params: {
          error: e.err,
          hint: e.hint,
        },
      });
    }
  });

  $: document.documentElement.setAttribute("data-theme", $theme);
</script>

{#if $activeRouteStore.resource === "booting"}
  <!-- Don't show anything -->
{:else if $activeRouteStore.resource === "home"}
  <Home {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "repo.issue"}
  <Issue {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "repo.issues"}
  <Issues {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "repo.patch"}
  <Patch {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "repo.patches"}
  <Patches {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "authenticationError"}
  <AuthenticationError {...$activeRouteStore.params} />
{:else}
  {unreachable($activeRouteStore)}
{/if}
