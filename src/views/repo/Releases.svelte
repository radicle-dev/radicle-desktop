<script lang="ts">
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Release } from "@bindings/cob/release/Release";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { RELEASES_PER_PAGE } from "@app/views/repo/router";

  import { invoke } from "@app/lib/invoke";
  import { modalStore, show } from "@app/lib/modal";
  import { createPaginatedList } from "@app/lib/paginatedList.svelte";
  import * as router from "@app/lib/router";
  import { isMac } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import ReleaseTeaser from "@app/components/ReleaseTeaser.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";
  import CreateReleaseModal from "@app/modals/CreateRelease.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    releases: PaginatedQuery<Release[]>;
    releaseCount: number;
    allAuthors: boolean;
  }

  const { repo, releases, releaseCount, allAuthors }: Props = $props();

  const delegateIds = $derived(new Set(repo.delegates.map(d => d.did)));

  function openCreateRelease() {
    show({ component: CreateReleaseModal, props: { repo } });
  }

  const list = createPaginatedList<Release>({
    key: () => `repo.releases:${repo.rid}:${allAuthors}`,
    page: () => releases,
    fetchPage: (skip, take) =>
      invoke<PaginatedQuery<Release[]>>("list_releases", {
        rid: repo.rid,
        filter: { allAuthors },
        skip,
        take,
      }),
    pageSize: RELEASES_PER_PAGE,
    id: release => release.id,
  });

  // The delegate-scoped count isn't in repo metadata; derive it from the pages
  // loaded so far. It labels the segment only, and reads as "30+" while more
  // pages are outstanding.
  const delegateCount = $derived(
    allAuthors
      ? list.items.filter(r => delegateIds.has(r.creator.did)).length
      : list.items.length,
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
    text-decoration: none;
    cursor: pointer;
    white-space: nowrap;
  }
  .filter:hover {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-primary);
  }
  .filter.active {
    background-color: var(--color-surface-subtle);
  }
  .filter .global-counter-badge {
    margin-left: 0.25rem;
  }
  .row {
    border-bottom: 1px solid var(--color-border-subtle);
  }
</style>

<svelte:document
  onkeydown={e => {
    const auxiliarKey = isMac() ? e.metaKey : e.ctrlKey;
    if (auxiliarKey && e.key.toLowerCase() === "n" && !$modalStore) {
      e.preventDefault();
      openCreateRelease();
    }
  }} />

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <span class="topbar-title">Releases</span>
      <div class="filters">
        <a
          class="filter"
          class:active={!allAuthors}
          href={router.routeToPath({
            resource: "repo.releases",
            rid: repo.rid,
            allAuthors: false,
          })}>
          <Icon name="badge" />Delegates
          <span class="global-counter-badge">
            {delegateCount}{list.more ? "+" : ""}
          </span>
        </a>
        <a
          class="filter"
          class:active={allAuthors}
          href={router.routeToPath({
            resource: "repo.releases",
            rid: repo.rid,
            allAuthors: true,
          })}>
          <Icon name="avatar-incognito" />All
          <span class="global-counter-badge">{releaseCount}</span>
        </a>
      </div>
      <div style:margin-left="auto">
        <Button
          styleHeight="2rem"
          variant="secondary"
          onclick={openCreateRelease}>
          <Icon name="plus" />New release
        </Button>
      </div>
    </Topbar>

    {#if list.items.length === 0}
      <div
        class="global-flex"
        style:flex="1"
        style:justify-content="center"
        style:align-items="center">
        <div class="txt-missing txt-body-m-regular">
          {allAuthors ? "No releases" : "No releases by delegates"}
        </div>
      </div>
    {:else}
      <ScrollArea style="height: 100%; min-width: 0;">
        <VirtualList
          items={list.items}
          hasMore={list.more}
          loadingMore={list.loadingMore}
          onLoadMore={() => list.loadMore()}
          estimatedItemSize={80}
          getKey={release => release.id}
          initialCache={list.initialCache}
          initialScrollOffset={list.initialScrollOffset}
          onRestored={list.consumeRestoredScroll}
          onState={list.persistScroll}>
          {#snippet row(release)}
            <div class="row">
              <ReleaseTeaser
                {release}
                {allAuthors}
                {delegateIds}
                rid={repo.rid} />
            </div>
          {/snippet}
        </VirtualList>
      </ScrollArea>
    {/if}
  </div>
</Layout>
