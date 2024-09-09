<script lang="ts">
  import type { Patch } from "@bindings/Patch";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Layout from "./Layout.svelte";

  export let repo: RepoInfo;
  export let patches: Patch[];
</script>

<Layout {repo}>
  <pre>
    <!-- prettier-ignore -->
    {#each patches as patch}
      - {patch.title}
    {:else}
      No patches.
    {/each}
  </pre>
</Layout>
