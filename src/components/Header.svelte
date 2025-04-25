<script lang="ts">
  import type { Snippet } from "svelte";

  import * as router from "@app/lib/router";
  import { nodeRunning } from "@app/lib/events";

  import Avatar from "./Avatar.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";

  const activeRouteStore = router.activeRouteStore;

  interface Props {
    publicKey: string;
    center?: Snippet;
  }

  const { center, publicKey }: Props = $props();
</script>

<style>
  .header {
    height: 3rem;
    padding: 0.5rem 1rem;
    display: flex;
    align-items: flex-start;
  }
  .header:after {
    content: " ";
    position: absolute;
    top: 0;
    left: 0.5rem;
    right: 0.5rem;
    height: 3rem;
    z-index: -1;
    background-color: var(--color-background-float);
    clip-path: var(--3px-bottom-corner-fill);
  }
  .wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    row-gap: 8px;
  }
  .top-row {
    display: flex;
    width: 100%;
    justify-content: space-between;
  }
</style>

<div class="header global-flex">
  <div class="wrapper">
    <div class="top-row">
      <div class="global-flex" style:gap="0.25rem">
        <NakedButton
          variant="ghost"
          onclick={() => {
            void router.push({ resource: "home" });
          }}
          stylePadding="0 4px">
          <Avatar {publicKey} />
        </NakedButton>
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.back();
          }}
          stylePadding="0 4px">
          <Icon name="arrow-left" />
        </NakedButton>
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.forward();
          }}
          stylePadding="0 4px">
          <Icon name="arrow-right" />
        </NakedButton>
      </div>

      {@render center?.()}

      <div class="global-flex">
        <div
          class="global-flex txt-semibold txt-small"
          style:margin-right="0.25rem">
          {#if $nodeRunning}
            <Icon name="online" />
            Online
          {:else}
            <Icon name="offline" />
            Offline
          {/if}
        </div>
        <NakedButton
          variant="ghost"
          stylePadding="0 4px"
          active={$activeRouteStore.resource === "inbox"}
          onclick={() => router.push({ resource: "inbox" })}>
          <Icon name="inbox" />
        </NakedButton>
      </div>
    </div>
  </div>
</div>
