<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { Patch } from "@bindings/Patch";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Layout from "./Layout.svelte";

  import Icon from "@app/components/Icon.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  export let repo: RepoInfo;
  export let patches: Patch[];
  export let config: Config;

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<Layout {repo}>
  <svelte:fragment slot="breadcrumbs">
    <Link route={{ resource: "home" }}>
      <NodeId
        nodeId={config.publicKey}
        alias={config.alias}
        styleFontFamily="var(--font-family-sans-serif)"
        styleFontSize="var(--font-size-tiny)" />
    </Link>
    <Icon name="chevron-right" />
    {project.data.name}
    <Icon name="chevron-right" />
    Patches
  </svelte:fragment>
  <pre>
    <!-- prettier-ignore -->
    {#each patches as patch}
      - {patch.title}
    {:else}
      No patches.
    {/each}
  </pre>
</Layout>
