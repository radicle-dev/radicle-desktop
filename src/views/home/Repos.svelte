<script lang="ts">
  import type { HomeReposTab } from "@app/lib/router/definitions";
  import type { Config } from "@bindings/config/Config";
  import type { NotificationCount } from "@bindings/cob/inbox/NotificationCount";
  import type { RepoCount } from "@bindings/repo/RepoCount";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";
  import { didFromPublicKey } from "@app/lib/utils";

  import CopyableId from "@app/components/CopyableId.svelte";
  import HomeSidebar from "@app/components/HomeSidebar.svelte";
  import Layout from "@app/views/repo/Layout.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";
  import Border from "@app/components/Border.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    activeTab?: HomeReposTab;
    config: Config;
    notificationCount: Map<string, NotificationCount>;
    repoCount: RepoCount;
    repos: RepoInfo[];
  }

  /* eslint-disable prefer-const */
  let { config, repos, notificationCount, repoCount, activeTab }: Props =
    /* eslint-enable prefer-const */
    $props();
</script>

<style>
  .container {
    padding: 1rem 1rem 1rem 0;
  }
  .repo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(21rem, 1fr));
    gap: 1rem;
  }
  .header {
    font-weight: var(--font-weight-medium);
    font-size: var(--font-size-medium);
    display: flex;
    justify-content: space-between;
    padding-right: 1.5rem;
    align-items: center;
    min-height: 40px;
  }
</style>

<Layout
  hideSidebar
  styleSecondColumnOverflow="visible"
  publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={config.publicKey} />
  {/snippet}
  {#snippet secondColumn()}
    <HomeSidebar
      activeTab={{ type: "repos", filter: activeTab }}
      {repoCount}
      {notificationCount} />
  {/snippet}
  <div class="container">
    <div class="header">Repositories</div>
    <div class="repo-grid">
      {#each repos as repo}
        {#if repo.payloads["xyz.radicle.project"]}
          <RepoCard
            {repo}
            selfDid={didFromPublicKey(config.publicKey)}
            onclick={() => {
              void router.push({
                resource: "repo.issues",
                rid: repo.rid,
                status: "open",
              });
            }} />
        {/if}
      {:else}
        <Border
          variant="ghost"
          styleAlignItems="center"
          styleJustifyContent="center">
          <div
            class="global-flex"
            style:height="74px"
            style:justify-content="center">
            <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
              <Icon name="none" />
              No repositories.
            </div>
          </div>
        </Border>
      {/each}
    </div>
  </div>
</Layout>
