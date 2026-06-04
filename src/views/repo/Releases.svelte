<script lang="ts">
  import type { ReleaseFilter } from "@app/views/repo/router";
  import type { Release } from "@bindings/cob/release/Release";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import fuzzysort from "fuzzysort";

  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import Icon from "@app/components/Icon.svelte";
  import ReleaseTeaser from "@app/components/ReleaseTeaser.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";

  import Layout from "./Layout.svelte";
  import NewRelease from "./NewRelease.svelte";

  interface Props {
    repo: RepoInfo;
    releases: Release[];
    filter: ReleaseFilter;
  }

  const { repo, releases, filter }: Props = $props();

  let showNew = $state(false);
  let searchInput = $state("");
  let showSearch = $state(false);

  async function setFilter(next: ReleaseFilter) {
    await router.push({
      resource: "repo.releases",
      rid: repo.rid,
      filter: next,
    });
  }

  const delegateDids = $derived(repo.delegates.map(d => d.did));

  // A redaction is "trusted" when its peer is the artifact author or a
  // repo delegate. The detail view already blurs the artifact in that
  // case; here we hide the whole release if every artifact has been
  // redacted that way so the list isn't cluttered with withdrawn rows.
  function isFullyRedacted(release: (typeof releases)[number]): boolean {
    if (release.artifacts.length === 0) return false;
    return release.artifacts.every(a =>
      a.redactions.some(
        r => r.peer.did === a.author.did || delegateDids.includes(r.peer.did),
      ),
    );
  }

  // Releases without artifacts are placeholders the user hasn't published
  // anything to yet — keep them off the list so it shows actual deliverables.
  const baseReleases = $derived(
    releases.filter(r => r.artifacts.length > 0 && !isFullyRedacted(r)),
  );

  // Pre-compute the delegate set so the counter badges stay in sync with
  // whatever the active filter is showing.
  const delegateReleases = $derived(
    baseReleases.filter(r => delegateDids.includes(r.creator.did)),
  );

  const visibleReleases = $derived(
    filter === "delegate" ? delegateReleases : baseReleases,
  );

  const searchable = $derived(
    visibleReleases.map(release => ({
      release,
      creator: release.creator.alias ?? "",
      tagName: release.tagName ?? "",
      summary: release.commitSummary ?? "",
      artifactNames: release.artifacts.map(a => a.name).join(" "),
    })),
  );

  const searchResults = $derived(
    fuzzysort.go(searchInput, searchable, {
      keys: ["release.oid", "tagName", "summary", "creator", "artifactNames"],
      threshold: 0.5,
      all: true,
    }),
  );
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .topbar-title {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-secondary);
    padding-right: 0.25rem;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-height: 100%;
  }
  .filters {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .filter {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
    padding: 0.25rem 0.5rem;
    border-radius: var(--border-radius-sm);
    background: none;
    border: none;
    cursor: pointer;
    white-space: nowrap;
  }
  .filter:hover {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .filter.active {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .filter-label {
    display: none;
  }
  .filter.active .filter-label {
    display: inline;
  }
  @media (min-width: 1011px) {
    .filter-label {
      display: inline;
    }
  }
</style>

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <span class="topbar-title">Releases</span>
      <div class="filters">
        <button
          class="filter"
          class:active={filter === "delegate"}
          onclick={() => void setFilter("delegate")}
          title="Only releases authored by repo delegates">
          <Icon name="badge" />
          <span class="filter-label">Delegates</span>
          <span class="global-counter-badge">{delegateReleases.length}</span>
        </button>
        <button
          class="filter"
          class:active={filter === "all"}
          onclick={() => void setFilter("all")}
          title="Every release published to this repo">
          <Icon name="archive" />
          <span class="filter-label">All</span>
          <span class="global-counter-badge">{baseReleases.length}</span>
        </button>
      </div>
      <div class="global-flex" style:margin-left="auto" style:gap="0.5rem">
        <FuzzySearch
          hasItems={visibleReleases.length > 0}
          placeholder={`Fuzzy filter releases ${modifierKey()} + f`}
          bind:show={showSearch}
          bind:value={searchInput} />
        <Button
          variant="secondary"
          styleHeight="2rem"
          onclick={() => (showNew = !showNew)}
          active={showNew}>
          <Icon name={showNew ? "close" : "plus"} />
          <span class="global-hide-on-small-desktop-down">
            {showNew ? "Cancel" : "New release"}
          </span>
        </Button>
      </div>
    </Topbar>

    {#if showNew}
      <div style:padding="1rem">
        <NewRelease {repo} onCancel={() => (showNew = false)} />
      </div>
    {/if}

    <ScrollArea style="height: 100%; min-width: 0;">
      <div class="list">
        {#each searchResults as result (result.obj.release.id)}
          <ReleaseTeaser release={result.obj.release} rid={repo.rid} {filter} />
        {/each}

        {#if searchResults.length === 0}
          <div
            class="global-flex"
            style:flex="1"
            style:justify-content="center"
            style:align-items="center">
            <div class="txt-missing txt-body-m-regular">
              {#if visibleReleases.length > 0}
                No matching releases
              {:else if filter === "delegate" && baseReleases.length > 0}
                No releases from delegates yet
              {:else}
                No releases yet
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </ScrollArea>
  </div>
</Layout>
