<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";

  import CopyableId from "@app/components/CopyableId.svelte";
  import Header from "@app/components/Header.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";
  import Settings from "@app/components/Settings.svelte";

  interface Props {
    repos: RepoInfo[];
    config: Config;
  }

  const { repos, config }: Props = $props();
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
</style>

<div style:height="fit-content">
  <div class="header">
    <Header publicKey={config.publicKey}>
      {#snippet center()}
        <CopyableId id={`did:key:${config.publicKey}`} />
      {/snippet}
      {#snippet settingsButton()}
        <Settings
          styleHeight="32px"
          popoverProps={{
            popoverPositionRight: "0",
            popoverPositionTop: "2.5rem",
          }} />
      {/snippet}
    </Header>
  </div>
  <div style:padding="1rem">
    <div class="txt-semibold" style:margin="0.5rem 0 1.5rem 1rem">
      Repositories
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
</div>
