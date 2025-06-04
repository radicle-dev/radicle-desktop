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
  import { guidePopoverToggleId } from "@app/components/GuideButton.svelte";
  import { invoke } from "@app/lib/invoke";
  import { sleep } from "@app/lib/sleep";

  import AddRepoButton from "@app/components/AddRepoButton.svelte";
  import Border from "@app/components/Border.svelte";
  import HomeSidebar from "@app/components/HomeSidebar.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Layout from "@app/views/repo/Layout.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import NodeBreadcrumb from "@app/components/NodeBreadcrumb.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";
  import RepoCardPlaceholder from "@app/components/RepoCardPlaceholder.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    activeTab: HomeReposTab;
    config: Config;
    notificationCount: number;
    repoCount: RepoCount;
    repos: RepoInfo[];
    seededNotReplicated: string[];
  }

  /* eslint-disable prefer-const */
  let {
    activeTab,
    config,
    notificationCount,
    repoCount,
    repos,
    seededNotReplicated,
  }: Props =
    /* eslint-enable prefer-const */
    $props();

  const startup = $state<{ error?: ErrorWrapper }>({ error: undefined });
  let showFilters: boolean = $state(false);
  let searchInput = $state("");

  let lock = false;

  async function reloadRepoList() {
    try {
      if (lock) {
        return;
      }
      if (seededNotReplicated.length === 0 && repoCount.total > 0) {
        return;
      }
      if (searchInput !== "") {
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
    dynamicInterval("repos", reloadRepoList, 5_000);
  });

  async function reload() {
    [repos, repoCount, config, seededNotReplicated] = await Promise.all([
      invoke<RepoInfo[]>("list_repos", { show: activeTab ?? "all" }),
      invoke<RepoCount>("repo_count"),
      invoke<Config>("config"),
      invoke<string[]>("seeded_not_replicated"),
    ]);
  }

  const searchableRepos = $derived(
    repos
      .flatMap(r => {
        if (r.payloads["xyz.radicle.project"]) {
          return {
            repo: r,
            name: r.payloads["xyz.radicle.project"].data.name,
            description: r.payloads["xyz.radicle.project"].data.description,
          };
        }
      })
      .filter((item): item is NonNullable<typeof item> => item !== undefined),
  );

  const searchResults = $derived(
    fuzzysort.go(searchInput, searchableRepos, {
      keys: ["name", "description", "repo.rid"],
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
    padding-right: 0.325rem;
    align-items: center;
    min-height: 2.5rem;
  }
  button {
    text-decoration: underline;
    border: 0;
    color: var(--color-foreground-dim);
    margin: 0;
    padding: 0;
    background-color: transparent;
    cursor: pointer;
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
      <div class="global-flex">
        <div class="header">Repositories</div>
        {#if repos.length > 0}
          {#if !showFilters}
            <NakedButton
              styleHeight="2.5rem"
              keyShortcuts="ctrl+f"
              variant="ghost"
              active={showFilters}
              onclick={() => {
                if (showFilters) {
                  showFilters = false;
                  searchInput = "";
                } else {
                  showFilters = true;
                }
              }}>
              <Icon name="filter" />
            </NakedButton>
          {/if}
          {#if showFilters}
            <TextInput
              autofocus
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
                showFilters = false;
              }}
              onBlur={() => {
                if (searchInput.trim() === "") {
                  showFilters = false;
                }
              }}
              placeholder={`Fuzzy filter repositories ${modifierKey()} + f`}
              keyShortcuts="ctrl+f"
              bind:value={searchInput}>
              {#snippet left()}
                <div style:padding-left="0.5rem">
                  <Icon name="filter" />
                </div>
              {/snippet}
            </TextInput>
          {/if}
        {/if}
      </div>
      <div class="global-flex" style:margin-left="auto">
        <AddRepoButton
          {repos}
          {reload}
          {seededNotReplicated}
          onOpen={() => {
            searchInput = "";
            showFilters = false;
          }} />
      </div>
    </div>
    {#if repoCount.total > 0 || seededNotReplicated.length > 0}
      {#if searchResults.length > 0 || seededNotReplicated.length > 0}
        <div class="repo-grid">
          {#if !showFilters}
            {#each seededNotReplicated as rid}
              <RepoCardPlaceholder {rid} {reload} />
            {/each}
          {/if}
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
      <div class="txt-missing txt-small">
        You don't have any repositories in your Radicle storage yet.
      </div>
      <!-- prettier-ignore -->
      <div class="txt-missing txt-small">
        To get started, check out
        <button
          class="txt-small"
          onclick={async () => {
                const guidePopoverButton = document.getElementById(guidePopoverToggleId);
                await sleep(1);
                guidePopoverButton?.click();
          }}>
          the guide
        </button>.
      </div>
    {/if}
  </div>
</Layout>
