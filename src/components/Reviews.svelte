<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { DraftReview } from "@app/lib/draftReviewStorage";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import ReviewTeaser from "@app/components/ReviewTeaser.svelte";
  import { didFromPublicKey } from "@app/lib/utils";
  import { draftReviewStorage } from "@app/lib/draftReviewStorage";
  import { push } from "@app/lib/router";

  interface Props {
    config: Config;
    patchId: string;
    revision: Revision;
    rid: string;
    status: PatchStatus | undefined;
  }

  const { config, patchId, revision, rid, status }: Props = $props();

  let hideReviews = $derived.by(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patchId;

    return reviews.length === 0;
  });

  const reviews: Array<Review | DraftReview> = $derived(
    [
      draftReviewStorage.getForRevision(revision.id, {
        did: didFromPublicKey(config.publicKey),
        alias: config.alias,
      }),
      ...(revision.reviews ?? []),
    ].filter((review): review is Review | DraftReview => Boolean(review)),
  );

  const hasOwnReview = $derived(
    reviews.some(
      value => value.author.did === didFromPublicKey(config.publicKey),
    ),
  );
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
    <div class="global-flex">
      <NakedButton
        stylePadding="0 4px"
        disabled={reviews.length === 0}
        variant="ghost"
        onclick={() => (hideReviews = !hideReviews)}>
        <Icon name={hideReviews ? "chevron-right" : "chevron-down"} />
      </NakedButton>
      <div
        class="txt-semibold global-flex txt-regular"
        style:color={reviews.length === 0
          ? "var(--color-foreground-disabled)"
          : undefined}>
        Reviews <span style:font-weight="var(--font-weight-regular)">
          {reviews.length}
        </span>
      </div>
    </div>

    <div class="global-flex" style:margin-left="auto">
      <NakedButton
        variant="ghost"
        disabled={hasOwnReview}
        onclick={() => {
          const id = draftReviewStorage.create(rid, revision.id);

          void push({
            resource: "repo.patch",
            rid,
            patch: patchId,
            reviewId: id,
            status,
          });
        }}
        title={hasOwnReview ? "You already published a review" : undefined}>
        <Icon name="add" />
        <span class="txt-small">Review</span>
      </NakedButton>
    </div>
  </div>

  <div style:display={hideReviews ? "none" : "flex"} class="review-list">
    {#each reviews as review, idx}
      <ReviewTeaser
        {rid}
        {review}
        {patchId}
        {status}
        first={idx === 0}
        last={idx === reviews.length - 1} />
    {/each}
  </div>
</div>
