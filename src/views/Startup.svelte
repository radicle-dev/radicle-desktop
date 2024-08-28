<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  import Repos from "@app/views/Repos.svelte";
  import Icon from "@app/components/Icon.svelte";

  let loading = true;
  let error: { err: string; hint?: string } | undefined = undefined;

  onMount(async () => {
    try {
      await invoke("authenticate");
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (e: any) {
      error = e;
    } finally {
      loading = false;
    }
  });
</script>

<style>
  main {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100%;
    width: 100%;
    row-gap: 0.5rem;
  }
</style>

{#if error}
  <main>
    <Icon name="warning" size="32" />
    <div class="txt-medium txt-semibold">
      {error.err}
    </div>
    {#if error.hint}
      <div class="txt-small">{@html error.hint}</div>
    {/if}
  </main>
{:else if !loading}
  <Repos />
{/if}
