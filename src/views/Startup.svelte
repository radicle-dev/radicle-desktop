<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Link } from "svelte-routing";

  import warningIcon from "/images/warning.png";

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
    padding-top: 7rem;
    margin: 0 auto;
    width: 100%;
    text-align: center;
  }
</style>

<main>
  {#if error}
    <img height="32" src={warningIcon} alt="warning" />
    <p class="txt-medium">{error.err}</p>
    {#if error.hint}
      <p class="txt-small">{@html error.hint}</p>
    {/if}
  {:else if !loading}
    <p class="txt-medium">You are all set!</p>
    <Link to="/repos">Repos</Link>
  {/if}
</main>
