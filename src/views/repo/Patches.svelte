<script lang="ts">
  import type { PatchStatus } from "@app/views/repo/router";
  import type { CacheEvent } from "@bindings/cob/CacheEvent";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { DEFAULT_TAKE } from "@app/views/repo/router";
  import { Channel } from "@tauri-apps/api/core";
  import fuzzysort from "fuzzysort";
  import delay from "lodash/delay";

  import { invoke } from "@app/lib/invoke";
  import * as mutexExecutor from "@app/lib/mutexExecutor";
  import {
    patchCountMismatch,
    resetPatchCounts,
    updatePatchCounts,
  } from "@app/lib/patchCounts.svelte";
  import * as router from "@app/lib/router";
  import { modifierKey } from "@app/lib/utils";

  import CobCacheWarning from "@app/components/CobCacheWarning.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NewPatchButton from "@app/components/NewPatchButton.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";
  import VirtualList from "@app/components/VirtualList.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    patches: PaginatedQuery<Patch[]>;
    status: PatchStatus | undefined;
  }

  const { repo, patches, status }: Props = $props();

  // Parent reuses this component across status filter changes; a sibling
  // $effect resets pagination state when the patches prop changes.
  // svelte-ignore state_referenced_locally
  let items = $state(patches.content);
  // svelte-ignore state_referenced_locally
  let more = $state(patches.more);
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
    items = patches.content;
    more = patches.more;
    // Abort any in-flight loadMoreContent so it cannot append a page
    // from the previous filter onto the just-reset items.
    void loader.run(abort);
  });

  $effect(() => {
    if (more === false) {
      updatePatchCounts(items.length, project.meta.patches, status);
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

  async function rebuildPatchCache() {
    try {
      await invoke("rebuild_patch_cache", {
        rid: repo.rid,
        onEvent: new Channel<CacheEvent>(message => {
          cacheState = message;
        }),
      });
    } catch (error) {
      console.error(error);
    } finally {
      const page = await loader.run(async () => {
        return await invoke<PaginatedQuery<Patch[]>>("list_patches", {
          rid: repo.rid,
          skip: 0,
          status,
          take: DEFAULT_TAKE,
        });
      });

      if (page !== undefined) {
        items = page.content;
        more = page.more;
      }

      resetPatchCounts();

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
        return await invoke<PaginatedQuery<Patch[]>>("list_patches", {
          rid: repo.rid,
          status,
          skip: all ? 0 : items.length,
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
      items = all ? page.content : [...items, ...page.content];
      if (page.content.length === 0) more = false;
    } finally {
      if (!superseded) {
        loadingMore = false;
      }
    }
  }

  const searchablePatches = $derived(
    items
      .flatMap(i => {
        return {
          patch: i,
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
    fuzzysort.go(debouncedSearch, searchablePatches, {
      keys: ["patch.title", "labels", "assignees", "author", "patch.id"],
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
  .row {
    border-bottom: 1px solid var(--color-border-subtle);
  }
</style>

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <span class="topbar-title">Patches</span>
      <div class="filters">
        <a
          class="filter"
          class:active={status === undefined}
          href={router.routeToPath({
            resource: "repo.patches",
            rid: repo.rid,
            status: undefined,
          })}>
          <Icon name="patch" />
          <span class="filter-label">All</span>
          <span class="global-counter-badge">
            {project.meta.patches.open +
              project.meta.patches.draft +
              project.meta.patches.archived +
              project.meta.patches.merged}
          </span>
        </a>
        <a
          class="filter"
          class:active={status === "open"}
          href={router.routeToPath({
            resource: "repo.patches",
            rid: repo.rid,
            status: "open",
          })}>
          <Icon name="patch" />
          <span class="filter-label">Open</span>
          <span class="global-counter-badge">{project.meta.patches.open}</span>
        </a>
        <a
          class="filter"
          class:active={status === "merged"}
          href={router.routeToPath({
            resource: "repo.patches",
            rid: repo.rid,
            status: "merged",
          })}>
          <Icon name="patch-merged" />
          <span class="filter-label">Merged</span>
          <span class="global-counter-badge">
            {project.meta.patches.merged}
          </span>
        </a>
        <a
          class="filter"
          class:active={status === "archived"}
          href={router.routeToPath({
            resource: "repo.patches",
            rid: repo.rid,
            status: "archived",
          })}>
          <Icon name="patch-archived" />
          <span class="filter-label">Archived</span>
          <span class="global-counter-badge">
            {project.meta.patches.archived}
          </span>
        </a>
        <a
          class="filter"
          class:active={status === "draft"}
          href={router.routeToPath({
            resource: "repo.patches",
            rid: repo.rid,
            status: "draft",
          })}>
          <Icon name="patch-draft" />
          <span class="filter-label">Drafts</span>
          <span class="global-counter-badge">{project.meta.patches.draft}</span>
        </a>
      </div>
      <div class="global-flex" style:margin-left="auto" style:gap="0.5rem">
        <FuzzySearch
          hasItems={items.length > 0}
          placeholder={`Fuzzy filter patches ${modifierKey()} + f`}
          icon={loading ? "clock" : "filter"}
          onFocus={async () => {
            try {
              loading = true;
              await loadMoreContent(true);
            } catch (e) {
              console.error("Loading all patches failed: ", e);
            } finally {
              loading = false;
            }
          }}
          onSubmit={async () => {
            if (searchResults.length === 1) {
              await router.push({
                patch: searchResults[0].obj.patch.id,
                resource: "repo.patch",
                reviewId: undefined,
                rid: repo.rid,
                status,
              });
            }
          }}
          bind:show={showSearch}
          bind:value={searchInput} />
        <NewPatchButton rid={repo.rid} />
      </div>
    </Topbar>

    <ScrollArea style="height: 100%; min-width: 0;">
      <div bind:clientHeight={chromeHeight}>
        {#if patchCountMismatch(status)}
          <CobCacheWarning
            noun="patches"
            {cacheState}
            onRebuild={rebuildPatchCache} />
        {/if}
      </div>

      {#if searchResults.length === 0}
        <div
          class="global-flex"
          style:flex="1"
          style:justify-content="center"
          style:align-items="center">
          <div
            class="txt-missing txt-body-m-regular global-flex"
            style:gap="0.25rem">
            {#if items.length > 0}
              No matching patches
            {:else}
              No {status === undefined ? "" : status} patches
            {/if}
          </div>
        </div>
      {:else}
        <VirtualList
          items={searchResults}
          hasMore={more}
          {loadingMore}
          onLoadMore={() => loadMoreContent()}
          startMargin={chromeHeight}
          estimatedItemSize={80}
          getKey={result => result.obj.patch.id}>
          {#snippet row(result)}
            <div class="row">
              <PatchTeaser
                focussed={searchResults.length === 1 && searchInput !== ""}
                patch={result.obj.patch}
                rid={repo.rid}
                {status} />
            </div>
          {/snippet}
        </VirtualList>
      {/if}
    </ScrollArea>
  </div>
</Layout>
