<script lang="ts">
  import Background from "./Header/Background.svelte";
  import Icon from "./Icon.svelte";
  import Label from "./Label.svelte";
</script>

<style>
  .flex-item {
    display: flex;
    align-items: center;
  }
  header {
    padding: 0 0.5rem;
    gap: 1.5rem;
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
      <Label
        styleBorderColor="var(--color-fill-ghost)"
        styleFillColor="var(--color-fill-ghost)">
        Repositories
      </Label>
      <Label styleBorderColor="var(--color-fill-ghost)">
        <Icon name="plus" />
      </Label>
    </div>
    <div class="flex-item">
      <Label styleBorderColor="var(--color-fill-ghost)">
        <Icon name="offline" /> Offline
      </Label>
    </div>
  </div>
  <Background />
</header>
