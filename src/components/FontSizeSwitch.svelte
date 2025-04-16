<script lang="ts">
  import {
    decreaseFontSize,
    fontSettings,
    increaseFontSize,
  } from "@app/lib/appearance.svelte";

  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
</script>

<style>
</style>

<div class="global-flex" style:gap="0">
  <Button
    flatRight
    variant="ghost"
    onclick={() => {
      decreaseFontSize();
    }}>
    <Icon name="minus" />
  </Button>

  <Button flatRight flatLeft active variant="ghost">
    {fontSettings.size}
  </Button>

  <Button
    flatLeft
    variant="ghost"
    onclick={() => {
      increaseFontSize();
    }}>
    <Icon name="plus" />
  </Button>
</div>
