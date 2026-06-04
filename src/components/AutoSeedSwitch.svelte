<script lang="ts">
  import { autoSeedArtifacts, storeAutoSeed } from "@app/lib/autoSeed";

  import Button from "@app/components/Button.svelte";
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
    active={$autoSeedArtifacts === true}
    onclick={() => storeAutoSeed(true)}>
    On
  </Button>
  <Button
    variant="ghost"
    flatLeft
    active={$autoSeedArtifacts === false}
    onclick={() => storeAutoSeed(false)}>
    Off
  </Button>
</div>
