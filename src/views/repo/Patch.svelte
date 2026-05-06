<script lang="ts">
  import type { PatchStatus } from "./router";
  import type { Operation } from "@bindings/cob/Operation";
  import type { Action } from "@bindings/cob/patch/Action";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { nodeRunning } from "@app/lib/events";
  import {
    cachedGetDiff,
    cachedListCommits,
    invoke,
  } from "@app/lib/invoke";
  import * as router from "@app/lib/router";
  import {
    didFromPublicKey,
    explorerUrl,
    patchStatusBackgroundColor,
    patchStatusColor,
    patchStatusLabel,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import CheckoutPatchButton from "@app/components/CheckoutPatchButton.svelte";
  import DraftReviewBar from "@app/components/DraftReviewBar.svelte";
  import EditableTitle from "@app/components/EditableTitle.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NewPatchButton from "@app/components/NewPatchButton.svelte";
  import PatchMetadata from "@app/components/PatchMetadata.svelte";
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
  }

  /* eslint-disable prefer-const */
  let { repo, patch, revisions, config, activity }: Props = $props();
  /* eslint-enable prefer-const */

  let tab: "patch" | "revisions" | "timeline" = $state(
    revisions.length > 1 ? "revisions" : "patch",
  );
  let patchView: "activity" | "changes" = $state("activity");
  let selectedRevision: Revision = $state(revisions.slice(-1)[0]);

  let lastPatchId = $state(patch.id);
  $effect(() => {
    if (patch.id !== lastPatchId) {
      lastPatchId = patch.id;
      tab = revisions.length > 1 ? "revisions" : "patch";
      selectedRevision = revisions.slice(-1)[0];
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
  const hasOwnPublishedReviewOnSelected = $derived(
    selectedRevision.reviews?.some(
      r => r.author.did === didFromPublicKey(config.publicKey),
    ) ?? false,
  );

  let reviewProgress: { checked: number; total: number } | undefined =
    $state();
  $effect(() => {
    const draft = ownDraftReviewForPatch;
    const rev = selectedRevision;
    if (!draft) {
      reviewProgress = undefined;
      return;
    }
    let cancelled = false;
    void Promise.all([
      cachedListCommits(repo.rid, rev.base, rev.head),
      cachedGetDiff(repo.rid, {
        base: rev.base,
        head: rev.head,
        unified: 3,
        highlight: false,
      }),
    ]).then(([commits, diff]) => {
      if (cancelled) return;
      const commitIds = new Set(commits.map(c => c.id));
      const filePaths = new Set(
        diff.files.map(f =>
          f.status === "moved" || f.status === "copied" ? f.newPath : f.path,
        ),
      );
      const checkedCommits = draft.checkedCommits.filter(id =>
        commitIds.has(id),
      ).length;
      const checkedFiles = draft.checkedFiles.filter(p =>
        filePaths.has(p),
      ).length;
      reviewProgress = {
        checked: checkedCommits + checkedFiles,
        total: commits.length + diff.files.length,
      };
    });
    return () => {
      cancelled = true;
    };
  });
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
  .breadcrumb-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .breadcrumb-link:hover {
    color: var(--color-text-primary);
  }
  .review-progress {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    margin-left: 0.75rem;
    padding: 0.125rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
  }
  .content {
    display: grid;
    grid-template-columns: 1fr 22rem;
  }
  @media (max-width: 1349.98px) {
    .content {
      grid-template-columns: 1fr;
    }
  }
  .main {
    padding: 1.5rem 2rem;
    min-width: 0;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--color-border-subtle);
    height: 100%;
    padding: 1.5rem 1rem;
    gap: 0.5rem;
  }
  @media (max-width: 1349.98px) {
    .sidebar {
      display: none;
    }
    .sidebar-inline {
      display: block;
    }
  }
  @media (min-width: 1350px) {
    .sidebar-inline {
      display: none;
    }
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
          <Id id={patch.id} clipboard={patch.id} placement="bottom-start" />
          <ExternalLink
            href={explorerUrl(`${repo.rid}/patches/${patch.id}`)}
            title="Open in radicle.network" />
        </div>
        {#if reviewProgress}
          <div class="review-progress" title="Items reviewed in this revision">
            <Icon name="comment" />
            {reviewProgress.checked}/{reviewProgress.total} reviewed
          </div>
        {/if}
        <div style:margin-left="auto">
          <NewPatchButton rid={repo.rid} ghost />
        </div>
      </Topbar>

      <ScrollArea style="flex: 1; min-height: 0;">
        <div class="content">
          <div class="main">
            <div class="title">
              <div
                class="global-chip"
                style:color={patchStatusColor[patch.state.status]}
                style:background-color={patchStatusBackgroundColor[
                  patch.state.status
                ]}
                style:height="2rem"
                style:width="2rem"
                style:padding="0">
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
              <div
                class="global-flex"
                style:margin-left="auto"
                style:z-index="40"
                style:gap="1rem">
                <CheckoutPatchButton
                  {tab}
                  selectedRevisionId={selectedRevision.id}
                  patchId={patch.id} />
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
                    <span
                      class="txt-body-m-regular global-hide-on-medium-desktop-down">
                      Review revision
                    </span>
                  </Button>
                {/if}
              </div>
            </div>
            <div class="sidebar-inline">
              <PatchMetadata
                {config}
                {loadPatch}
                {patch}
                {repo}
                {saveState}
                horizontal />
            </div>

            <RevisionComponent
              rid={repo.rid}
              repoDelegates={repo.delegates}
              patchId={patch.id}
              {loadPatch}
              revision={revisions[0]}
              {config}
              view="description" />

            <div class="tabs">
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

            <RevisionComponent
              rid={repo.rid}
              repoDelegates={repo.delegates}
              patchId={patch.id}
              {loadPatch}
              revision={selectedRevision}
              {config}
              view={patchView}
              {activity}
              {revisions}
              onSelectRevision={rev => (selectedRevision = rev)}
              draftReviewId={ownDraftReviewForPatch?.id} />
          </div>

          <div class="sidebar">
            <PatchMetadata {config} {loadPatch} {patch} {repo} {saveState} />
          </div>
        </div>
      </ScrollArea>

      {#if ownDraftReviewForPatch}
        <DraftReviewBar
          draftReview={ownDraftReviewForPatch}
          onChange={loadPatch}
          onPublish={async () => {
            await loadPatch();
          }}
          onCancel={() => {
            draftReviewStorage.delete(ownDraftReviewForPatch.id);
            void loadPatch();
          }} />
      {/if}
    </div>
</Layout>
