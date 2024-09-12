<script lang="ts">
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
    clip-path: polygon(
      0 0,
      100% 0,
      100% calc(100% - 6px),
      calc(100% - 2px) calc(100% - 6px),
      calc(100% - 2px) calc(100% - 4px),
      calc(100% - 4px) calc(100% - 4px),
      calc(100% - 4px) calc(100% - 2px),
      calc(100% - 6px) calc(100% - 2px),
      calc(100% - 6px) 100%,
      6px 100%,
      6px calc(100% - 2px),
      4px calc(100% - 2px),
      4px calc(100% - 4px),
      2px calc(100% - 4px),
      2px calc(100% - 6px),
      0 calc(100% - 6px)
    );
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
          <Icon name="offline" />
          Offline
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
    <div
      class="global-flex txt-tiny txt-semibold"
      style:gap="0.5rem"
      style:margin-left="1rem"
      style:min-height="1.5rem">
      <slot name="breadcrumbs" />
    </div>
  </div>
  <div class="bottom-pixel-corners"></div>
</div>
