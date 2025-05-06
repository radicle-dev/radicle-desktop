<script lang="ts">
  import type { Action } from "@bindings/cob/patch/Action";
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Operation } from "@bindings/cob/Operation";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { PatchStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import capitalize from "lodash/capitalize";
  import fuzzysort from "fuzzysort";

  import * as roles from "@app/lib/roles";
  import * as router from "@app/lib/router";
  import { DEFAULT_TAKE } from "./router";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import {
    formatOid,
    patchStatusBackgroundColor,
    patchStatusColor,
  } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import { modifierKey } from "@app/lib/utils";
  import { nodeRunning } from "@app/lib/events";

  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Command from "@app/components/Command.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import EditableTitle from "@app/components/EditableTitle.svelte";
  import Icon from "@app/components/Icon.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import Layout from "./Layout.svelte";
  import Link from "@app/components/Link.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import NewPatchButton from "@app/components/NewPatchButton.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import PatchStateButton from "@app/components/PatchStateButton.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import PatchTimeline from "@app/components/PatchTimeline.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
  import ReviewComponent from "@app/components/Review.svelte";
  import RevisionBadges from "@app/components/RevisionBadges.svelte";
  import RevisionComponent from "@app/components/Revision.svelte";
  import RevisionSelector from "@app/components/RevisionSelector.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import Tab from "@app/components/Tab.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  interface Props {
    repo: RepoInfo;
    patch: Patch;
    patches: PaginatedQuery<Patch[]>;
    revisions: Revision[];
    config: Config;
    activity: Operation<Action>[];
    status: PatchStatus | undefined;
    review: Review | undefined;
  }

  /* eslint-disable prefer-const */
  let {
    repo,
    patch,
    patches: initialPatches,
    revisions,
    config,
    status: initialStatus,
    activity,
    review,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let cursor: number = $state(0);
  let more: boolean = $state(false);
  let patchTeasers: Patch[] = $state([]);
  let checkoutPopoverExpanded = $state(false);

  let patches = $state(initialPatches);
  let status = $state(initialStatus);
  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);
  let tab: "patch" | "revisions" | "timeline" = $state(
    revisions.length > 1 ? "revisions" : "patch",
  );
  let selectedRevision: Revision = $state(revisions.slice(-1)[0]);

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patch.id;

    tab = revisions.length > 1 ? "revisions" : "patch";
    selectedRevision = revisions.slice(-1)[0];
  });

  $effect(() => {
    patchTeasers = patches.content;
    cursor = patches.cursor;
    more = patches.more;
  });

  const checkoutCommand = $derived.by(() => {
    if (tab === "revisions" && selectedRevision.id !== patch.id) {
      return `rad patch checkout ${formatOid(patch.id)} --revision ${formatOid(selectedRevision.id)}`;
    } else {
      return `rad patch checkout ${formatOid(patch.id)}`;
    }
  });
  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  async function updateTitle(newTitle: string) {
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          id: patch.id,
          type: "edit",
          title: newTitle,
          target: "delegates",
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing title failed: ", error);
    } finally {
      await loadPatch();
    }
  }

  async function saveLabels(labels: string[]) {
    try {
      labelSaveInProgress = true;
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "label",
          labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing labels failed", error);
    } finally {
      labelSaveInProgress = false;
      await loadPatch();
    }
  }

  async function saveAssignees(assignees: Author[]) {
    try {
      assigneesSaveInProgress = true;
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "assign",
          assignees,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing assignees failed", error);
    } finally {
      assigneesSaveInProgress = false;
      await loadPatch();
    }
  }

  async function saveState(newState: Patch["state"]) {
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "lifecycle",
          state: newState,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Changing state failed", error);
    } finally {
      await loadPatch();
    }
  }

  async function loadMoreTeasers(all: boolean = false) {
    if (more) {
      const p = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        status,
        skip: cursor + DEFAULT_TAKE,
        take: all ? undefined : DEFAULT_TAKE,
      });

      cursor = p.cursor;
      more = p.more;
      if (all) {
        patchTeasers = p.content;
      } else {
        patchTeasers = [...patchTeasers, ...p.content];
      }
    }
  }

  async function loadPatch(patchId: string = patch.id) {
    [patch, revisions, activity, patches] = await Promise.all([
      invoke<Patch>("patch_by_id", {
        rid: repo.rid,
        id: patchId,
      }),
      invoke<Revision[]>("revisions_by_patch", {
        rid: repo.rid,
        id: patchId,
      }),
      invoke<Operation<Action>[]>("activity_by_patch", {
        rid: repo.rid,
        id: patchId,
      }),
      invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        status,
        take: DEFAULT_TAKE,
      }),
    ]);
  }

  async function loadReview(reviewId: string | undefined = review?.id) {
    if (!reviewId) {
      return;
    }

    review = await invoke<Review>("review_by_patch_and_revision_and_id", {
      rid: repo.rid,
      id: patch.id,
      revisionId: findReviewRevision(reviewId).id,
      reviewId,
    });
  }

  async function loadPatches(filter: PatchStatus | undefined) {
    try {
      patches = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        status: filter,
        take: DEFAULT_TAKE,
      });
      status = filter;
    } catch (error) {
      console.error("Loading patch list failed", error);
    }
  }

  function findReviewRevision(reviewId: string): Revision {
    // Every review is guaranteed to have a revision according to the protocol
    // model, so using type assertions here is safe.
    return revisions.find(revision => {
      return revision.reviews!.find(rev => {
        return rev.id === reviewId;
      });
    }) as Revision;
  }

  let showFilters: boolean = $state(false);
  let loading: boolean = $state(false);
  let searchInput = $state("");

  const searchablePatches = $derived(
    patches.content
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
  .status {
    padding: 0;
    height: 2.5rem;
    width: 2.5rem;
  }
  .content {
    padding: 1rem 1rem 1rem 0;
  }

  .metadata-divider {
    width: 2px;
    background-color: var(--color-fill-ghost);
    height: calc(100% + 4px);
    top: 0;
    position: relative;
  }
  .metadata-section {
    padding: 0.5rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    height: 100%;
    z-index: 20;
  }
  .metadata-section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>

{#snippet icons(status: PatchStatus | undefined)}
  <div class="icon" style:color={status ? patchStatusColor[status] : undefined}>
    <Icon
      name={status === undefined || status === "open"
        ? "patch"
        : `patch-${status}`} />
  </div>
{/snippet}

{#snippet counters(status: PatchStatus | undefined)}
  <div style:margin-left="auto" style:padding-left="0.25rem">
    {#if status}
      {project.meta.patches[status]}
    {:else}
      {project.meta.patches.draft +
        project.meta.patches.open +
        project.meta.patches.archived +
        project.meta.patches.merged}
    {/if}
  </div>
{/snippet}

<Layout loadMoreSecondColumn={loadMoreTeasers} publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={patch.id} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab="patches" rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <div
      class="txt-medium global-flex"
      style:font-weight="var(--font-weight-medium)"
      style:min-width="450px"
      style:min-height="2.5rem"
      style:margin-bottom="1rem">
      <div
        class="global-flex"
        style:font-weight="var(--font-weight-medium)"
        style:gap="4px"
        style:white-space="nowrap">
        {project.data.name}
        <Icon name="chevron-right" />
        <Link
          underline={false}
          route={{
            resource: "repo.patches",
            rid: repo.rid,
            status: "open",
          }}>
          Patches
        </Link>
      </div>
      <div class="global-flex" style:margin-left="auto">
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
        <NewPatchButton rid={repo.rid} outline />
      </div>
    </div>
    {#if showFilters}
      <div class="global-flex" style:margin="1rem 0">
        <Popover popoverPositionLeft="0" popoverPositionTop="3rem">
          {#snippet toggle(onclick)}
            <OutlineButton variant="ghost" {onclick} styleHeight="2.5rem">
              {@render icons(status)}
              {status ? capitalize(status) : "All"}
              {@render counters(status)}
              <Icon name="chevron-down" />
            </OutlineButton>
          {/snippet}

          {#snippet popover()}
            <Border variant="ghost">
              <DropdownList
                items={[
                  undefined,
                  "draft",
                  "open",
                  "archived",
                  "merged",
                ] as const}>
                {#snippet item(state)}
                  <DropdownListItem
                    styleGap="0.5rem"
                    styleMinHeight="2.5rem"
                    selected={status === state}
                    onclick={async () => {
                      await loadPatches(state);
                      closeFocused();
                    }}>
                    {@render icons(state)}
                    {state ? capitalize(state) : "All"}
                    {@render counters(state)}
                  </DropdownListItem>
                {/snippet}
              </DropdownList>
            </Border>
          {/snippet}
        </Popover>
        {#if patchTeasers.length > 0}
          <TextInput
            onFocus={async () => {
              try {
                loading = true;
                await loadMoreTeasers(true);
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
              showFilters = false;
              searchInput = "";
            }}
            placeholder={`Fuzzy filter patches ${modifierKey()} + f`}
            autofocus
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
      </div>
    {/if}

    {#if searchResults.length > 0}
      <div class="list">
        {#each searchResults as teaser}
          <PatchTeaser
            selected={teaser.obj.patch.id === patch.id}
            focussed={searchResults.length === 1 && searchInput !== ""}
            compact
            loadPatch={async (id: string) => {
              review = undefined;
              await loadPatch(id);
            }}
            patch={teaser.obj.patch}
            rid={repo.rid}
            {status} />
        {/each}
      </div>
    {/if}

    {#if searchResults.length === 0}
      <Border
        variant="ghost"
        styleFlexDirection="column"
        styleOverflow="hidden"
        styleAlignItems="center"
        styleJustifyContent="center">
        <div
          class="global-flex"
          style:height="84px"
          style:justify-content="center">
          <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
            <Icon name="none" />
            {#if patchTeasers.length > 0 && searchResults.length === 0}
              No matching patches.
            {:else}
              No {status === undefined ? "" : status} patches.
            {/if}
          </div>
        </div>
      </Border>
    {/if}
  {/snippet}

  {#if review}
    <ReviewComponent
      {config}
      patchId={patch.id}
      {repo}
      {loadReview}
      {review}
      revision={findReviewRevision(review.id)}
      onNavigateBack={() => {
        review = undefined;
      }} />
  {:else}
    <div class="content">
      <div class="global-flex" style:margin-bottom="1rem" style:gap="0.75rem">
        <div
          class="global-counter status"
          style:color={patchStatusColor[patch.state.status]}
          style:background-color={patchStatusBackgroundColor[
            patch.state.status
          ]}>
          <Icon
            name={patch.state.status === "open"
              ? "patch"
              : `patch-${patch.state.status}`} />
        </div>
        <EditableTitle
          {updateTitle}
          allowedToEdit={true}
          title={patch.title}
          cobId={patch.id} />
        <div class="txt-small" style:margin-left="auto" style:z-index="40">
          <Popover
            bind:expanded={checkoutPopoverExpanded}
            popoverPositionRight="0"
            popoverPositionTop="3rem">
            {#snippet toggle(onclick)}
              <Button styleHeight="2.5rem" variant="secondary" {onclick}>
                <Icon name="checkout" />Checkout<Icon name="chevron-down" />
              </Button>
            {/snippet}
            {#snippet popover()}
              <Border
                styleAlignItems="flex-start"
                styleBackgroundColor="var(--color-background-float)"
                styleFlexDirection="column"
                styleGap="0.5rem"
                stylePadding="1rem"
                styleWidth="max-content"
                variant="ghost">
                To checkout this patch in your working copy, run:
                <Command command={checkoutCommand} styleWidth="100%" />
              </Border>
            {/snippet}
          </Popover>
        </div>
      </div>
      <Border variant="ghost" styleGap="0">
        <div class="metadata-section" style:min-width="8rem">
          <div class="metadata-section-title">Status</div>
          <PatchStateButton
            selectedState={patch.state}
            onSelect={newState => {
              void saveState(newState);
              if (status !== undefined && newState.status !== status) {
                status = undefined;
                void loadPatches(status);
              }
            }} />
        </div>

        <div class="metadata-divider"></div>

        <div class="metadata-section" style:flex="1">
          <LabelInput
            allowedToEdit={!!roles.isDelegateOrAuthor(
              config.publicKey,
              repo.delegates.map(delegate => delegate.did),
              patch.author.did,
            )}
            labels={patch.labels}
            submitInProgress={labelSaveInProgress}
            save={saveLabels} />
        </div>

        <div class="metadata-divider"></div>

        <div class="metadata-section" style:flex="1">
          <AssigneeInput
            allowedToEdit={!!roles.isDelegateOrAuthor(
              config.publicKey,
              repo.delegates.map(delegate => delegate.did),
              patch.author.did,
            )}
            assignees={patch.assignees}
            submitInProgress={assigneesSaveInProgress}
            save={saveAssignees} />
        </div>
      </Border>

      <div class="global-flex" style:gap="0.5rem" style:margin-top="1rem">
        <Border stylePosition="relative" variant="ghost" flatBottom>
          <div
            class="global-flex"
            style:z-index="10"
            style:gap="1rem"
            style:padding="0 1rem"
            style:width="100%">
            <span class="txt-small" style:color="var(--color-foreground-dim)">
              Revisions
            </span>
            <Tab
              active={tab === "patch"}
              onclick={() => {
                tab = "patch";
              }}>
              {formatOid(patch.id)}
              <span
                class="global-counter"
                style:height="1.5rem"
                style:color="var(--color-foreground-contrast)">
                Initial
              </span>
            </Tab>
            {#if revisions.length > 1}
              <Tab
                active={tab === "revisions"}
                onclick={() => {
                  tab = "revisions";
                }}>
                {formatOid(selectedRevision.id)}
                <div class="global-flex" style:gap="0.25rem">
                  <RevisionBadges revision={selectedRevision} {revisions} />
                  <RevisionSelector
                    {patch}
                    {revisions}
                    {selectedRevision}
                    selectRevision={rev => {
                      selectedRevision = rev;
                      tab = "revisions";
                    }} />
                </div>
              </Tab>
            {/if}

            <div style:margin-left="auto">
              <Tab
                active={tab === "timeline"}
                onclick={() => {
                  tab = "timeline";
                }}>
                <Icon name="clock" />
                Timeline
              </Tab>
            </div>
          </div>
        </Border>
      </div>

      <Border
        variant="ghost"
        flatTop
        styleWidth="100%"
        stylePadding="1rem"
        styleMinWidth="0"
        styleDisplay="block"
        styleFlexDirection="column"
        styleAlignItems="flex-start">
        {#if tab === "patch"}
          <RevisionComponent
            rid={repo.rid}
            repoDelegates={repo.delegates}
            patchId={patch.id}
            {loadPatch}
            {status}
            revision={revisions[0]}
            {config} />
        {:else if tab === "timeline"}
          <PatchTimeline {activity} patchId={patch.id} />
        {:else}
          <RevisionComponent
            rid={repo.rid}
            repoDelegates={repo.delegates}
            patchId={patch.id}
            {loadPatch}
            {status}
            revision={selectedRevision}
            {config} />
        {/if}
      </Border>
    </div>
  {/if}
</Layout>
