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
  .header {
    position: sticky;
    top: 0;
  }
  .repo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(21rem, 1fr));
    gap: 1rem;
    padding: 1rem;
  }
</style>

<div style:height="fit-content">
  <div class="header">
    <Header>
      <svelte:fragment slot="center">
        <CopyableId id={`did:key:${config.publicKey}`} />
      </svelte:fragment>
    </Header>
  </div>
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
