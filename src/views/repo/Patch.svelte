<script lang="ts">
  import type { PatchStatus } from "./router";
  import type { Operation } from "@bindings/cob/Operation";
  import type { Action } from "@bindings/cob/patch/Action";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import * as router from "@app/lib/router";
  import {
    didFromPublicKey,
    explorerUrl,
    patchStatusBackgroundColor,
    patchStatusColor,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Button from "@app/components/Button.svelte";
  import CheckoutPatchButton from "@app/components/CheckoutPatchButton.svelte";
  import EditableTitle from "@app/components/EditableTitle.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NewPatchButton from "@app/components/NewPatchButton.svelte";
  import PatchMetadata from "@app/components/PatchMetadata.svelte";
  import PatchTimeline from "@app/components/PatchTimeline.svelte";
  import ReviewComponent from "@app/components/Review.svelte";
  import RevisionComponent from "@app/components/Revision.svelte";
  import Revisions from "@app/components/Revisions.svelte";
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
    review: Review | DraftReview | undefined;
  }

  /* eslint-disable prefer-const */
  let {
    repo,
    patch,
    revisions,
    config,
    status: initialStatus,
    activity,
    review,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let hideTimeline = $state(true);
  const status = initialStatus;
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

  async function loadReview() {
    if (!review) {
      return;
    }

    if ("draft" in review) {
      review = draftReviewStorage.get(review.id, review.author);
    } else {
      review = await invoke<Review>("review_by_patch_and_revision_and_id", {
        rid: repo.rid,
        id: patch.id,
        revisionId: findReviewRevision(review).id,
        reviewId: review.id,
      });
    }
  }

  function findReviewRevision(review: Review | DraftReview): Revision {
    // Every review is guaranteed to have a revision according to the protocol
    // model, so using type assertions here is safe.
    if ("draft" in review) {
      return revisions.find(
        revision => revision.id === review.revisionId,
      ) as Revision;
    } else {
      return revisions.find(revision =>
        revision.reviews?.find(rev => rev.id === review.id),
      ) as Revision;
    }
  }

  const reviewsOfSelectedRevision: Array<Review | DraftReview> = $derived(
    [
      draftReviewStorage.getForRevision(selectedRevision.id, {
        did: didFromPublicKey(config.publicKey),
        alias: config.alias,
      }),
      ...(selectedRevision.reviews ?? []),
    ].filter((review): review is Review | DraftReview => Boolean(review)),
  );
  const ownDraftReview = $derived(
    reviewsOfSelectedRevision.find(
      value =>
        value.author.did === didFromPublicKey(config.publicKey) &&
        "draft" in value,
    ),
  );
  const hasOwnPublishedReview = $derived(
    reviewsOfSelectedRevision.some(
      value =>
        value.author.did === didFromPublicKey(config.publicKey) &&
        !("draft" in value),
    ),
  );
  const patchDescription = $derived(
    revisions[0]?.description.slice(-1)[0]?.body ?? "",
  );
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
  .patch-description {
    background-color: var(--color-surface-canvas);
    border-radius: var(--border-radius-sm);
    font: var(--txt-body-m-regular);
    margin-bottom: 1rem;
    padding: 0.75rem;
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
  {#if review}
    <ReviewComponent
      {config}
      patchId={patch.id}
      {repo}
      {loadReview}
      {review}
      revision={findReviewRevision(review)}
      onNavigateBack={() => {
        review = undefined;
      }} />
  {:else}
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
                status: undefined,
              })}>
            All Patches
          </button>
          <Icon name="chevron-right" />
          <Id id={patch.id} clipboard={patch.id} placement="bottom-start" />
          <ExternalLink
            href={explorerUrl(`${repo.rid}/patches/${patch.id}`)}
            title="Open in radicle.network" />
        </div>
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
                <Button
                  variant="secondary"
                  disabled={hasOwnPublishedReview}
                  onclick={() => {
                    const id =
                      ownDraftReview?.id ??
                      draftReviewStorage.create(repo.rid, selectedRevision.id);
                    void router.push({
                      resource: "repo.patch",
                      rid: repo.rid,
                      patch: patch.id,
                      reviewId: id,
                      status,
                    });
                  }}
                  title={hasOwnPublishedReview
                    ? "You already created a review for this revision"
                    : ownDraftReview
                      ? "Continue review"
                      : "Review revision"}>
                  <Icon name="comment" />
                  <span
                    class="txt-body-m-regular global-hide-on-medium-desktop-down">
                    {ownDraftReview ? "Continue review" : "Review revision"}
                  </span>
                </Button>
              </div>
            </div>
            <div class="patch-description">
              {#if patchDescription.trim()}
                <Markdown rid={repo.rid} breaks content={patchDescription} />
              {:else}
                <span class="txt-missing txt-body-m-regular">
                  No description.
                </span>
              {/if}
            </div>

            <div class="sidebar-inline">
              <PatchMetadata
                {config}
                {loadPatch}
                {patch}
                {repo}
                {saveState}
                horizontal />
              <div style:padding="0.5rem" style:margin-bottom="1rem">
                <div
                  class="txt-body-m-regular"
                  style:margin-bottom="1rem"
                  style:color="var(--color-text-secondary)">
                  Revisions
                </div>
                <Revisions
                  {config}
                  rid={repo.rid}
                  selectRevision={rev => {
                    selectedRevision = rev;
                    tab = "revisions";
                  }}
                  {patch}
                  {revisions}
                  {selectedRevision}
                  {status} />
              </div>
            </div>

            <RevisionComponent
              rid={repo.rid}
              repoDelegates={repo.delegates}
              patchId={patch.id}
              {loadPatch}
              revision={selectedRevision}
              hideDescription={selectedRevision.id === revisions[0]?.id}
              {config} />

            <div class="global-flex" style:margin-top="1.5rem">
              <Button
                variant="naked"
                onclick={() => (hideTimeline = !hideTimeline)}>
                <Icon name={hideTimeline ? "chevron-right" : "chevron-down"} />
              </Button>
              <div class="txt-body-m-regular global-flex">Timeline</div>
            </div>
            <div
              style:display={hideTimeline ? "none" : "revert"}
              style:margin-top="1rem">
              <PatchTimeline {activity} patchId={patch.id} />
            </div>
          </div>

          <div class="sidebar">
            <PatchMetadata {config} {loadPatch} {patch} {repo} {saveState} />
            <div style:padding="0.5rem" style:margin-top="0.5rem">
              <div
                class="txt-body-m-regular"
                style:margin-bottom="1rem"
                style:color="var(--color-text-secondary)">
                Revisions
              </div>
              <Revisions
                {config}
                rid={repo.rid}
                selectRevision={rev => {
                  selectedRevision = rev;
                  tab = "revisions";
                }}
                {patch}
                {revisions}
                {selectedRevision}
                {status} />
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  {/if}
</Layout>
