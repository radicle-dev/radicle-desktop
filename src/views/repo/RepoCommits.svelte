<script lang="ts">
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { COMMITS_PAGE_SIZE } from "@app/views/repo/router";
  import fuzzysort from "fuzzysort";

  import { invoke } from "@app/lib/invoke";
  import * as mutexExecutor from "@app/lib/mutexExecutor";
  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import CobCommitTeaser from "@app/components/CobCommitTeaser.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import RepoHeader from "@app/components/RepoHeader.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import SourceHeader from "@app/components/SourceHeader.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    peer?: string;
    revision?: string;
    oid: string;
    commit: Commit;
    commits: PaginatedQuery<Commit[]>;
  }

  const { repo, peer, revision, oid, commit, commits }: Props = $props();

  const baseRoute = $derived({
    resource: "repo.commits" as const,
    rid: repo.rid,
    peer,
    revision,
  });

  type CommitGroup = {
    key: string;
    label: string;
    commits: Commit[];
  };

  // Parent reuses this component across navigations; a sibling $effect resets
  // pagination state when the commits prop changes.
  // svelte-ignore state_referenced_locally
  let items = $state(commits.content);
  // svelte-ignore state_referenced_locally
  let more = $state(commits.more);
  let loadingMore = $state(false);
  let loading = $state(false);
  let searchInput = $state("");
  let debouncedSearch = $state("");
  let showSearch = $state(false);

  const loader = mutexExecutor.create();
  const abort = async (): Promise<undefined> => undefined;

  $effect(() => {
    items = commits.content;
    more = commits.more;
    // Abort any in-flight loadMoreContent so it cannot append a page
    // from the previous navigation onto the just-reset items.
    void loader.run(abort);
  });

  $effect(() => {
    const value = searchInput;
    const timer = setTimeout(() => {
      debouncedSearch = value;
    }, 150);
    return () => clearTimeout(timer);
  });

  async function loadMoreContent(all: boolean = false): Promise<void> {
    if (!more) return;
    loadingMore = true;
    try {
      const page = await loader.run(async () => {
        return await invoke<PaginatedQuery<Commit[]>>("list_repo_commits", {
          rid: repo.rid,
          head: oid,
          skip: all ? 0 : items.length,
          take: all ? undefined : COMMITS_PAGE_SIZE,
        });
      });

      // Superseded by a newer load (e.g. fuzzy-focus triggered a load-all).
      // Leave items/more alone for the new call.
      if (page === undefined) return;

      more = page.more;
      items = all ? page.content : [...items, ...page.content];
      if (page.content.length === 0) more = false;
    } finally {
      loadingMore = false;
    }
  }

  function dayKey(timestamp: number) {
    const date = new Date(timestamp);
    const month = `${date.getMonth() + 1}`.padStart(2, "0");
    const day = `${date.getDate()}`.padStart(2, "0");

    return `${date.getFullYear()}-${month}-${day}`;
  }

  function dayLabel(timestamp: number) {
    const date = new Date(timestamp);
    const today = new Date();
    const yesterday = new Date(
      today.getFullYear(),
      today.getMonth(),
      today.getDate() - 1,
    );

    if (dayKey(date.getTime()) === dayKey(today.getTime())) {
      return "Today";
    }
    if (dayKey(date.getTime()) === dayKey(yesterday.getTime())) {
      return "Yesterday";
    }

    return new Intl.DateTimeFormat("en", {
      weekday: "long",
      month: "long",
      day: "numeric",
      year: "numeric",
    }).format(date);
  }

  const searchableCommits = $derived(
    items.map(c => ({
      commit: c,
      id: c.id,
      summary: c.summary,
      author: c.author.name,
    })),
  );

  const searchResults = $derived(
    fuzzysort.go(debouncedSearch, searchableCommits, {
      keys: ["summary", "id", "author"],
      threshold: 0.5,
      all: true,
    }),
  );

  const filteredCommits = $derived(searchResults.map(r => r.obj.commit));

  const groupedCommits = $derived.by<CommitGroup[]>(() => {
    const groups: Record<string, CommitGroup> = {};

    for (const commit of filteredCommits) {
      const timestamp = commit.committer.time * 1000;
      const key = dayKey(timestamp);
      const current = groups[key];

      if (current) {
        current.commits.push(commit);
      } else {
        groups[key] = {
          key,
          label: dayLabel(timestamp),
          commits: [commit],
        };
      }
    }

    return Object.values(groups);
  });

  type CommitRow =
    | { type: "header"; key: string; label: string }
    | { type: "commit"; commit: Commit; first: boolean; last: boolean };

  // Flatten the day groups into a single row stream so the list can be
  // virtualized; the row snippet renders headers and commits differently.
  // `first`/`last` mark a group's boundaries so its commits form a bordered card.
  const commitRows = $derived.by<CommitRow[]>(() => {
    const rows: CommitRow[] = [];
    for (const group of groupedCommits) {
      rows.push({ type: "header", key: group.key, label: group.label });
      group.commits.forEach((commit, i) => {
        rows.push({
          type: "commit",
          commit,
          first: i === 0,
          last: i === group.commits.length - 1,
        });
      });
    }
    return rows;
  });
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .group-title {
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
    padding: 1rem 1rem 0.75rem;
  }
  .commit-item {
    margin: 0 1rem;
    padding: 0.625rem 1rem;
    cursor: pointer;
    background: var(--color-surface-canvas);
    border: 1px solid var(--color-border-subtle);
    border-bottom: none;
  }
  .commit-item.first {
    border-top-left-radius: var(--border-radius-md);
    border-top-right-radius: var(--border-radius-md);
  }
  .commit-item.last {
    border-bottom: 1px solid var(--color-border-subtle);
    border-bottom-left-radius: var(--border-radius-md);
    border-bottom-right-radius: var(--border-radius-md);
  }
  .commit-item:hover {
    background: var(--color-surface-subtle);
  }
</style>

<Layout selfScroll>
  <div class="page">
    <RepoHeader {repo} />
    <SourceHeader
      {repo}
      {peer}
      {revision}
      {oid}
      {commit}
      {baseRoute}
      active="commits">
      {#snippet extra()}
        <FuzzySearch
          hasItems={items.length > 0}
          placeholder={`Fuzzy filter commits ${modifierKey()} + f`}
          styleHeight="1.75rem"
          icon={loading ? "clock" : "filter"}
          onFocus={async () => {
            try {
              loading = true;
              await loadMoreContent(true);
            } catch (e) {
              console.error("Loading all commits failed: ", e);
            } finally {
              loading = false;
            }
          }}
          onSubmit={() => {
            if (filteredCommits.length === 1) {
              void router.push({
                resource: "repo.commit",
                rid: repo.rid,
                commit: filteredCommits[0].id,
              });
            }
          }}
          bind:show={showSearch}
          bind:value={searchInput} />
      {/snippet}
    </SourceHeader>
    <ScrollArea style="flex: 1; min-height: 0;">
      {#if filteredCommits.length === 0}
        <div
          class="global-flex"
          style:flex="1"
          style:justify-content="center"
          style:align-items="center">
          <div
            class="txt-missing txt-body-m-regular global-flex"
            style:gap="0.25rem">
            {#if items.length > 0}
              No matching commits
            {:else}
              No commits
            {/if}
          </div>
        </div>
      {:else}
        <VirtualList
          items={commitRows}
          hasMore={more}
          {loadingMore}
          onLoadMore={() => loadMoreContent()}
          getKey={r =>
            r.type === "header" ? `h:${r.key}` : `c:${r.commit.id}`}>
          {#snippet row(item)}
            {#if item.type === "header"}
              <div class="group-title">{item.label}</div>
            {:else}
              <div
                class="commit-item"
                class:first={item.first}
                class:last={item.last}
                role="button"
                tabindex="0"
                onclick={() => {
                  void router.push({
                    resource: "repo.commit",
                    rid: repo.rid,
                    commit: item.commit.id,
                  });
                }}
                onkeydown={e => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    void router.push({
                      resource: "repo.commit",
                      rid: repo.rid,
                      commit: item.commit.id,
                    });
                  }
                }}>
                <CobCommitTeaser
                  commit={item.commit}
                  disabled={false}
                  flush
                  hoverable />
              </div>
            {/if}
          {/snippet}
        </VirtualList>
      {/if}
    </ScrollArea>
  </div>
</Layout>
