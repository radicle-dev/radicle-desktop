<script lang="ts">
  import { activeRouteStore } from "@app/lib/router";

  import { explorerUrl } from "@app/lib/utils";

  import Link from "@app/components/Link.svelte";
  import BreadcrumbCopyButton from "./BreadcrumbCopyButton.svelte";

  interface Props {
    name: string;
    rid: string;
  }

  const { name, rid }: Props = $props();
</script>

{#if $activeRouteStore.resource === "repo.home"}
  {name}
  <BreadcrumbCopyButton icon="repo" id={rid} url={explorerUrl(`${rid}`)} />
{:else}
  <Link route={{ resource: "repo.home", rid }}>
    {name}
  </Link>
{/if}
