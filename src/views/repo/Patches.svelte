<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { PatchStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import fuzzysort from "fuzzysort";

  import * as router from "@app/lib/router";
  import { DEFAULT_TAKE } from "./router";
  import { invoke } from "@app/lib/invoke";
  import { modifierKey } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Layout from "./Layout.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import PatchesSecondColumn from "@app/components/PatchesSecondColumn.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    repo: RepoInfo;
    patches: PaginatedQuery<Patch[]>;
    config: Config;
    status: PatchStatus | undefined;
  }

  const { repo, patches, config, status }: Props = $props();

  let items = $state(patches.content);
  let cursor = patches.cursor;
  let more = patches.more;

  $effect(() => {
    items = patches.content;
    cursor = patches.cursor;
    more = patches.more;
  });

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
    }
  }

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

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
      keys: ["patch.title", "labels", "assignees", "author"],
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
    min-height: 40px;
    margin-bottom: 0.5rem;
  }
</style>

<Layout
  {loadMoreContent}
  hideSidebar
  styleSecondColumnOverflow="visible"
  publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={repo.rid} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab="patches" rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <PatchesSecondColumn {project} {status} {repo} />
  {/snippet}

  <div class="container">
    <div class="header">
      Patches

      {#if items.length > 0}
        <div class="global-flex" style:margin-left="auto">
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
        </div>
      {/if}
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
          styleAlignItems="center"
          styleJustifyContent="center">
          <div
            class="global-flex"
            style:height="74px"
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
