<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";
  import type { HomeReposTab } from "@app/views/home/router";
  import type { RepoCount } from "@bindings/repo/RepoCount";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import fuzzysort from "fuzzysort";
  import { onMount } from "svelte";

  import * as router from "@app/lib/router";
  import { didFromPublicKey, modifierKey } from "@app/lib/utils";
  import { dynamicInterval } from "@app/lib/interval";
  import { invoke } from "@app/lib/invoke";
  import { setFocused } from "@app/components/Popover.svelte";

  import Border from "@app/components/Border.svelte";
  import HomeSidebar from "@app/components/HomeSidebar.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Layout from "@app/views/repo/Layout.svelte";
  import NodeBreadcrumb from "@app/components/NodeBreadcrumb.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    activeTab: HomeReposTab;
    config: Config;
    repoCount: RepoCount;
    repos: RepoInfo[];
    notificationCount: number;
  }

  /* eslint-disable prefer-const */
  let { config, repos, repoCount, activeTab, notificationCount }: Props =
    /* eslint-enable prefer-const */
    $props();

  let lock = false;
  const startup = $state<{ error?: ErrorWrapper }>({ error: undefined });

  async function checkRepos() {
    try {
      if (lock) {
        return;
      }
      if (repoCount.total > 0) {
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
      invoke<RepoInfo[]>("list_repos", { show: activeTab ?? "all" }),
      invoke<RepoCount>("repo_count"),
      invoke<Config>("config"),
    ]);
  }

  let searchInput = $state("");

  const searchableRepos = $derived(
    repos
      .flatMap(r => {
        if (r.payloads["xyz.radicle.project"]) {
          return { repo: r, name: r.payloads["xyz.radicle.project"].data.name };
        }
      })
      .filter((item): item is NonNullable<typeof item> => item !== undefined),
  );

  const searchResults = $derived(
    fuzzysort.go(searchInput, searchableRepos, {
      keys: ["name", "repo.rid"],
      threshold: 0.5,
      all: true,
    }),
  );
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
    min-height: 2.5rem;
  }
</style>

<Layout
  {notificationCount}
  hideSidebar
  styleSecondColumnOverflow="visible"
  {config}>
  {#snippet breadcrumbs()}
    <NodeBreadcrumb {config} />
  {/snippet}

  {#snippet secondColumn()}
    <HomeSidebar {activeTab} {repoCount} />
  {/snippet}
  <div class="container">
    <div class="global-flex" style:margin-bottom="1rem">
      <div class="header">Repositories</div>
      {#if repos.length > 0}
        <div class="global-flex" style:margin-left="auto">
          <TextInput
            onSubmit={async () => {
              if (searchResults.length === 1) {
                await router.push({
                  resource: "repo.home",
                  rid: searchResults[0].obj.repo.rid,
                });
              }
            }}
            onDismiss={() => {
              searchInput = "";
            }}
            placeholder={`Fuzzy filter repositories ${modifierKey()} + f`}
            keyShortcuts="ctrl+f"
            bind:value={searchInput}>
            {#snippet left()}
              <div
                style:color="var(--color-foreground-dim)"
                style:padding-left="0.5rem">
                <Icon name="filter" />
              </div>
            {/snippet}
          </TextInput>
        </div>
      {/if}
    </div>
    {#if repoCount.total > 0}
      {#if searchResults.length > 0}
        <div class="repo-grid">
          {#each searchResults as result}
            <RepoCard
              focussed={searchResults.length === 1 && searchInput !== ""}
              repo={result.obj.repo}
              selfDid={didFromPublicKey(config.publicKey)}
              onclick={() => {
                void router.push({
                  resource: "repo.home",
                  rid: result.obj.repo.rid,
                });
              }} />
          {/each}
        </div>
      {:else}
        <Border
          variant="ghost"
          styleAlignItems="center"
          styleJustifyContent="center">
          <div
            class="global-flex"
            style:height="7.875rem"
            style:justify-content="center">
            <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
              <Icon name="none" />
              {#if repos.length > 0}
                No matching repositories.
              {:else}
                No repositories.
              {/if}
            </div>
          </div>
        </Border>
      {/if}
    {:else}
      <div class="txt-missing txt-small" style:margin-bottom="1.5rem">
        You don't have any repositories in your Radicle storage yet. To get
        started, check out the guide below.
      </div>
      <div style="display: flex; gap: 1rem;">
        <OutlineButton
          popoverToggle="popover-guide"
          onclick={() => setFocused("popover-guide")}
          variant="ghost">
          <Icon name="info" />Guide
        </OutlineButton>
      </div>
    {/if}
  </div>
</Layout>
