<script lang="ts">
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";

  let { preview = $bindable(true) }: { preview?: boolean } = $props();
</script>

<style>
  .container {
    display: flex;
    align-items: center;
  }
</style>

<div class="container">
  <Button
    variant="ghost"
    flatRight
    active={!preview}
    onclick={() => {
      preview = !preview;
    }}>
    <Icon name="code" />Code
  </Button>

  <Button
    variant="ghost"
    flatLeft
    active={preview}
    onclick={() => {
      preview = !preview;
    }}>
    <Icon name="eye" />Preview
  </Button>
</div>
