<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Verdict } from "@bindings/cob/patch/Verdict";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { closeFocused } from "./Popover.svelte";
  import { didFromPublicKey } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import { push } from "@app/lib/router";

  import Border from "@app/components/Border.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import Popover from "@app/components/Popover.svelte";
  import ReviewTeaser from "@app/components/ReviewTeaser.svelte";

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
      <Popover popoverPositionRight="0" popoverPositionTop="2.5rem">
        {#snippet toggle(onclick)}
          <NakedButton
            variant="ghost"
            disabled={hasOwnReview}
            {onclick}
            title={hasOwnReview ? "You already published a review" : undefined}>
            <Icon name="plus" />
            <span class="txt-small">Review</span>
          </NakedButton>
        {/snippet}

        {#snippet popover()}
          <Border variant="ghost" stylePadding="1rem">
            <OutlineButton
              variant="ghost"
              disabled={hasOwnReview}
              title={hasOwnReview
                ? "You already published a review"
                : undefined}
              onclick={async () => {
                const newReview = await createReview();
                if (newReview) {
                  await push({
                    resource: "repo.patch",
                    rid,
                    patch: patchId,
                    status,
                    reviewId: newReview.id,
                  });
                }
                closeFocused();
              }}>
              <span
                class="global-flex"
                style:color="var(--color-foreground-dim)">
                <Icon name="comment" />
                <span class="txt-small">Write Review</span>
              </span>
            </OutlineButton>
            <OutlineButton
              variant="ghost"
              disabled={hasOwnReview}
              title={hasOwnReview
                ? "You already published a review"
                : undefined}
              onclick={async () => {
                await createReview("reject");
                await loadPatch();
                closeFocused();
              }}>
              <span
                class="global-flex"
                style:color="var(--color-foreground-red)">
                <Icon name="comment-cross" />
                <span class="txt-small">Reject</span>
              </span>
            </OutlineButton>
            <OutlineButton
              variant="ghost"
              disabled={hasOwnReview}
              title={hasOwnReview
                ? "You already published a review"
                : undefined}
              onclick={async () => {
                await createReview("accept");
                await loadPatch();
                closeFocused();
              }}>
              <span
                class="global-flex"
                style:color="var(--color-foreground-success)">
                <Icon name="comment-checkmark" />
                <span class="txt-small">Accept</span>
                <span></span>
              </span>
            </OutlineButton>
          </Border>
        {/snippet}
      </Popover>
    </div>
  </div>

  {#if revision.reviews && revision.reviews.length}
    <div style:display={hideReviews ? "none" : "revert"} class="review-list">
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
