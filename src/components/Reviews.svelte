<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Verdict } from "@bindings/cob/patch/Verdict";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import ReviewTeaser from "@app/components/ReviewTeaser.svelte";

  import { didFromPublicKey } from "@app/lib/utils";

  interface Props {
    rid: string;
    patchId: string;
    revision: Revision;
    config: Config;
    status: PatchStatus | undefined;
    loadPatch: () => Promise<void>;
  }

  const { rid, patchId, revision, config, status, loadPatch }: Props = $props();

  let hideReviews = $state(
    revision.reviews === undefined || revision.reviews.length === 0,
  );

  const hasOwnReview = $derived(
    Boolean(
      revision.reviews &&
        revision.reviews.some(
          value => value.author.did === didFromPublicKey(config.publicKey),
        ),
    ),
  );

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    hideReviews =
      revision.reviews === undefined || revision.reviews.length === 0;
  });

  async function createReview(verdict?: Verdict) {
    try {
      await invoke("edit_patch", {
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
    } finally {
      await loadPatch();
    }
  }
</script>

<style>
  .hide {
    display: none;
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
      <NakedButton
        variant="secondary"
        disabled={hasOwnReview}
        title={hasOwnReview ? "You already published a review" : undefined}
        onclick={() => createReview()}>
        <Icon name="plus" />
        <span class="txt-small">Write Review</span>
      </NakedButton>
      <Button
        variant="danger"
        disabled={hasOwnReview}
        title={hasOwnReview ? "You already published a review" : undefined}
        onclick={() => createReview("reject")}>
        <Icon name="comment-cross" />
        <span class="txt-small">Reject</span>
      </Button>
      <Button
        variant="success"
        disabled={hasOwnReview}
        title={hasOwnReview ? "You already published a review" : undefined}
        onclick={() => createReview("accept")}>
        <Icon name="comment-checkmark" />
        <span class="txt-small">Accept</span>
      </Button>
    </div>
  </div>

  {#if revision.reviews && revision.reviews.length}
    <div class:hide={hideReviews} style:margin-top="1rem">
      {#each revision.reviews as review}
        <ReviewTeaser {rid} {review} {patchId} {status} />
      {/each}
    </div>
  {/if}
</div>
