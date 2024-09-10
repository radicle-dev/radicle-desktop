<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import * as router from "@app/lib/router";

  import CopyableId from "@app/components/CopyableId.svelte";
  import Header from "@app/components/Header.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";

  export let repos: RepoInfo[];
  export let config: Config;
</script>

<style>
  .layout {
    padding: 1rem;
  }
  .repo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(21rem, 1fr));
    gap: 1rem;
  }
</style>

<Header>
  <svelte:fragment slot="center">
    <CopyableId id={`did:key:${config.publicKey}`} />
  </svelte:fragment>
</Header>
<div class="layout">
  <div class="repo-grid">
    {#each repos as repo}
      {#if repo.payloads["xyz.radicle.project"]}
        <RepoCard
          {repo}
          selfDid={`did:key:${config.publicKey}`}
          onclick={() => {
            void router.push({
              resource: "repo.issues",
              rid: repo.rid,
              status: "open",
            });
          }} />
      {/if}
    {/each}
  </div>
</div>
