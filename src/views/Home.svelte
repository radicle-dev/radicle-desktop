<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import * as router from "@app/lib/router";

  import CopyableId from "@app/components/CopyableId.svelte";
  import Header from "@app/components/Header.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  export let repos: RepoInfo[];
  export let config: Config;
</script>

<style>
  .header {
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .repo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(21rem, 1fr));
    gap: 1rem;
  }

  .hero-image {
    background-image: url("aliens.png");
    background-position: center;
    background-size: cover;
    height: 9.5rem;
    clip-path: var(--3px-corner-fill);
  }
</style>

<div style:height="fit-content">
  <div class="header">
    <Header>
      <svelte:fragment slot="center">
        <CopyableId id={`did:key:${config.publicKey}`} />
      </svelte:fragment>
      <svelte:fragment slot="breadcrumbs">
        <NodeId
          nodeId={config.publicKey}
          alias={config.alias}
          styleFontFamily="var(--font-family-sans-serif)"
          styleFontSize="var(--font-size-tiny)" />
      </svelte:fragment>
    </Header>
  </div>
  <div style:padding="1rem">
    <div class="hero-image"></div>

    <div class="txt-semibold" style:margin="1.5rem 0">Repositories</div>
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
</div>
