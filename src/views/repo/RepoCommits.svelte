<script lang="ts">
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { COMMITS_PAGE_SIZE } from "@app/views/repo/router";
  import fuzzysort from "fuzzysort";

  import { cachedRepoCommitCount } from "@app/lib/invoke";
  import { invoke } from "@app/lib/invoke";
  import * as mutexExecutor from "@app/lib/mutexExecutor";
  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import CobCommitTeaser from "@app/components/CobCommitTeaser.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InfiniteScrollSentinel from "@app/components/InfiniteScrollSentinel.svelte";
  import Topbar from "@app/components/Topbar.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    commits: PaginatedQuery<Commit[]>;
  }

  const { repo, commits }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
  let commitCount: number | undefined = $state();

  $effect(() => {
    const rid = repo.rid;
    commitCount = undefined;
    void cachedRepoCommitCount(rid, project.meta.head)
      .then(count => {
        if (repo.rid === rid) {
          commitCount = count;
        }
      })
      .catch(error => {
        console.error("Failed to load commit count", error);
      });
  });

  type CommitGroup = {
    key: string;
    label: string;
    commits: Commit[];
  };

  let items = $state(commits.content);
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
          head: project.meta.head,
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
    const yesterday = new Date();
    yesterday.setDate(today.getDate() - 1);

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
    const groups = new Map<string, CommitGroup>();

    for (const commit of filteredCommits) {
      const timestamp = commit.committer.time * 1000;
      const key = dayKey(timestamp);
      const current = groups.get(key);

      if (current) {
        current.commits.push(commit);
      } else {
        groups.set(key, {
          key,
          label: dayLabel(timestamp),
          commits: [commit],
        });
      }
    }

    return [...groups.values()];
  });
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .topbar-title {
    font: var(--txt-body-m-semibold);
    color: var(--color-text-secondary);
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .branch-group {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: 1.5rem;
  }
  .branch {
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
  }
  .canonical-badge {
    display: inline-flex;
    align-items: center;
    height: 1.25rem;
    padding: 0 0.375rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-strong);
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
    text-transform: lowercase;
    flex-shrink: 0;
  }
  .content {
    display: flex;
    flex-direction: column;
    flex: 1;
    padding: 1rem;
  }
  .group + .group {
    margin-top: 1.5rem;
  }
  .group-title {
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
    margin-bottom: 0.75rem;
  }
  .commit-list {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    overflow: hidden;
    background: var(--color-surface-canvas);
  }
  .commit-item {
    padding: 0.625rem 1rem;
    cursor: pointer;
  }
  .commit-item + .commit-item {
    border-top: 1px solid var(--color-border-subtle);
  }
  .commit-item:hover {
    background: var(--color-surface-subtle);
  }
</style>

<Layout>
  <div class="page">
    <Topbar>
      <span class="topbar-title">
        Commits
        <span class="branch-group">
          <Icon name="branch" />
          <span class="branch">{project.data.defaultBranch}</span>
          <span class="canonical-badge">canonical</span>
        </span>
        {#if commitCount !== undefined}
          <span class="global-counter-badge">{commitCount}</span>
        {/if}
      </span>
      <div class="global-flex" style:margin-left="auto" style:gap="0.5rem">
        <FuzzySearch
          hasItems={items.length > 0}
          placeholder={`Fuzzy filter commits ${modifierKey()} + f`}
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
      </div>
    </Topbar>
    <div class="content">
      {#each groupedCommits as group (group.key)}
        <section class="group">
          <div class="group-title">{group.label}</div>
          <div class="commit-list">
            {#each group.commits as commit (commit.id)}
              <div
                class="commit-item"
                role="button"
                tabindex="0"
                onclick={() => {
                  void router.push({
                    resource: "repo.commit",
                    rid: repo.rid,
                    commit: commit.id,
                  });
                }}
                onkeydown={e => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    void router.push({
                      resource: "repo.commit",
                      rid: repo.rid,
                      commit: commit.id,
                    });
                  }
                }}>
                <CobCommitTeaser {commit} disabled={false} flush hoverable />
              </div>
            {/each}
          </div>
        </section>
      {/each}

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
      {/if}

      <InfiniteScrollSentinel
        onIntersect={loadMoreContent}
        disabled={!more || loadingMore} />
    </div>
  </div>
</Layout>
