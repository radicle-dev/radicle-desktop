<script lang="ts">
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Patch } from "@bindings/cob/patch/Patch";

  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { cachedDiffStats } from "@app/lib/invoke";
  import { push } from "@app/lib/router";
  import {
    absoluteTimestamp,
    authorForNodeId,
    formatTimestamp,
    patchStatusBackgroundColor,
    patchStatusColor,
    pluralize,
    publicKeyFromDid,
  } from "@app/lib/utils";

  import DiffStatBadge from "@app/components/DiffStatBadge.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import Label from "@app/components/Label.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";

  interface Props {
    focussed?: boolean;
    patch: Patch;
    rid: string;
    status: PatchStatus | undefined;
  }

  const { focussed, patch, rid, status }: Props = $props();

  const hasDraftReview = $derived(draftReviewStorage.hasForPatch(patch.id));
</script>

<style>
  .patch-teaser {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    min-height: 5rem;
    background-color: var(--color-surface-canvas);
    padding: 1rem;
    cursor: pointer;
    font: var(--txt-body-l-regular);
    word-break: break-word;
    width: 100%;
  }
  .patch-teaser:hover {
    background-color: var(--color-surface-subtle);
  }
  .status {
    padding: 0;
    margin-right: 1rem;
  }
  .patch-teaser:first-of-type {
    border-radius: var(--border-radius-sm) var(--border-radius-sm) 0 0;
  }
  .patch-teaser:last-of-type {
    border-radius: 0 0 var(--border-radius-sm) var(--border-radius-sm);
  }
  .patch-teaser:only-of-type {
    border-radius: var(--border-radius-sm);
  }
  .reviews {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    height: 1.5rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }
  .reviewer-stack {
    display: inline-flex;
    align-items: center;
  }
  .reviewer-stack :global(img) {
    outline: 1px solid var(--color-surface-canvas);
    margin-left: -0.25rem;
  }
  .reviewer-stack :global(img:first-child) {
    margin-left: 0;
  }
  .reviewer-overflow {
    margin-left: 0.25rem;
    color: var(--color-text-tertiary);
  }
  .verdict-accept {
    color: var(--color-feedback-success-text);
  }
  .verdict-reject {
    color: var(--color-feedback-error-text);
  }
</style>

{#snippet patchSnippet()}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    tabindex="0"
    role="button"
    class="patch-teaser"
    style:align-items="flex-start"
    style:clip-path={focussed ? "none" : undefined}
    style:padding={focussed ? "1rem" : "1.25rem"}
    onclick={() => {
      void push({
        resource: "repo.patch",
        rid,
        patch: patch.id,
        status,
        reviewId: undefined,
      });
    }}>
    <div class="global-flex" style:align-items="flex-start">
      <div
        class="global-chip status"
        style:color={patchStatusColor[patch.state.status]}
        style:background-color={patchStatusBackgroundColor[patch.state.status]}>
        <Icon
          name={patch.state.status === "open"
            ? "patch"
            : `patch-${patch.state.status}`} />
      </div>
      <div
        class="global-flex"
        style:flex-direction="column"
        style:align-items="flex-start">
        <InlineTitle content={patch.title} />
        <div class="global-flex txt-body-m-regular" style:flex-wrap="wrap">
          <NodeId {...authorForNodeId(patch.author)} />
          opened
          <Id id={patch.id} clipboard={patch.id} label="patch ID" />
          <span title={absoluteTimestamp(patch.timestamp)}>
            {formatTimestamp(patch.timestamp)}
          </span>
        </div>
      </div>
    </div>

    <div class="global-flex" style:margin-left="auto">
      {#if hasDraftReview}
        <div
          class="txt-body-m-regular"
          style:white-space="nowrap"
          style:border="1px solid var(--color-border-subtle)"
          style:border-radius="var(--border-radius-sm)"
          style:padding="0.125rem 0.5rem"
          style:color="var(--color-text-primary)">
          Review in progress
        </div>
      {/if}

      {#await cachedDiffStats(rid, patch.base, patch.head) then stats}
        <DiffStatBadge {stats} />
      {/await}

      {#each patch.labels as label}
        <Label {label} />
      {/each}
      {#if patch.commentCount > 0}
        <div
          class="txt-body-m-regular global-flex"
          style:gap="0.25rem"
          style:border="1px solid var(--color-border-subtle)"
          style:border-radius="var(--border-radius-sm)"
          style:height="1.5rem"
          style:padding="0 0.5rem"
          style:color="var(--color-text-tertiary)">
          <Icon name="comment" />
          {patch.commentCount}
        </div>
      {/if}
      {#if patch.reviewers.length > 0}
        {@const verdicts = patch.reviewers.map(r => r.verdict)}
        {@const hasReject = verdicts.includes("reject")}
        {@const allAccept =
          verdicts.length > 0 && verdicts.every(v => v === "accept")}
        <div
          class="reviews txt-body-m-regular"
          title="{patch.reviewers.length} {pluralize(
            'reviewer',
            patch.reviewers.length,
          )}">
          <span
            class:verdict-accept={allAccept}
            class:verdict-reject={hasReject}>
            <Icon
              name={hasReject ? "stop" : allAccept ? "thumbs-up" : "comment"} />
          </span>
          <span>{patch.reviewers.length}</span>
          <span class="reviewer-stack">
            {#each patch.reviewers.slice(0, 3) as reviewer (reviewer.author.did)}
              <UserAvatar
                nodeId={publicKeyFromDid(reviewer.author.did)}
                styleWidth="1rem" />
            {/each}
            {#if patch.reviewers.length > 3}
              <span class="reviewer-overflow">
                +{patch.reviewers.length - 3}
              </span>
            {/if}
          </span>
        </div>
      {/if}
      <div
        class="txt-body-m-regular global-flex"
        style:gap="0.25rem"
        style:white-space="nowrap"
        style:border="1px solid var(--color-border-subtle)"
        style:border-radius="var(--border-radius-sm)"
        style:height="1.5rem"
        style:padding="0 0.5rem"
        style:color="var(--color-text-tertiary)">
        <Icon name="revision" />
        {patch.revisionCount}
      </div>
    </div>
  </div>
{/snippet}

{#if focussed}
  <div
    style:border="1px solid var(--color-border-brand)"
    style:border-radius="var(--border-radius-sm)"
    style:display="flex"
    style:gap="0.5rem"
    style:align-items="center"
    style:background-color="var(--color-surface-canvas)">
    {@render patchSnippet()}
  </div>
{:else}
  {@render patchSnippet()}
{/if}
