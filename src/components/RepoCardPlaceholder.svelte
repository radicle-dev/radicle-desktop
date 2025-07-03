<script lang="ts">
  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import { formatRepositoryId } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import Border from "@app/components/Border.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";

  interface Props {
    reload: () => Promise<void>;
    rid: string;
  }

  const { reload, rid }: Props = $props();

  async function unseed() {
    try {
      await invoke<null>("unseed", {
        rid: rid,
        opts: { announce: $nodeRunning && $announce },
      });
      await reload();
    } catch (error) {
      console.error("Seeding failed", error);
    }
  }
</script>

<style>
  .unseed {
    display: none;
    color: var(--color-fill-gray);
    height: 1.375rem;
  }
  .container:hover .unseed {
    display: flex;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    gap: 0.5rem;
  }
  .footer {
    margin-top: 1rem;
  }
</style>

<div class="container">
  <Border
    variant="float"
    styleWidth="100%"
    styleHeight="8.375rem"
    styleAlignItems="flex-start"
    styleFlexDirection="column"
    styleGap="0"
    stylePadding="0.5rem 0.75rem"
    styleOverflow="hidden">
    <div class="header txt-small">
      <div class="global-flex txt-overflow">
        <div
          class="global-counter"
          style:background-color="var(--color-fill-ghost)">
        </div>
        <span class="global-flex" style:height="1.375rem">
          <div
            style:height="1rem"
            style:width="7rem"
            style:background-color="var(--color-fill-ghost)">
          </div>
        </span>
      </div>
      <div class="global-flex">
        <div class="global-flex unseed">
          <NakedButton
            stylePadding="0 0.25rem"
            variant="ghost"
            onclick={unseed}>
            <Icon name="broom" />
            Remove
          </NakedButton>
        </div>
      </div>
    </div>
    <div class="global-flex" style:height="1.375rem" style:margin-top="0.25rem">
      <div
        style:height="0.875rem"
        style:width="13rem"
        style:background-color="var(--color-fill-ghost)">
      </div>
    </div>
    <Id
      ariaLabel="repo-id"
      clipboard={rid}
      shorten={false}
      variant="oid"
      id={formatRepositoryId(rid)} />

    <div
      class="global-flex footer txt-small"
      style:margin-top="auto"
      style:width="100%">
      <span
        title={$nodeRunning
          ? "This may take a while depending on your network connectivity and repo size."
          : "Your node is offline. Start your node to fetch this repo."}
        class="global-flex"
        style:color="var(--color-fill-gray)"
        style:margin-left="auto">
        <Icon name="hourglass" />
        Queued for fetching…
      </span>
    </div>
  </Border>
</div>
