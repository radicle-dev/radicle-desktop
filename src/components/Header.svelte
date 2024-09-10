<script lang="ts">
  import Background from "./Header/Background.svelte";
  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";
  import ThemeSwitch from "./ThemeSwitch.svelte";
</script>

<style>
  .flex-item {
    display: flex;
    align-items: center;
  }
  header {
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
</style>

<header class="flex-item">
  <div class="wrapper flex-item">
    <div class="wrapper-left flex-item">
      <div class="flex-item navigation" style:gap="0.5rem">
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

    <div class="flex-item" style:gap="0.5rem">
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

  <Background />
</header>
