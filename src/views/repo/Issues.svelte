<script lang="ts">
  import type { Issue } from "@bindings/Issue";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Layout from "./Layout.svelte";

  export let repo: RepoInfo;
  export let issues: Issue[];
</script>

<Layout {repo}>
  <pre>
    <!-- prettier-ignore -->
    {#each issues as issue}
      - {issue.title}
    {:else}
      No issues.
    {/each}
  </pre>
</Layout>
