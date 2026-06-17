<script lang="ts">
  import type { IssueStatus } from "@app/views/repo/router";
  import type { CacheEvent } from "@bindings/cob/CacheEvent";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { DEFAULT_TAKE } from "@app/views/repo/router";
  import { Channel } from "@tauri-apps/api/core";
  import fuzzysort from "fuzzysort";
  import delay from "lodash/delay";

  import { invoke } from "@app/lib/invoke";
  import {
    issueCountMismatch,
    resetIssueCounts,
    updateIssueCounts,
  } from "@app/lib/issueCounts.svelte";
  import type { ListCacheSnapshot } from "@app/lib/listState";
  import { readListState, saveListState } from "@app/lib/listState";
  import { show } from "@app/lib/modal";
  import * as mutexExecutor from "@app/lib/mutexExecutor";
  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import CobCacheWarning from "@app/components/CobCacheWarning.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import Icon from "@app/components/Icon.svelte";
  import IssueTeaser from "@app/components/IssueTeaser.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";
  import CreateIssueModal from "@app/modals/CreateIssue.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    issues: PaginatedQuery<Issue[]>;
    status: IssueStatus;
  }

  const { repo, issues, status }: Props = $props();

  function listKey(filter: IssueStatus): string {
    return `repo.issues:${repo.rid}:${filter}`;
  }

  // Restore the prior list and scroll position only when arriving via
  // back/forward; a fresh navigation (sidebar, links) starts from the top.
  // svelte-ignore state_referenced_locally
  const restored = router.isHistoryNavigation()
    ? readListState<Issue>(listKey(status))
    : undefined;

  // Parent reuses this component across status filter changes; a sibling
  // $effect resets pagination state when the filter changes.
  // svelte-ignore state_referenced_locally
  let items = $state(restored?.items ?? issues.content);
  // Rows fetched so far — used as the pagination offset. Tracked separately
  // from items.length because appends are deduped (overlapping pages, e.g. when
  // the list grows underneath us, must still advance the offset).
  // svelte-ignore state_referenced_locally
  let cursor = $state((restored?.items ?? issues.content).length);
  // svelte-ignore state_referenced_locally
  let more = $state(restored?.more ?? issues.more);
  const initialScrollOffset = restored?.scrollOffset;
  const initialCache = restored?.cache;
  // svelte-ignore state_referenced_locally
  let activeKey = listKey(status);
  let loadingMore = $state(false);
  let loading = $state(false);
  let searchInput = $state("");
  let debouncedSearch = $state("");
  let showSearch = $state(false);
  let cacheState: CacheEvent | undefined = $state();
  // Height of the cache-warning banner above the list; fed to the virtualizer
  // as a start offset since it shares the same scroll viewport.
  let chromeHeight = $state(0);

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  const loader = mutexExecutor.create();
  const abort = async (): Promise<undefined> => undefined;

  $effect(() => {
    const key = listKey(status);
    const fresh = issues;
    // Skip the initial mount (state is seeded above, possibly restored); only
    // reset when the filter actually changes.
    if (key === activeKey) return;
    activeKey = key;
    items = fresh.content;
    cursor = fresh.content.length;
    more = fresh.more;
    // Abort any in-flight loadMoreContent so it cannot append a page
    // from the previous filter onto the just-reset items.
    void loader.run(abort);
  });

  // Persist the loaded list + scroll position so back/forward can restore it.
  // Only the unfiltered list is cached, so its length matches virtua's size
  // snapshot.
  function persistScroll(state: {
    scrollOffset: number;
    cache: ListCacheSnapshot;
  }) {
    if (searchInput !== "") return;
    saveListState(listKey(status), {
      items: [...items],
      more,
      scrollOffset: state.scrollOffset,
      cache: state.cache,
    });
  }

  $effect(() => {
    if (more === false) {
      updateIssueCounts(
        items.length,
        {
          ...project.meta.issues,
          all: project.meta.issues.open + project.meta.issues.closed,
        },
        status,
      );
    }
  });

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    status;

    searchInput = "";
    showSearch = false;
  });

  $effect(() => {
    const value = searchInput;
    const timer = setTimeout(() => {
      debouncedSearch = value;
    }, 150);
    return () => clearTimeout(timer);
  });

  async function rebuildIssueCache() {
    try {
      await invoke("rebuild_issue_cache", {
        rid: repo.rid,
        onEvent: new Channel<CacheEvent>(message => {
          cacheState = message;
        }),
      });
    } catch (error) {
      console.error(error);
    } finally {
      const page = await loader.run(async () => {
        return await invoke<PaginatedQuery<Issue[]>>("list_issues", {
          rid: repo.rid,
          skip: 0,
          status,
          take: DEFAULT_TAKE,
        });
      });

      if (page !== undefined) {
        items = page.content;
        cursor = page.content.length;
        more = page.more;
      }

      resetIssueCounts();

      delay(() => {
        cacheState = undefined;
      }, 1500);
    }
  }

  async function loadMoreContent(all: boolean = false): Promise<void> {
    if (!more) return;
    loadingMore = true;
    let superseded = false;
    try {
      const page = await loader.run(async () => {
        return await invoke<PaginatedQuery<Issue[]>>("list_issues", {
          rid: repo.rid,
          status,
          skip: all ? 0 : cursor,
          take: all ? undefined : DEFAULT_TAKE,
        });
      });

      // Superseded by a newer load (e.g. fuzzy-focus triggered a load-all).
      // Leave items/more alone for the new call. The flag stays set too: the
      // newer call owns it now, and clearing it here would let the
      // virtualizer's auto load-more re-fire and abort that call in turn.
      if (page === undefined) {
        superseded = true;
        return;
      }

      more = page.more;
      if (all) {
        items = page.content;
        cursor = page.content.length;
      } else {
        // Drop ids already shown so duplicate keys never reach the virtual
        // list (which would leave blank, persistent gaps); still advance the
        // offset by the rows fetched so paging keeps moving forward.
        const seen = new Set(items.map(i => i.id));
        items = [...items, ...page.content.filter(i => !seen.has(i.id))];
        cursor += page.content.length;
      }
      if (page.content.length === 0) more = false;
    } finally {
      if (!superseded) {
        loadingMore = false;
      }
    }
  }

  const searchableIssues = $derived(
    items
      .flatMap(i => {
        return {
          issue: i,
          labels: i.labels.join(" "),
          assignees: i.assignees
            .map(a => {
              return a.alias ?? "";
            })
            .join(" "),
          author: i.author.alias ?? "",
        };
      })
      .filter((item): item is NonNullable<typeof item> => item !== undefined),
  );

  const searchResults = $derived(
    fuzzysort.go(debouncedSearch, searchableIssues, {
      keys: ["issue.title", "labels", "assignees", "author", "issue.id"],
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

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <span class="topbar-title">Issues</span>
      <div class="filters">
        <a
          class="filter"
          class:active={status === "all"}
          href={router.routeToPath({
            resource: "repo.issues",
            rid: repo.rid,
            status: "all",
          })}>
          <Icon name="issue" />All
          <span class="global-counter-badge">
            {project.meta.issues.open + project.meta.issues.closed}
          </span>
        </a>
        <a
          class="filter"
          class:active={status === "open"}
          href={router.routeToPath({
            resource: "repo.issues",
            rid: repo.rid,
            status: "open",
          })}>
          <Icon name="issue" />Open
          <span class="global-counter-badge">{project.meta.issues.open}</span>
        </a>
        <a
          class="filter"
          class:active={status === "closed"}
          href={router.routeToPath({
            resource: "repo.issues",
            rid: repo.rid,
            status: "closed",
          })}>
          <Icon name="issue-closed" />Closed
          <span class="global-counter-badge">{project.meta.issues.closed}</span>
        </a>
      </div>
      <div class="global-flex" style:margin-left="auto" style:gap="0.5rem">
        <FuzzySearch
          hasItems={items.length > 0}
          placeholder={`Fuzzy filter issues ${modifierKey()} + f`}
          icon={loading ? "clock" : "filter"}
          onFocus={async () => {
            try {
              loading = true;
              await loadMoreContent(true);
            } catch (e) {
              console.error("Loading all issues failed: ", e);
            } finally {
              loading = false;
            }
          }}
          onSubmit={async () => {
            if (searchResults.length === 1) {
              await router.push({
                resource: "repo.issue",
                rid: repo.rid,
                issue: searchResults[0].obj.issue.id,
                status,
              });
            }
          }}
          bind:show={showSearch}
          bind:value={searchInput} />
        <Button
          styleHeight="2rem"
          variant="secondary"
          onclick={() =>
            show({
              component: CreateIssueModal,
              props: { repo },
            })}>
          <Icon name="plus" />New issue
        </Button>
      </div>
    </Topbar>

    {#if searchResults.length === 0}
      {#if issueCountMismatch(status)}
        <CobCacheWarning
          noun="issues"
          {cacheState}
          onRebuild={rebuildIssueCache} />
      {/if}
      <div
        class="global-flex"
        style:flex="1"
        style:justify-content="center"
        style:align-items="center">
        <div
          class="txt-missing txt-body-m-regular global-flex"
          style:gap="0.25rem">
          {#if items.length > 0}
            No matching issues
          {:else}
            No {status === "all" ? "" : status} issues
          {/if}
        </div>
      </div>
    {:else}
      <ScrollArea style="height: 100%; min-width: 0;">
        <div bind:clientHeight={chromeHeight}>
          {#if issueCountMismatch(status)}
            <CobCacheWarning
              noun="issues"
              {cacheState}
              onRebuild={rebuildIssueCache} />
          {/if}
        </div>
        <VirtualList
          items={searchResults}
          hasMore={more}
          {loadingMore}
          onLoadMore={() => loadMoreContent()}
          startMargin={chromeHeight}
          estimatedItemSize={80}
          getKey={result => result.obj.issue.id}
          {initialCache}
          {initialScrollOffset}
          onState={persistScroll}>
          {#snippet row(result)}
            <div class="row">
              <IssueTeaser
                focussed={searchResults.length === 1 && searchInput !== ""}
                issue={result.obj.issue}
                rid={repo.rid}
                {status} />
            </div>
          {/snippet}
        </VirtualList>
      </ScrollArea>
    {/if}
  </div>
</Layout>
