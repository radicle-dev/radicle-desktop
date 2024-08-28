<script lang="ts">
  import Background from "./Header/Background.svelte";
  import Border from "./Border.svelte";
  import Fill from "./Fill.svelte";
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
</style>

<header class="flex-item">
  <div class="wrapper flex-item">
    <div class="wrapper-left flex-item">
      <div class="flex-item" style:gap="0.5rem">
        <Icon name="arrow-left" />
        <Icon name="arrow-right" />
      </div>
      <Fill variant="ghost" stylePadding="0 0.5rem" styleHeight="32px">
        Repositories
      </Fill>
      <Border variant="ghost" stylePadding="0 0.25rem" styleHeight="32px">
        <Icon name="plus" />
      </Border>
    </div>

    <div class="flex-item" style:gap="0.5rem">
      <Border variant="ghost" stylePadding="0 0.5rem" styleHeight="32px">
        <Icon name="offline" /> Offline
      </Border>
      <Popover popoverPositionRight="0" popoverPositionTop="3rem">
        <Border
          slot="toggle"
          let:toggle
          on:click={toggle}
          variant="ghost"
          stylePadding="0 0.25rem"
          styleHeight="32px">
          <Icon name="settings" />
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
