<script lang="ts">
  import Icon from "@app/components/Icon.svelte";

  export let error: string;
  export let hint: string | undefined = undefined;
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

  /* This tag comes from the backend. */
  :global(code) {
    font-family: var(--font-family-monospace);
    font-size: var(--font-size-small);
    background-color: var(--color-fill-ghost);
    padding: 0.125rem 0.25rem;
  }
</style>

<main>
  <Icon name="warning" size="32" />
  <div class="txt-medium txt-semibold">
    {error}
  </div>
  {#if hint}
    <div class="txt-small">{@html hint}</div>
  {/if}
</main>
