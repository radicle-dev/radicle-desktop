<script lang="ts" module>
  export const announce = writable<boolean>(loadAnnounce());

  function loadAnnounce(): boolean {
    const storedAnnounce = localStorage
      ? localStorage.getItem("announce")
      : null;

    if (storedAnnounce === null) {
      return true;
    } else {
      return storedAnnounce === "true";
    }
  }

  export function storeAnnounce(newAnnounce: boolean): void {
    announce.set(newAnnounce);
    if (localStorage) {
      localStorage.setItem("announce", newAnnounce.toString());
    } else {
      console.warn(
        "localStorage isn't available, not able to persist the selected announce preference without it.",
      );
    }
  }
</script>

<script lang="ts">
  import { writable } from "svelte/store";
</script>

<style>
  .container {
    background-color: var(--color-fill-ghost);
    clip-path: var(--2px-corner-fill);
    display: flex;
    height: 32px;
    align-items: center;
    padding: 0 2px;
  }
  button {
    height: 28px;
    cursor: pointer;
    display: flex;
    align-items: center;
    border: none;
    white-space: nowrap;
    touch-action: manipulation;
    clip-path: var(--1px-corner-fill);
    font-size: var(--font-size-small);
    margin: 0;
    padding: 0 1rem;
    color: var(--color-foreground-contrast);
    background-color: var(--color-fill-ghost);
    font-weight: var(--font-weight-semibold);
  }

  .active {
    color: var(--color-foreground-emphasized);
    background-color: var(--color-background-dip);
  }
</style>

<div class="container">
  <button
    class:active={$announce}
    onclick={() => {
      storeAnnounce(true);
    }}>
    Right away
  </button>

  <button
    class:active={!$announce}
    onclick={() => {
      storeAnnounce(false);
    }}>
    Periodically
  </button>
</div>
