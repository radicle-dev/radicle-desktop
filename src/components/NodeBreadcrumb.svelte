<script lang="ts">
  import type { Config } from "@bindings/config/Config";

  import * as router from "@app/lib/router";

  import { didFromPublicKey, explorerUrl } from "@app/lib/utils";

  import BreadcrumbCopyButton from "@app/views/repo/BreadcrumbCopyButton.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  const activeRouteStore = router.activeRouteStore;

  interface Props {
    config: Config;
  }

  const { config }: Props = $props();
</script>

{#if $activeRouteStore.resource === "home"}
  <NodeId publicKey={config.publicKey} alias={config.alias} />
  <BreadcrumbCopyButton
    icon="user"
    id={didFromPublicKey(config.publicKey)}
    url={explorerUrl(`users/${didFromPublicKey(config.publicKey)}`)} />
{:else}
  <Link route={{ resource: "home", activeTab: "all" }}>
    <NodeId publicKey={config.publicKey} alias={config.alias} />
  </Link>
{/if}
