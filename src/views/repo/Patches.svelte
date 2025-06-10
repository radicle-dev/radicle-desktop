<script lang="ts">
  import type { CacheEvent } from "@bindings/cob/CacheEvent";
  import type { Config } from "@bindings/config/Config";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import delay from "lodash/delay";
  import fuzzysort from "fuzzysort";
  import { Channel } from "@tauri-apps/api/core";

  import * as router from "@app/lib/router";
  import { DEFAULT_TAKE } from "@app/views/repo/router";
  import { explorerUrl, modifierKey } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import {
    patchCountMismatch,
    resetPatchCounts,
    updatePatchCounts,
  } from "@app/lib/patchCounts.svelte";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NewPatchButton from "@app/components/NewPatchButton.svelte";
  import NodeBreadcrumb from "@app/components/NodeBreadcrumb.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import PatchesSecondColumn from "@app/components/PatchesSecondColumn.svelte";
  import Spinner from "@app/components/Spinner.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  import BreadcrumbCopyButton from "./BreadcrumbCopyButton.svelte";
  import Layout from "./Layout.svelte";
  import PatchesBreadcrumb from "./PatchesBreadcrumb.svelte";
  import RepoBreadcrumb from "./RepoBreadcrumb.svelte";

  interface Props {
    repo: RepoInfo;
    patches: PaginatedQuery<Patch[]>;
    config: Config;
    status: PatchStatus | undefined;
    notificationCount: number;
  }

  const { repo, patches, config, status, notificationCount }: Props = $props();

  let items = $state(patches.content);
  let cursor = patches.cursor;
  let more = patches.more;

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  let cacheState: CacheEvent | undefined = $state();

  $effect(() => {
    items = patches.content;
    cursor = patches.cursor;
    // If the first page is not full, we know there are no more patches.
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

      // If the newly fetched patches are empty, there is no more to fetch.
      if (p.content.length === 0) {
        more = false;
      }

      if (more === false) {
        updatePatchCounts(items.length, project.meta.patches, status);
      }
    }
  }

  let loading: boolean = $state(false);
  let searchInput = $state("");

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
  .container {
    padding: 1rem 1rem 1rem 0;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .header {
    font-weight: var(--font-weight-medium);
    font-size: var(--font-size-medium);
    display: flex;
    align-items: center;
    min-height: 2.5rem;
    margin-bottom: 1rem;
    gap: 0.75rem;
  }
</style>

<Layout
  {notificationCount}
  {loadMoreContent}
  hideSidebar
  styleSecondColumnOverflow="visible"
  {config}>
  {#snippet breadcrumbs()}
    <NodeBreadcrumb {config} />
    <Icon name="chevron-right" />
    <RepoBreadcrumb name={project.data.name} rid={repo.rid} />
    <Icon name="chevron-right" />
    <PatchesBreadcrumb rid={repo.rid} {status} />
    <BreadcrumbCopyButton
      url={explorerUrl(`${repo.rid}/patches`)}
      icon="repo"
      id={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <PatchesSecondColumn {project} {status} {repo} />
  {/snippet}

  <div class="container">
    {#if patchCountMismatch(status)}
      <div style="margin-bottom: 1rem;">
        <Border
          styleOverflow="hidden"
          styleBackgroundColor="var(--color-fill-private)"
          stylePadding="0.25rem 0.5rem"
          styleGap="1rem"
          variant="outline">
          <div class="txt-overflow txt-small global-flex">
            <Icon name="warning" />
            <span class="txt-overflow">
              There’s a problem with your COB cache, so some patches may not be
              displayed. You can rebuild the cache to resolve this.
            </span>
          </div>
          <div style:margin-left="auto">
            <Button
              variant="ghost"
              onclick={rebuildPatchCache}
              disabled={cacheState !== undefined}>
              {#if cacheState?.event === "started" || cacheState?.event === "progress"}
                Rebuilding
                <Spinner />
              {:else if cacheState?.event === "finished"}
                Done
                <Icon name="checkmark" />
              {:else}
                Rebuild cache
              {/if}
            </Button>
          </div>
        </Border>
      </div>
    {/if}
    <div class="header">
      Patches

      <div class="global-flex" style:margin-left="auto" style:gap="0.75rem">
        {#if items.length > 0}
          <TextInput
            onFocus={async () => {
              try {
                loading = true;
                // Load all patches.
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
            onDismiss={() => {
              searchInput = "";
            }}
            placeholder={`Fuzzy filter patches ${modifierKey()} + f`}
            keyShortcuts="ctrl+f"
            bind:value={searchInput}>
            {#snippet left()}
              <div
                style:color="var(--color-foreground-dim)"
                style:padding-left="0.5rem">
                <Icon name={loading ? "clock" : "filter"} />
              </div>
            {/snippet}
          </TextInput>
        {/if}
        <NewPatchButton rid={repo.rid} />
      </div>
    </div>

    <div class="list">
      {#each searchResults as result}
        <PatchTeaser
          focussed={searchResults.length === 1 && searchInput !== ""}
          patch={result.obj.patch}
          rid={repo.rid}
          {status} />
      {/each}

      {#if searchResults.length === 0}
        <Border
          variant="ghost"
          styleFlexDirection="column"
          styleAlignItems="center"
          styleJustifyContent="center">
          <div
            class="global-flex"
            style:height="84px"
            style:justify-content="center">
            <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
              <Icon name="none" />
              {#if items.length > 0 && searchResults.length === 0}
                No matching patches.
              {:else}
                No {status === undefined ? "" : status} patches.
              {/if}
            </div>
          </div>
        </Border>
      {/if}
    </div>
  </div>
</Layout>
