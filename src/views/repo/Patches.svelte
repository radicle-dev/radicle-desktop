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
  import {
    patchCountMismatch,
    resetPatchCounts,
    updatePatchCounts,
  } from "@app/lib/patchCounts.svelte";
  import * as router from "@app/lib/router";
  import type { SidebarData } from "@app/lib/router/definitions";
  import { modifierKey } from "@app/lib/utils";

  import CobCacheWarning from "@app/components/CobCacheWarning.svelte";
  import FuzzySearch from "@app/components/FuzzySearch.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NewPatchButton from "@app/components/NewPatchButton.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    patches: PaginatedQuery<Patch[]>;
    status: PatchStatus | undefined;
    sidebarData: SidebarData;
  }

  const { repo, patches, status, sidebarData }: Props = $props();

  let items = $state(patches.content);
  let cursor = patches.cursor;
  let more = patches.more;

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  let cacheState: CacheEvent | undefined = $state();

  $effect(() => {
    items = patches.content;
    cursor = patches.cursor;
    if (patches.more === true && patches.content.length < DEFAULT_TAKE) {
      more = false;
    } else {
      more = patches.more;
    }
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
      const p = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        skip: 0,
        status,
        take: DEFAULT_TAKE,
      });

      items = p.content;
      cursor = p.cursor;
      more = p.more;

      resetPatchCounts();

      delay(() => {
        cacheState = undefined;
      }, 1500);
    }
  }

  async function loadMoreContent(all: boolean = false) {
    if (more) {
      const p = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        skip: cursor + DEFAULT_TAKE,
        status,
        take: all ? undefined : DEFAULT_TAKE,
      });

      cursor = p.cursor;
      more = p.more;

      if (all) {
        items = p.content;
      } else {
        items = [...items, ...p.content];
      }

      if (p.content.length === 0) {
        more = false;
      }

      if (more === false) {
        updatePatchCounts(items.length, project.meta.patches, status);
      }
    }
  }

  let loadingMore: boolean = $state(false);
  let loading: boolean = $state(false);
  let searchInput = $state("");
  let showSearch = $state(false);

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
    fuzzysort.go(searchInput, searchablePatches, {
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
  .topbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0 1rem;
    height: 2.75rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
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
  .list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-height: 100%;
  }
</style>

<Layout {sidebarData} activeRepo={repo} selfScroll>
  <div class="page">
    <div class="topbar">
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
    </div>

    <ScrollArea
      style="height: 100%; min-width: 0;"
      onScrollHalf={() => {
        if (!loadingMore) {
          loadingMore = true;
          void loadMoreContent().finally(() => (loadingMore = false));
        }
      }}>
      {#if patchCountMismatch(status)}
        <CobCacheWarning
          noun="patches"
          {cacheState}
          onRebuild={rebuildPatchCache} />
      {/if}

      <div class="list">
        {#each searchResults as result}
          <PatchTeaser
            focussed={searchResults.length === 1 && searchInput !== ""}
            patch={result.obj.patch}
            rid={repo.rid}
            {status} />
        {/each}

        {#if searchResults.length === 0}
          <div
            class="global-flex"
            style:flex="1"
            style:justify-content="center"
            style:align-items="center">
            <div
              class="txt-missing txt-body-m-regular global-flex"
              style:gap="0.25rem">
              {#if items.length > 0 && searchResults.length === 0}
                No matching patches
              {:else}
                No {status === undefined ? "" : status} patches
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </ScrollArea>
  </div>
</Layout>
