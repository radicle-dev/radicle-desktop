<script lang="ts">
  import { nodeState } from "@app/lib/events";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import OutlineButton from "./OutlineButton.svelte";
  import Popover from "./Popover.svelte";
  import ThemeSwitch from "./ThemeSwitch.svelte";
</script>

<style>
  .header {
    padding: 0 0.5rem;
    gap: 0.25rem;
    height: 5rem;
  }
  .wrapper {
    width: 100%;
    justify-content: space-between;
    padding: 0 0.5rem;
  }
  .wrapper-left {
    gap: 0.5rem;
    padding: 0 0.5rem;
  }
  .bottom-pixel-corners {
    position: absolute;
    top: 0;
    left: 0.5rem;
    right: 0.5rem;
    height: 5rem;
    z-index: -1;

    background-color: var(--color-background-float);
    clip-path: var(--3px-bottom-corner-fill);
  }
  .breadcrumbs {
    gap: 0.5rem;
    margin-left: 1rem;
    min-height: 1.5rem;
  }
</style>

<div class="header global-flex">
  <div
    class="global-flex"
    style:flex-direction="column"
    style:width="100%"
    style:align-items="flex-start">
    <div class="wrapper global-flex">
      <div class="wrapper-left global-flex" style:gap="0">
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
        <slot name="icon-left" />
      </div>

      <slot name="center" />

      <div class="global-flex" style:gap="0.5rem">
        <OutlineButton variant="ghost">
          {#if $nodeState === "running"}
            <Icon name="online" />
            Online
          {:else}
            <Icon name="offline" />
            Offline
          {/if}
        </OutlineButton>
        <Popover popoverPositionRight="0" popoverPositionTop="3rem">
          <NakedButton
            variant="ghost"
            slot="toggle"
            let:toggle
            onclick={toggle}>
            <Icon name="more-vertical" />
          </NakedButton>
          <Border variant="ghost" slot="popover" stylePadding="0.5rem 1rem">
            <div style="display: flex; gap: 2rem; align-items: center;">
              Theme <ThemeSwitch />
            </div>
          </Border>
        </Popover>
      </div>
    </div>
    <div class="global-flex txt-tiny txt-semibold breadcrumbs">
      <slot name="breadcrumbs" />
    </div>
  </div>
  <div class="bottom-pixel-corners"></div>
</div>
