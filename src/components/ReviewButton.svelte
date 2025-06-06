<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Review } from "@bindings/cob/patch/Review";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Verdict } from "@bindings/cob/patch/Verdict";

  import { closeFocused } from "./Popover.svelte";
  import { didFromPublicKey } from "@app/lib/utils";
  import { push } from "@app/lib/router";

  import Border from "@app/components/Border.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import Popover from "@app/components/Popover.svelte";

  interface Props {
    rid: string;
    patchId: string;
    revision: Revision;
    config: Config;
    status: PatchStatus | undefined;
    loadPatch: () => Promise<void>;
    createReview: (verdict?: Verdict) => Promise<Review | undefined>;
  }

  const {
    rid,
    patchId,
    revision,
    config,
    status,
    loadPatch,
    createReview,
  }: Props = $props();

  const hasOwnReview = $derived(
    Boolean(
      revision.reviews &&
        revision.reviews.some(
          value => value.author.did === didFromPublicKey(config.publicKey),
        ),
    ),
  );

  let popoverExpanded: boolean = $state(false);
</script>

<Popover
  popoverPositionRight="0"
  popoverPositionTop="2.5rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <NakedButton
      variant="ghost"
      disabled={hasOwnReview}
      active={popoverExpanded}
      {onclick}
      title={hasOwnReview ? "You already published a review" : undefined}>
      <Icon name="add" />
      <span class="txt-small">Review</span>
    </NakedButton>
  {/snippet}

  {#snippet popover()}
    <Border
      variant="ghost"
      stylePadding="1rem"
      styleDisplay="flex"
      styleFlexDirection="column">
      <div class="global-flex">
        <OutlineButton
          variant="ghost"
          disabled={hasOwnReview}
          title={hasOwnReview ? "You already published a review" : undefined}
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
          <span class="global-flex" style:color="var(--color-foreground-dim)">
            <Icon name="comment" />
            <span class="txt-small">Write Review</span>
          </span>
        </OutlineButton>
        <OutlineButton
          variant="ghost"
          disabled={hasOwnReview}
          title={hasOwnReview ? "You already published a review" : undefined}
          onclick={async () => {
            await createReview("reject");
            await loadPatch();
            closeFocused();
          }}>
          <span class="global-flex" style:color="var(--color-foreground-red)">
            <Icon name="comment-cross" />
            <span class="txt-small">Reject</span>
          </span>
        </OutlineButton>
        <OutlineButton
          variant="ghost"
          disabled={hasOwnReview}
          title={hasOwnReview ? "You already published a review" : undefined}
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
      </div>

      <div
        class="txt-small txt-missing global-flex"
        style:margin-top="0.5rem"
        style:align-items="flex-start">
        <div style:padding-top="3px"><Icon name="info" /></div>
        <div>
          Clicking the buttons will create a blank review, add comments, a
          summary, and your verdict after. Depending on your sync settings your
          review might be published to the network right away. We are actively
          working on draft reviews, stay tuned.
        </div>
      </div>
    </Border>
  {/snippet}
</Popover>
