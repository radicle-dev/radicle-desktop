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

  import Icon from "./Icon.svelte";
  import Button from "./Button.svelte";
</script>

<style>
  .container {
    display: flex;
    align-items: center;
  }
</style>

<div class="container">
  <Button
    flatRight
    active={$theme === "dark"}
    variant="ghost"
    onclick={() => {
      storeTheme("dark");
    }}>
    <Icon name="moon" />
    Dark
  </Button>

  <Button
    flatLeft
    variant="ghost"
    active={$theme === "light"}
    onclick={() => {
      storeTheme("light");
    }}>
    <Icon name="sun" />
    Light
  </Button>
</div>
