<script lang="ts">
  import type { IssueStatus } from "./router";

  import { activeRouteStore } from "@app/lib/router";

  import Link from "@app/components/Link.svelte";

  interface Props {
    rid: string;
    status: IssueStatus;
  }

  const { rid, status }: Props = $props();
</script>

{#if $activeRouteStore.resource === "repo.issues"}
  Issues
{:else}
  <Link route={{ resource: "repo.issues", rid, status }}>Issues</Link>
{/if}
