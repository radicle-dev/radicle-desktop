<script lang="ts">
  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";
  import ThemeSwitch from "./ThemeSwitch.svelte";
</script>

<style>
  .header {
    padding: 0 0.5rem;
    gap: 0.25rem;
    height: 3rem;
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
  .navigation :global(svg:hover) {
    display: flex;
    color: var(--color-fill-secondary);
  }
  .bottom-pixel-corners {
    position: absolute;
    top: 0;
    left: 0.5rem;
    right: 0.5rem;
    height: 3rem;
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
  <div class="wrapper global-flex">
    <div class="wrapper-left global-flex">
      <div class="global-flex navigation" style:gap="0.5rem">
        <Icon
          name="arrow-left"
          onclick={() => {
            window.history.back();
          }} />
        <Icon
          name="arrow-right"
          onclick={() => {
            window.history.forward();
          }} />
      </div>
      <slot name="icon-left" />
    </div>

    <slot name="center" />

    <div class="global-flex" style:gap="0.5rem">
      <Border variant="ghost" stylePadding="0 0.5rem" styleHeight="32px">
        <Icon name="offline" />
        <span class="txt-small txt-semibold">Offline</span>
      </Border>
      <Popover popoverPositionRight="0" popoverPositionTop="3rem">
        <Border
          slot="toggle"
          let:toggle
          onclick={toggle}
          variant="ghost"
          stylePadding="0 0.25rem"
          styleHeight="32px">
          <Icon name="more-vertical" />
        </Border>
        <Border variant="ghost" slot="popover" stylePadding="0.5rem 1rem">
          <div style="display: flex; gap: 2rem; align-items: center;">
            Theme <ThemeSwitch />
          </div>
        </Border>
      </Popover>
    </div>
  </div>
  <div class="bottom-pixel-corners"></div>
</div>
