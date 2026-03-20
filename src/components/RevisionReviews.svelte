<script lang="ts">
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Config } from "@bindings/config/Config";

  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { push } from "@app/lib/router";
  import {
    authorForNodeId,
    didFromPublicKey,
    verdictIcon,
  } from "@app/lib/utils";

  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NodeId from "@app/components/NodeId.svelte";

  interface Props {
    config: Config;
    patch: Patch;
    revision: Revision;
    rid: string;
    status: PatchStatus | undefined;
  }

  const { config, patch, revision, rid, status }: Props = $props();

  const reviews: Array<Review | DraftReview> = $derived(
    [
      draftReviewStorage.getForRevision(revision.id, {
        did: didFromPublicKey(config.publicKey),
        alias: config.alias,
      }),
      ...(revision.reviews ?? []),
    ].filter((review): review is Review | DraftReview => Boolean(review)),
  );

  function unresolvedCommentsCount(review: Review | DraftReview) {
    return review.comments.filter(t => {
      return t.resolved === false && t.location !== null && t.replyTo === null;
    }).length;
  }
</script>

<style>
  .status {
    padding: 0;
  }
  .accepted {
    color: var(--color-feedback-success-text);
  }

  .rejected {
    color: var(--color-feedback-error-text);
  }

  .no-verdict {
    color: var(--color-text-secondary);
  }
</style>

{#each reviews as review}
  <div style:margin="0.5rem 0">
    <DropdownListItem
      selected={false}
      onclick={() => {
        void push({
          resource: "repo.patch",
          rid,
          patch: patch.id,
          status,
          reviewId: review.id,
        });
      }}>
      <div
        class="global-flex"
        style:width="100%"
        style:gap="0.25rem"
        style:padding-left="1.5rem">
        <div
          style:margin-right="0.25rem"
          class:accepted={review.verdict === "accept"}
          class:rejected={review.verdict === "reject"}
          class:no-verdict={review.verdict === undefined}
          class="status">
          <Icon name={verdictIcon(review.verdict)} />
        </div>
        <span class="global-flex" style:gap="0">
          <NodeId {...authorForNodeId(review.author)} />'s
        </span>
        review
        {#if "draft" in review}
          <span
            class="global-chip"
            title="This review is not yet visible to your peers">
            Draft
          </span>
        {/if}
        <div class="global-flex" style:margin-left="auto">
          {#if review.comments.length > 0}
            {@const unresolved = unresolvedCommentsCount(review)}
            {#if unresolved > 0}
              <div class="global-flex" style:gap="0.25rem">
                <Icon name="comment-cross" />
                {unresolved}
              </div>
            {/if}
            {#if unresolved === 0 || review.comments.length > unresolved}
              <div class="global-flex" style:gap="0.25rem">
                <Icon name="comment" />{review.comments.length}
              </div>
            {/if}
          {/if}
        </div>
      </div>
    </DropdownListItem>
  </div>
{/each}
