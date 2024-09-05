<script lang="ts">
  import { onMount } from "svelte";

  import { invoke } from "@tauri-apps/api/core";

  import * as router from "@app/lib/router";
  import { theme } from "@app/components/ThemeSwitch.svelte";
  import { unreachable } from "@app/lib/utils";

  import AuthenticationError from "@app/views/AuthenticationError.svelte";
  import Home from "@app/views/Home.svelte";

  const activeRouteStore = router.activeRouteStore;
  void router.loadFromLocation();

  onMount(async () => {
    try {
      // For development purposes don't run tauri commands when viewing from
      // a browser.
      if (window.__TAURI_INTERNALS__) {
        await invoke("authenticate");
      }

      void router.push({ resource: "home" });
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
{:else if $activeRouteStore.resource === "authenticationError"}
  <AuthenticationError {...$activeRouteStore.params} />
{:else}
  {unreachable($activeRouteStore)}
{/if}
