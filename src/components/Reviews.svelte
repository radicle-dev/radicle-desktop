<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Verdict } from "@bindings/cob/patch/Verdict";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";

  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import ReviewButton from "@app/components/ReviewButton.svelte";
  import ReviewTeaser from "@app/components/ReviewTeaser.svelte";

  interface Props {
    config: Config;
    loadPatch: () => Promise<void>;
    patchId: string;
    revision: Revision;
    rid: string;
    status: PatchStatus | undefined;
  }

  const { config, loadPatch, patchId, revision, rid, status }: Props = $props();

  let hideReviews = $state(
    revision.reviews === undefined || revision.reviews.length === 0,
  );

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    hideReviews =
      revision.reviews === undefined || revision.reviews.length === 0;
  });

  async function createReview(verdict?: Verdict): Promise<Review | undefined> {
    try {
      return await invoke("edit_patch", {
        rid: rid,
        cobId: patchId,
        action: {
          type: "review",
          revision: revision.id,
          verdict,
          // We need to pass an empty string to create a review without a verdict.
          summary: "",
          labels: [],
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Creating a review failed: ", error);
    }
  }
</script>

<style>
  .review-list {
    margin-top: 1rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>

<div style:margin={hideReviews ? "1.5rem 0" : "1.5rem 0 2.5rem 0"}>
  <div class="global-flex">
    <NakedButton
      disabled={revision.reviews === undefined || revision.reviews.length === 0}
      variant="ghost"
      onclick={() => (hideReviews = !hideReviews)}>
      <Icon name={hideReviews ? "chevron-right" : "chevron-down"} />
      <div class="txt-semibold global-flex txt-regular">
        Reviews <span style:font-weight="var(--font-weight-regular)">
          {revision.reviews?.length ?? 0}
        </span>
      </div>
    </NakedButton>

    <div class="global-flex" style:margin-left="auto">
      <ReviewButton
        {rid}
        {patchId}
        {revision}
        {config}
        {status}
        {loadPatch}
        {createReview} />
    </div>
  </div>

  {#if revision.reviews && revision.reviews.length}
    <div style:display={hideReviews ? "none" : "flex"} class="review-list">
      {#each revision.reviews as review, idx}
        <ReviewTeaser
          {rid}
          {review}
          {patchId}
          {status}
          first={idx === 0}
          last={idx === revision.reviews.length - 1} />
      {/each}
    </div>
  {/if}
</div>
