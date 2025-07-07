<script lang="ts">
  import Icon from "@app/components/Icon.svelte";

  const {
    name,
    fetchBlob,
    active,
  }: { name: string; fetchBlob: () => Promise<void>; active: boolean } =
    $props();
</script>

<style>
  .file {
    width: 100%;
    cursor: pointer;
  }
  .file:hover,
  .active {
    background-color: var(--color-background-float);
  }
  .active:hover {
    background-color: var(--color-fill-ghost-hover);
  }
  .file:hover .icon,
  .active .icon {
    color: var(--color-foreground-contrast);
  }
</style>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="file"
  class:active
  style:padding="0 0.5rem"
  style:margin="0.15rem 0"
  onclick={fetchBlob}>
  <div class="global-flex" style:padding="0.25rem 0">
    <div class="icon txt-missing">
      <Icon name="file" />
    </div>
    <div class="txt-small">
      {name}
    </div>
  </div>
</div>
