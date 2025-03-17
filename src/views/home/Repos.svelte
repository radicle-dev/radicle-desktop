<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";
  import type { HomeReposTab } from "@app/lib/router/definitions";
  import type { Config } from "@bindings/config/Config";
  import type { NotificationCount } from "@bindings/cob/inbox/NotificationCount";
  import type { RepoCount } from "@bindings/repo/RepoCount";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";
  import { didFromPublicKey } from "@app/lib/utils";
  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";
  import { onMount } from "svelte";

  import CopyableId from "@app/components/CopyableId.svelte";
  import HomeSidebar from "@app/components/HomeSidebar.svelte";
  import Layout from "@app/views/repo/Layout.svelte";
  import Onboarding from "@app/views/home/Onboarding.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";

  interface Props {
    activeTab?: HomeReposTab;
    config: Config;
    notificationCount: Map<string, NotificationCount>;
    repoCount: RepoCount;
    repos: RepoInfo[];
  }

  /* eslint-disable prefer-const */
  let {
    config,
    repos: initialRepos,
    notificationCount,
    repoCount,
    activeTab,
  }: Props =
    /* eslint-enable prefer-const */
    $props();

  let repos = $state(initialRepos);
  let lock = false;
  const startup = $state<{ error?: ErrorWrapper }>({ error: undefined });

  async function checkRepos() {
    try {
      if (lock) {
        return;
      }
      if (repos.length > 0) {
        return;
      }
      lock = true;
      await reload();
    } catch (err) {
      const error = err as ErrorWrapper;
      startup.error = error;
    } finally {
      lock = false;
    }
  }

  onMount(() => {
    dynamicInterval("repos", checkRepos, 5_000);
  });

  async function reload() {
    [repos, repoCount, config] = await Promise.all([
      invoke<RepoInfo[]>("list_repos", { show: "all" }),
      invoke<RepoCount>("repo_count"),
      invoke<Config>("config"),
    ]);
  }
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
    margin-bottom: 0.5rem;
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
    {#if repos.length > 0}
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
        {/each}
      </div>
    {:else}
      <Onboarding {reload} />
    {/if}
  </div>
</Layout>
