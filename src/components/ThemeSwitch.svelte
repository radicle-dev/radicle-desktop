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
  import Fill from "./Fill.svelte";
  import Icon from "./Icon.svelte";
</script>

<div style="display: flex; gap: 1rem;">
  <Border variant="secondary">
    <Fill
      stylePadding="0 0.5rem"
      variant={$theme === "dark" ? "secondary" : "transparent"}
      on:click={() => {
        storeTheme("dark");
      }}>
      <Icon name="moon" />
      Dark
    </Fill>

    <Fill
      stylePadding="0 0.5rem"
      variant={$theme === "light" ? "secondary" : "transparent"}
      on:click={() => {
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
    </Fill>
  </Border>
</div>
