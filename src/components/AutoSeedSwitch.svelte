<script lang="ts">
  import { onMount } from "svelte";

  import { invoke } from "@app/lib/invoke";

  import Button from "@app/components/Button.svelte";

  let enabled = $state<boolean | undefined>(undefined);

  onMount(async () => {
    try {
      enabled = await invoke<boolean>("get_auto_seed_artifacts");
    } catch (err) {
      console.error("get_auto_seed_artifacts failed", err);
      enabled = true;
    }
  });

  async function set(value: boolean) {
    // Optimistically reflect the choice, then roll back if the write fails
    // so the toggle never shows a state the backend didn't persist.
    const previous = enabled;
    enabled = value;
    try {
      await invoke("set_auto_seed_artifacts", { enabled: value });
    } catch (err) {
      console.error("set_auto_seed_artifacts failed", err);
      enabled = previous;
    }
  }
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
    active={enabled === true}
    onclick={() => set(true)}>
    On
  </Button>
  <Button
    variant="ghost"
    flatLeft
    active={enabled === false}
    onclick={() => set(false)}>
    Off
  </Button>
</div>
