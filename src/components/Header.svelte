<script lang="ts">
  import type { Snippet } from "svelte";

  import { nodeRunning } from "@app/lib/events";

  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";

  interface Props {
    breadcrumbs: Snippet;
    columnSwitch?: Snippet;
    center?: Snippet;
    settingsButton?: Snippet;
  }

  const { breadcrumbs, columnSwitch, center, settingsButton }: Props = $props();
</script>

<style>
  .header {
    height: 5rem;
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
    height: 5rem;
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
  .bottom-row {
    display: flex;
    gap: 0.5rem;
    font-size: var(--font-size-tiny);
    font-weight: var(--font-weight-semibold);
    align-items: center;

    min-height: 1.5rem;
    width: 100%;
    padding-left: 12px;
    /* Fixed height so that the navigation arrow buttons don't jump vertically
       when the column buttons aren't shown on the Home view vs Repo view. */
    height: 24px;
  }
</style>

<div class="header global-flex">
  <div class="wrapper">
    <div class="top-row">
      <div class="global-flex" style:gap="0">
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.back();
          }}>
          <Icon name="arrow-left" />
        </NakedButton>
        <NakedButton
          variant="ghost"
          onclick={() => {
            window.history.forward();
          }}>
          <Icon name="arrow-right" />
        </NakedButton>
      </div>

      {@render center?.()}

      <div class="global-flex">
        {@render settingsButton?.()}
        <OutlineButton variant="ghost">
          {#if $nodeRunning}
            <Icon name="online" />
            Online
          {:else}
            <Icon name="offline" />
            Offline
          {/if}
        </OutlineButton>
      </div>
    </div>

    <div class="bottom-row">
      {@render breadcrumbs()}
      {#if columnSwitch}
        <div style:margin-left="auto">
          {@render columnSwitch()}
        </div>
      {/if}
    </div>
  </div>
</div>
