<script lang="ts">
  import type { PatchStatus } from "./router";

  import { activeRouteStore } from "@app/lib/router";

  import Link from "@app/components/Link.svelte";

  interface Props {
    rid: string;
    status: PatchStatus | undefined;
  }

  const { rid, status }: Props = $props();
</script>

{#if $activeRouteStore.resource === "repo.patches"}
  Patches
{:else}
  <Link route={{ resource: "repo.patches", rid, status }}>Patches</Link>
{/if}
