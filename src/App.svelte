<script lang="ts">
  import { onMount } from "svelte";

  import { invoke } from "@tauri-apps/api/core";

  import * as router from "@app/lib/router";
  import { theme } from "@app/components/ThemeSwitch.svelte";
  import { unreachable } from "@app/lib/utils";

  import DesignSystem from "@app/views/DesignSystem.svelte";
  import Home from "@app/views/Home.svelte";
  import AuthenticationError from "@app/views/AuthenticationError.svelte";

  const activeRouteStore = router.activeRouteStore;
  void router.loadFromLocation();

  onMount(async () => {
    try {
      await invoke("authenticate");
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
  <Home />
{:else if $activeRouteStore.resource === "authenticationError"}
  <AuthenticationError {...$activeRouteStore.params} />
{:else if $activeRouteStore.resource === "designSystem"}
  <DesignSystem />
{:else}
  {unreachable($activeRouteStore)}
{/if}
