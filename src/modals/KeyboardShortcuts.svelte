<script lang="ts">
  import { cubicIn, cubicOut } from "svelte/easing";
  import { fly } from "svelte/transition";

  import { modifierKey } from "@app/lib/utils";

  type Shortcut = { keys: string[]; description: string };

  const shortcuts: Shortcut[] = [
    { keys: ["?"], description: "Keyboard shortcuts" },
    { keys: [modifierKey(), "f"], description: "Search the current list" },
    { keys: [modifierKey(), "n"], description: "New issue" },
    { keys: [modifierKey(), "+"], description: "Increase font size" },
    { keys: [modifierKey(), "-"], description: "Decrease font size" },
    { keys: [modifierKey(), "0"], description: "Reset font size" },
    { keys: ["esc"], description: "Close modal" },
  ];
</script>

<style>
  .modal {
    width: 30rem;
    display: flex;
    flex-direction: column;
    border-radius: var(--border-radius-lg);
    background-color: var(--color-surface-canvas);
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    padding: 0 1.5rem;
    height: 3.25rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .title {
    font: var(--txt-heading-s);
    color: var(--color-text-primary);
  }
  .rows {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    column-gap: 1rem;
    row-gap: 0.75rem;
    padding: 1.5rem;
  }
  .keys {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    justify-content: flex-end;
  }
  .key {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 1.75rem;
    min-width: 1.75rem;
    padding: 0 0.5rem;
    border-radius: var(--border-radius-md);
    background-color: var(--color-surface-subtle);
    font: var(--txt-body-m-medium);
    color: var(--color-text-primary);
  }
  .plus {
    font: var(--txt-body-m-regular);
    color: var(--color-text-tertiary);
  }
  .description {
    font: var(--txt-body-m-regular);
    color: var(--color-text-primary);
  }
</style>

<div
  class="modal"
  in:fly={{ y: 8, duration: 160, easing: cubicOut }}
  out:fly={{ y: 8, duration: 120, easing: cubicIn }}>
  <div class="header">
    <span class="title">Keyboard shortcuts</span>
  </div>
  <div class="rows">
    {#each shortcuts as shortcut (shortcut.description)}
      <div class="keys">
        {#each shortcut.keys as key, index (key)}
          {#if index > 0}
            <span class="plus">+</span>
          {/if}
          <span class="key">{key}</span>
        {/each}
      </div>
      <span class="description">{shortcut.description}</span>
    {/each}
  </div>
</div>
