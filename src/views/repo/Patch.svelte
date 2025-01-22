<script lang="ts">
  import type { Action } from "@bindings/cob/patch/Action";
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Operation } from "@bindings/cob/Operation";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { PatchStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import capitalize from "lodash/capitalize";

  import * as roles from "@app/lib/roles";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import {
    formatOid,
    patchStatusBackgroundColor,
    patchStatusColor,
  } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";

  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Border from "@app/components/Border.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import Layout from "./Layout.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import PatchStateBadge from "@app/components/PatchStateBadge.svelte";
  import PatchStateButton from "@app/components/PatchStateButton.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import PatchTimeline from "@app/components/PatchTimeline.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
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
  }: Props = $props();
  /* eslint-enable prefer-const */

  let cursor: number = $state(0);
  let more: boolean = $state(false);
  let patchTeasers: Patch[] = $state([]);

  let patches = $state(initialPatches);
  let status = $state(initialStatus);
  let editingTitle = $state(false);
  let updatedTitle = $state("");
  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);
  let tab: "patch" | "revisions" | "timeline" = $state("patch");
  let selectedRevision: Revision = $state(revisions.slice(-1)[0]);

  $effect(() => {
    patchTeasers = patches.content;
    cursor = patches.cursor;
    more = patches.more;
  });

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patch.id;

    tab = "patch";
    editingTitle = false;
    updatedTitle = patch.title;
    selectedRevision = revisions.slice(-1)[0];
  });

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  async function loadPatch(rid: string, patchId: string) {
    patch = await invoke<Patch>("patch_by_id", {
      rid: rid,
      id: patchId,
    });
    revisions = await invoke<Revision[]>("revisions_by_patch", {
      rid: rid,
      id: patchId,
    });
    activity = await invoke<Operation<Action>[]>("activity_by_patch", {
      rid: repo.rid,
      id: patch.id,
    });
  }

  async function editTitle(rid: string, patchId: string, title: string) {
    if (patch.title === updatedTitle) {
      editingTitle = false;
      return;
    }

    try {
      await invoke("edit_patch", {
        rid,
        cobId: patchId,
        action: {
          id: patchId,
          type: "edit",
          title,
          target: "delegates",
        },
        opts: { announce: $nodeRunning && $announce },
      });
      editingTitle = false;
    } catch (error) {
      console.error("Editing title failed: ", error);
    } finally {
      await reload();
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
      await reload();
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
          assignees: assignees.map(a => a.did),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing assignees failed", error);
    } finally {
      assigneesSaveInProgress = false;
      await reload();
    }
  }

  async function saveState(state: Patch["state"]) {
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "lifecycle",
          state,
        },
        opts: { announce: $nodeRunning && $announce },
      });
      if (initialStatus !== undefined) {
        status = state["status"];
      }
    } catch (error) {
      console.error("Changing state failed", error);
    } finally {
      await reload();
    }
  }

  async function loadMoreTeasers() {
    if (more) {
      const p = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        skip: cursor + 20,
        take: 20,
      });

      cursor = p.cursor;
      more = p.more;
      patchTeasers = [...patchTeasers, ...p.content];
    }
  }

  async function reload() {
    [config, repo, patches, patch, revisions, activity] = await Promise.all([
      invoke<Config>("config"),
      invoke<RepoInfo>("repo_by_id", {
        rid: repo.rid,
      }),
      invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        status,
      }),
      invoke<Patch>("patch_by_id", {
        rid: repo.rid,
        id: patch.id,
      }),
      invoke<Revision[]>("revisions_by_patch", {
        rid: repo.rid,
        id: patch.id,
      }),
      invoke<Operation<Action>[]>("activity_by_patch", {
        rid: repo.rid,
        id: patch.id,
      }),
    ]);
  }

  async function loadPatches(filter: PatchStatus | undefined) {
    try {
      patches = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        status: filter,
      });
      status = filter;
    } catch (error) {
      console.error("Loading patch list failed", error);
    }
  }
</script>

<style>
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    display: flex;
    align-items: center;
    justify-content: space-between;
    word-break: break-word;
    min-height: 40px;
  }
  .title-icons {
    display: flex;
    gap: 1rem;
    margin-left: 1rem;
    align-items: center;
  }
  .status {
    padding: 0;
    margin-right: 0.75rem;
    height: 2rem;
    width: 2rem;
  }
  .patch-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 1rem;
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
  }
  .metadata-section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
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
    <Sidebar activeTab={{ type: "patches", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <div
      class="txt-regular txt-semibold global-flex"
      style:min-height="40px"
      style:justify-content="space-between">
      <div class="global-flex" style:gap="4px">
        {project.data.name}
        <Icon name="chevron-right" />
        Patches
      </div>

      <Popover popoverPositionRight="0" popoverPositionTop="2.5rem">
        {#snippet toggle(onclick)}
          <OutlineButton variant="ghost" {onclick}>
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
                  style="gap: 0.5rem"
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
    </div>
    <div class="patch-list">
      {#each patchTeasers as teaser}
        <PatchTeaser
          compact
          {loadPatch}
          patch={teaser}
          rid={repo.rid}
          {status}
          selected={patch && teaser.id === patch.id} />
      {/each}

      {#if patches.content.length === 0}
        <Border
          styleMinWidth="25rem"
          variant="ghost"
          styleAlignItems="center"
          styleJustifyContent="center">
          <div
            class="global-flex"
            style:height="74px"
            style:justify-content="center">
            <div class="txt-missing txt-small global-flex" style:gap="0.25rem">
              <Icon name="none" />
              {#if status === undefined}
                No patches.
              {:else}
                No {status} patches.
              {/if}
            </div>
          </div>
        </Border>
      {/if}
    </div>
  {/snippet}

  <div class="content">
    <div style:margin-bottom="0.5rem">
      {#if editingTitle}
        <div class="title">
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

          <TextInput
            valid={updatedTitle.trim().length > 0}
            bind:value={updatedTitle}
            autofocus
            onSubmit={async () => {
              if (updatedTitle.trim().length > 0) {
                await editTitle(repo.rid, patch.id, updatedTitle);
              }
            }}
            onDismiss={() => {
              updatedTitle = patch.title;
              editingTitle = !editingTitle;
            }} />
          <div class="title-icons">
            <Icon
              name="checkmark"
              onclick={async () => {
                if (updatedTitle.trim().length > 0) {
                  await editTitle(repo.rid, patch.id, updatedTitle);
                }
              }} />
            <Icon
              name="cross"
              onclick={() => {
                updatedTitle = patch.title;
                editingTitle = !editingTitle;
              }} />
            <PatchStateButton patchState={patch.state} save={saveState} />
          </div>
        </div>
      {:else}
        <div class="title">
          <div class="global-flex" style:gap="0">
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
            <InlineTitle content={patch.title} fontSize="medium" />
          </div>
          {#if roles.isDelegateOrAuthor( config.publicKey, repo.delegates.map(delegate => delegate.did), patch.author.did, )}
            <div class="title-icons">
              <Icon name="pen" onclick={() => (editingTitle = !editingTitle)} />
              <PatchStateButton patchState={patch.state} save={saveState} />
            </div>
          {/if}
        </div>
      {/if}
    </div>
    <Border variant="ghost" styleGap="0">
      <div class="metadata-section" style:min-width="8rem">
        <div class="metadata-section-title">Status</div>
        <PatchStateBadge state={patch.state} />
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
              style:height="22px"
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
              <RevisionBadges revision={selectedRevision} {revisions} />
            </Tab>

            <RevisionSelector
              {patch}
              {revisions}
              {selectedRevision}
              selectRevision={rev => {
                selectedRevision = rev;
                tab = "revisions";
              }} />
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
          {reload}
          revision={revisions[0]}
          {config} />
      {:else if tab === "timeline"}
        <PatchTimeline {activity} patchId={patch.id} />
      {:else}
        <RevisionComponent
          rid={repo.rid}
          repoDelegates={repo.delegates}
          patchId={patch.id}
          {reload}
          revision={selectedRevision}
          {config} />
      {/if}
    </Border>
  </div>
</Layout>
