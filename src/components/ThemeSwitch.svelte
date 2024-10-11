<script lang="ts" module>
  type Theme = "dark" | "light";

  export const theme = writable<Theme>(loadTheme());

  function loadTheme(): Theme {
    const { matches } = window.matchMedia("(prefers-color-scheme: dark)");
    const storedTheme = localStorage ? localStorage.getItem("theme") : null;

    if (storedTheme === null) {
      return matches ? "dark" : "light";
    } else {
      return storedTheme as Theme;
    }
  }

  export function storeTheme(newTheme: Theme): void {
    theme.set(newTheme);
    if (localStorage) {
      localStorage.setItem("theme", newTheme);
    } else {
      console.warn(
        "localStorage isn't available, not able to persist the selected theme without it.",
      );
    }
  }
</script>

<script lang="ts">
  import { writable } from "svelte/store";

  import Border from "./Border.svelte";
  import Icon from "./Icon.svelte";
</script>

<style>
  button {
    cursor: pointer;
    display: flex;
    align-items: center;
    border: none;
    white-space: nowrap;
    touch-action: manipulation;
    clip-path: var(--1px-corner-fill);
    height: 24px;
    font-size: var(--font-size-small);
    margin: 0;
  }
</style>

<Border variant="secondary">
  <button
    style:background-color={$theme === "dark"
      ? "var(--color-fill-secondary)"
      : "transparent"}
    onclick={() => {
      storeTheme("dark");
    }}>
    <span style="display: flex; align-items: center; gap: 0.5rem">
      <Icon name="moon" />
      Dark
    </span>
  </button>

  <button
    style:background-color={$theme === "light"
      ? "var(--color-fill-secondary)"
      : "transparent"}
    onclick={() => {
      storeTheme("light");
    }}>
    <span
      style="display: flex; align-items: center; gap: 0.5rem"
      style:color={$theme === "light"
        ? "var(--color-foreground-white)"
        : "var(--color-foreground-contrast)"}>
      <Icon name="sun" />
      Light
    </span>
  </button>
</Border>
