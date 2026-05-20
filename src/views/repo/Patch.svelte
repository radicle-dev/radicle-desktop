<script lang="ts">
  import type { PatchStatus } from "./router";
  import type { Operation } from "@bindings/cob/Operation";
  import type { Action } from "@bindings/cob/patch/Action";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Config } from "@bindings/config/Config";
  import type { Stats } from "@bindings/diff/Stats";
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import debounce from "lodash/debounce";

  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { nodeRunning } from "@app/lib/events";
  import {
    cachedGetDiff,
    cachedListCommits,
    invoke,
    writeToClipboard,
  } from "@app/lib/invoke";
  import * as roles from "@app/lib/roles";
  import * as router from "@app/lib/router";
  import {
    absoluteTimestamp,
    didFromPublicKey,
    explorerUrl,
    formatTimestamp,
    patchStatusLabel,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import CheckoutPatchButton from "@app/components/CheckoutPatchButton.svelte";
  import DraftReviewBar from "@app/components/DraftReviewBar.svelte";
  import DropdownList from "@app/components/DropdownList.svelte";
  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import EditableTitle from "@app/components/EditableTitle.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import PatchMetadata from "@app/components/PatchMetadata.svelte";
  import PatchStateButton from "@app/components/PatchStateButton.svelte";
  import Popover, { closeFocused } from "@app/components/Popover.svelte";
  import ReviewPage from "@app/components/ReviewPage.svelte";
  import RevisionComponent from "@app/components/Revision.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    patch: Patch;
    revisions: Revision[];
    config: Config;
    activity: Operation<Action>[];
    status: PatchStatus | undefined;
    review?: Review | DraftReview;
  }

  /* eslint-disable prefer-const */
  let { repo, patch, revisions, config, activity, status, review }: Props =
    $props();
  /* eslint-enable prefer-const */

  const currentReview = $derived.by(() => {
    if (!review) return undefined;
    if ("draft" in review) return review;
    return revisions
      .flatMap(r => r.reviews ?? [])
      .find(r => r.id === review.id);
  });

  let tab: "patch" | "revisions" | "timeline" = $state(
    revisions.length > 1 ? "revisions" : "patch",
  );
  let patchView: "activity" | "changes" = $state("activity");
  let selectedRevisionId: string = $state(revisions.slice(-1)[0].id);
  const selectedRevision: Revision = $derived(
    revisions.find(r => r.id === selectedRevisionId) ?? revisions.slice(-1)[0],
  );
  let selectedRevisionStats: Stats | undefined = $state();
  let revisionPickerExpanded = $state(false);
  let filesExpanded = $state(true);
  let commitCountsByRevisionId: Record<string, number> = $state({});

  const sortedRevisions = $derived(
    [...revisions].sort((a, b) => a.timestamp - b.timestamp),
  );

  $effect(() => {
    const ridLocal = repo.rid;
    const sorted = [...revisions].sort((a, b) => a.timestamp - b.timestamp);
    void Promise.all(
      sorted.map(async (rev): Promise<[string, Commit[]]> => {
        try {
          const commits = await cachedListCommits(
            ridLocal,
            rev.base,
            rev.head,
          );
          return [rev.id, commits];
        } catch {
          return [rev.id, []];
        }
      }),
    ).then(entries => {
      const next: Record<string, number> = {};
      const seen = new Set<string>();
      sorted.forEach((rev, i) => {
        const [, commits] = entries[i];
        const novel = commits.filter(c => !seen.has(c.id));
        novel.forEach(c => seen.add(c.id));
        next[rev.id] = novel.length;
      });
      commitCountsByRevisionId = next;
    });
  });
  const selectedRevisionIndex = $derived(
    sortedRevisions.findIndex(r => r.id === selectedRevisionId),
  );

  function revisionTitle(rev: Revision): string | undefined {
    const body = rev.description.at(-1)?.body?.trim();
    if (!body) return undefined;
    const line = body.split("\n")[0].trim();
    return line.length > 0 ? line : undefined;
  }

  $effect(() => {
    const rev = selectedRevision;
    let cancelled = false;
    void cachedGetDiff(repo.rid, {
      base: rev.base,
      head: rev.head,
      unified: 0,
      highlight: false,
    }).then(diff => {
      if (cancelled) return;
      selectedRevisionStats = diff.stats;
    });
    return () => {
      cancelled = true;
    };
  });

  let lastPatchId = $state(patch.id);
  $effect(() => {
    if (patch.id !== lastPatchId) {
      lastPatchId = patch.id;
      tab = revisions.length > 1 ? "revisions" : "patch";
      selectedRevisionId = revisions.slice(-1)[0].id;
      patchView = "activity";
    }
  });

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

  async function mergePatch() {
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "merge",
          revision: selectedRevision.id,
          commit: selectedRevision.head,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Merging patch failed", error);
    } finally {
      await loadPatch();
    }
  }

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

  async function loadPatch(patchId: string = patch.id) {
    [patch, revisions, activity] = await Promise.all([
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
    ]);
  }

  const ownDraftReviewForPatch = $derived(
    draftReviewStorage.getForPatch(patch.id, {
      did: didFromPublicKey(config.publicKey),
      alias: config.alias,
    }),
  );
  const isRepoDelegate = $derived(
    roles.isDelegate(
      config.publicKey,
      repo.delegates.map(d => d.did),
    ),
  );
  const canShowMerge = $derived(patch.state.status === "open");
  const mergeDisabledReason = $derived.by(() => {
    if (!isRepoDelegate) {
      return "Only delegates can merge patches";
    }
    return undefined;
  });
  const hasOwnPublishedReviewOnSelected = $derived(
    selectedRevision.reviews?.some(
      r => r.author.did === didFromPublicKey(config.publicKey),
    ) ?? false,
  );

  let fileProgress:
    | { filesChecked: number; filesTotal: number }
    | undefined = $state();
  $effect(() => {
    const draft = ownDraftReviewForPatch;
    const rev = selectedRevision;
    if (!draft) {
      fileProgress = undefined;
      return;
    }
    let cancelled = false;
    void cachedGetDiff(repo.rid, {
      base: rev.base,
      head: rev.head,
      unified: 3,
      highlight: false,
    }).then(diff => {
      if (cancelled) return;
      const filePaths = new Set(
        diff.files.map(f =>
          f.status === "moved" || f.status === "copied" ? f.newPath : f.path,
        ),
      );
      const filesChecked = draft.checkedFiles.filter(p =>
        filePaths.has(p),
      ).length;
      fileProgress = {
        filesChecked,
        filesTotal: diff.files.length,
      };
    });
    return () => {
      cancelled = true;
    };
  });

  let copyIcon: "link" | "checkmark" = $state("link");
  const restoreCopyIcon = debounce(() => {
    copyIcon = "link";
  }, 1000);
  async function copyPatchLink() {
    await writeToClipboard(explorerUrl(`${repo.rid}/patches/${patch.id}`));
    copyIcon = "checkmark";
    restoreCopyIcon();
  }
  let deleteMenuExpanded = $state(false);
  let deleting = $state(false);
  async function deletePatch() {
    if (deleting) return;
    deleting = true;
    try {
      await invoke("delete_patch", {
        rid: repo.rid,
        cobId: patch.id,
        opts: { announce: $nodeRunning && $announce },
      });
      void router.push({
        resource: "repo.patches",
        rid: repo.rid,
        status: patch.state.status,
      });
    } catch (error) {
      console.error("Deleting patch failed", error);
    } finally {
      deleting = false;
      deleteMenuExpanded = false;
    }
  }
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .confirm-delete {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.75rem;
    min-width: 16rem;
  }
  .confirm-delete-text {
    color: var(--color-text-primary);
  }
  .confirm-delete-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .confirm-delete-button {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2rem;
    padding: 0 0.75rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-feedback-error-fill);
    color: var(--color-text-on-brand);
    cursor: pointer;
    transition: background-color 0.1s ease;
  }
  .confirm-delete-button:hover:not(:disabled),
  .confirm-delete-button:focus-visible:not(:disabled) {
    background-color: var(--color-feedback-error-fill-hover);
  }
  .confirm-delete-button:active:not(:disabled) {
    background-color: var(--color-feedback-error-fill-active);
  }
  .confirm-delete-button:disabled {
    cursor: default;
    opacity: 0.6;
  }
  .breadcrumb-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .breadcrumb-title {
    color: var(--color-text-primary);
    font: var(--txt-body-m-medium);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .breadcrumb-link:hover {
    color: var(--color-text-primary);
  }
  .main {
    padding: 1.5rem 6rem;
    min-width: 0;
    max-width: 80rem;
    margin: 0 auto;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-areas:
      "title"
      "meta"
      "content";
    column-gap: 2rem;
  }
  .title {
    grid-area: title;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 1.5rem;
    margin-bottom: 1rem;
  }
  .meta-bar {
    grid-area: meta;
    margin-bottom: 0.5rem;
  }
  .content {
    grid-area: content;
    min-width: 0;
  }
  .tabs {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0;
    border-top: 1px solid var(--color-border-subtle);
    border-bottom: 1px solid var(--color-border-subtle);
    margin-bottom: 1rem;
  }
  .tabs-left,
  .tabs-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .tabs-right {
    margin-left: auto;
  }
  .revision-title {
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    max-width: 24rem;
  }
  .revision-date {
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }
  .revision-commits-meta {
    margin-left: auto;
    padding-left: 1rem;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }
</style>

<Layout>
  <div class="page">
    <Topbar>
      <div class="breadcrumb">
        <Icon
          name={patch.state.status === "open"
            ? "patch"
            : `patch-${patch.state.status}`} />
        <button
          class="breadcrumb-link"
          onclick={() =>
            router.push({
              resource: "repo.patches",
              rid: repo.rid,
              status: patch.state.status,
            })}>
          {patchStatusLabel[patch.state.status]}
        </button>
        <Icon name="chevron-right" />
        {#if currentReview}
          <button
            class="breadcrumb-link breadcrumb-title"
            onclick={() =>
              router.push({
                resource: "repo.patch",
                rid: repo.rid,
                patch: patch.id,
                status,
                reviewId: undefined,
              })}>
            {patch.title}
          </button>
          <Icon name="chevron-right" />
          <span style:color="var(--color-text-secondary)">
            Review by {currentReview.author.alias ??
              currentReview.author.did.slice(0, 16)}
          </span>
        {:else}
          <span class="breadcrumb-title">{patch.title}</span>
        {/if}
      </div>
      <div
        class="global-flex"
        style:margin-left="auto"
        style:gap="0.5rem"
        style:z-index="40">
        <Popover
          popoverPadding="0"
          placement="bottom-end"
          bind:expanded={deleteMenuExpanded}>
          {#snippet toggle(onclick)}
            <Button
              variant="naked"
              {onclick}
              active={deleteMenuExpanded}
              title="Delete patch">
              <Icon name="trash" />
              <span class="global-hide-on-medium-desktop-down">Delete</span>
            </Button>
          {/snippet}
          {#snippet popover()}
            <div
              style:border="1px solid var(--color-border-subtle)"
              style:border-radius="var(--border-radius-sm)"
              style:background-color="var(--color-surface-canvas)">
              <div class="confirm-delete">
                <div class="confirm-delete-text txt-body-m-regular">
                  Delete this patch? This can't be undone.
                </div>
                <div class="confirm-delete-actions">
                  <Button
                    variant="outline"
                    disabled={deleting}
                    onclick={() => (deleteMenuExpanded = false)}>
                    Cancel
                  </Button>
                  <button
                    type="button"
                    class="confirm-delete-button txt-body-m-medium"
                    disabled={deleting}
                    onclick={deletePatch}>
                    <Icon name="trash" />
                    {deleting ? "Deleting…" : "Delete"}
                  </button>
                </div>
              </div>
            </div>
          {/snippet}
        </Popover>
        <Button
          variant="naked"
          title="Copy patch link"
          onclick={copyPatchLink}>
          <Icon name={copyIcon} />
          <span class="global-hide-on-medium-desktop-down">Copy link</span>
        </Button>
        {#if !ownDraftReviewForPatch}
          <Button
            variant="secondary"
            disabled={hasOwnPublishedReviewOnSelected}
            onclick={() => {
              draftReviewStorage.create(
                repo.rid,
                patch.id,
                selectedRevision.id,
              );
            }}
            title={hasOwnPublishedReviewOnSelected
              ? "You already created a review for this revision"
              : "Start a review of this revision"}>
            <Icon name="comment" />
            <span class="txt-body-m-regular global-hide-on-medium-desktop-down">
              Review
            </span>
          </Button>
        {/if}
      </div>
    </Topbar>

    <ScrollArea style="flex: 1; min-height: 0;">
      <div>
        <div class="main">
          <div class="title">
            <PatchStateButton
              selectedState={patch.state}
              onSelect={newState => {
                void saveState(newState);
              }}
              onMerge={canShowMerge ? mergePatch : undefined}
              {mergeDisabledReason}
              disabled={!roles.isDelegateOrAuthor(
                config.publicKey,
                repo.delegates.map(d => d.did),
                patch.author.did,
              )} />
            <EditableTitle
              {updateTitle}
              allowedToEdit={roles.isDelegateOrAuthor(
                config.publicKey,
                repo.delegates.map(d => d.did),
                patch.author.did,
              )
                ? true
                : undefined}
              title={patch.title}
              cobId={patch.id} />
          </div>
          <div class="meta-bar">
            <PatchMetadata
              {config}
              {loadPatch}
              {patch}
              {repo}
              {revisions}
              stats={selectedRevisionStats}
              onShowChanges={() => (patchView = "changes")} />
          </div>

          <div class="content">
            <RevisionComponent
              rid={repo.rid}
              {repo}
              repoDelegates={repo.delegates}
              patchId={patch.id}
              {loadPatch}
              revision={revisions[0]}
              {config}
              view="description" />

          {#if currentReview}
            <ReviewPage
              {config}
              {loadPatch}
              {patch}
              repoDelegates={repo.delegates}
              review={currentReview}
              {revisions}
              rid={repo.rid}
              {status} />
          {:else}
            <div class="tabs">
              <div class="tabs-left">
                <Button
                  variant={patchView === "activity" ? "ghost" : "naked"}
                  active={patchView === "activity"}
                  onclick={() => (patchView = "activity")}>
                  <Icon name="activity" />
                  Activity
                </Button>
                <Button
                  variant={patchView === "changes" ? "ghost" : "naked"}
                  active={patchView === "changes"}
                  onclick={() => (patchView = "changes")}>
                  <Icon name="diff" />
                  Changes
                </Button>
              </div>
              {#if patchView === "changes"}
                <div class="tabs-right">
                  <CheckoutPatchButton
                    {tab}
                    selectedRevisionId={selectedRevision.id}
                    patchId={patch.id} />
                  {#if sortedRevisions.length > 1}
                    <Popover
                      popoverPadding="0"
                      placement="bottom-start"
                      bind:expanded={revisionPickerExpanded}>
                      {#snippet toggle(onclick)}
                        <Button
                          variant="outline"
                          {onclick}
                          active={revisionPickerExpanded}>
                          <Icon name="revision" />
                          <span style:color="var(--color-text-secondary)">
                            Revision {selectedRevisionIndex >= 0
                              ? selectedRevisionIndex + 1
                              : "?"} of
                            {sortedRevisions.length}
                          </span>
                          <span class="txt-id">
                            {selectedRevision.id.substring(0, 7)}
                          </span>
                          <Icon
                            name={revisionPickerExpanded
                              ? "chevron-up"
                              : "chevron-down"} />
                        </Button>
                      {/snippet}
                      {#snippet popover()}
                        <div
                          style:border="1px solid var(--color-border-subtle)"
                          style:border-radius="var(--border-radius-sm)"
                          style:background-color="var(--color-surface-canvas)">
                          <DropdownList items={sortedRevisions}>
                            {#snippet item(rev)}
                              {@const title = revisionTitle(rev)}
                              {@const commitCount =
                                commitCountsByRevisionId[rev.id]}
                              <DropdownListItem
                                selected={rev.id === selectedRevision.id}
                                styleGap="0.5rem"
                                onclick={() => {
                                  selectedRevisionId = rev.id;
                                  closeFocused();
                                }}>
                                <Icon name="revision" />
                                <span class="txt-id">
                                  {rev.id.substring(0, 7)}
                                </span>
                                <span
                                  class="revision-date"
                                  title={absoluteTimestamp(rev.timestamp)}>
                                  {formatTimestamp(rev.timestamp)}
                                </span>
                                {#if title}
                                  <span class="revision-title">{title}</span>
                                {/if}
                                {#if commitCount !== undefined}
                                  <span class="revision-commits-meta">
                                    <Icon name="commit" />
                                    {commitCount}
                                  </span>
                                {/if}
                              </DropdownListItem>
                            {/snippet}
                          </DropdownList>
                        </div>
                      {/snippet}
                    </Popover>
                  {/if}
                </div>
              {/if}
            </div>

            <RevisionComponent
              rid={repo.rid}
              {repo}
              repoDelegates={repo.delegates}
              patchId={patch.id}
              {loadPatch}
              revision={selectedRevision}
              {config}
              view={patchView}
              {activity}
              {revisions}
              draftReviewId={ownDraftReviewForPatch?.id}
              onMerge={canShowMerge ? mergePatch : undefined}
              {mergeDisabledReason}
              bind:filesExpanded />
          {/if}
          </div>
        </div>
      </div>
    </ScrollArea>

    {#if ownDraftReviewForPatch}
      <DraftReviewBar
        draftReview={ownDraftReviewForPatch}
        filesChecked={fileProgress?.filesChecked}
        filesTotal={fileProgress?.filesTotal}
        onChange={loadPatch}
        onPublish={async revisionId => {
          await loadPatch();
          const myDid = didFromPublicKey(config.publicKey);
          const updatedRev = revisions.find(r => r.id === revisionId);
          const newReview = updatedRev?.reviews?.find(
            r => r.author.did === myDid,
          );
          if (newReview) {
            void router.push({
              resource: "repo.patch",
              rid: repo.rid,
              patch: patch.id,
              status: undefined,
              reviewId: newReview.id,
            });
          }
        }}
        onCancel={() => {
          draftReviewStorage.delete(ownDraftReviewForPatch.id);
          void loadPatch();
        }} />
    {/if}
  </div>
</Layout>
