<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { Issue } from "@bindings/Issue";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Icon from "@app/components/Icon.svelte";
  import Layout from "./Layout.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  export let repo: RepoInfo;
  export let issues: Issue[];
  export let config: Config;

  $: project = repo.payloads["xyz.radicle.project"]!;
</script>

<Layout {repo}>
  <svelte:fragment slot="breadcrumbs">
    <NodeId
      nodeId={config.publicKey}
      alias={config.alias}
      styleFontFamily="var(--font-family-sans-serif)"
      styleFontSize="var(--font-size-tiny)" />
    <Icon name="chevron-right" />
    {project.data.name}
    <Icon name="chevron-right" />
    Issues
  </svelte:fragment>

  <pre>
    <!-- prettier-ignore -->
    {#each issues as issue}
      - {issue.title}
    {:else}
      No issues.
    {/each}
  </pre>
</Layout>
